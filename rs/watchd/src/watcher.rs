use futures::StreamExt;

use crate::constant::LOCAL_THREAD_LIMIT;
use k8s_openapi::chrono;
use kube::runtime::watcher::Error as WatcherError;
use kube::runtime::watcher::{watcher, Event as WatcherEvent};
use kube::Api;
use serde::Serialize;
use shared::client::Hik8sClient;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::error;

pub async fn setup_watcher<T>(
    api: Api<T>,
    hik8s_client: Hik8sClient,
    route: &'static str,
    report_deleted: bool,
) -> Result<(), Box<dyn Error>>
where
    T: kube::Resource + Debug + Clone + Send + Sync + 'static,
    T: for<'kubeapi> serde::Deserialize<'kubeapi> + Serialize,
{
    let watcher = watcher(api, Default::default());

    let thread_limit = Arc::new(Semaphore::new(LOCAL_THREAD_LIMIT));

    // Poll the stream to keep the store up-to-date
    tokio::spawn(async move {
        watcher
            .for_each(|event| async {
                let client = hik8s_client.clone();
                if let Ok(permit) = thread_limit.clone().acquire_owned().await {
                    tokio::spawn(async move {
                        match event {
                            Ok(watcher_event) => {
                                handle_event_and_dispatch(
                                    watcher_event,
                                    client,
                                    route,
                                    report_deleted,
                                )
                                .await
                            }
                            Err(e) => match e {
                                WatcherError::WatchError(err_res) => match err_res.code {
                                    403 => {}
                                    _ => error!("Watcher error: {:?}", err_res),
                                },
                                _ => error!("Watcher error: {:?}", e),
                            },
                        };
                        drop(permit);
                    });
                }
            })
            .await;
    });
    Ok(())
}

pub async fn handle_event_and_dispatch<T: Serialize>(
    event: WatcherEvent<T>,
    client: Hik8sClient,
    route: &str,
    report_deleted: bool,
) {
    match event {
        WatcherEvent::Apply(resource) => {
            let json = wrap_kubeapi_data(resource, "apply");
            if let Err(e) = client.send_request(route, &json).await {
                tracing::error!("Failed to handle apply event: {}", e);
            }
            tracing::info!("{route}(Apply)");
        }
        WatcherEvent::InitApply(resource) => {
            let json = wrap_kubeapi_data(resource, "initapply");
            if let Err(e) = client.send_request(route, &json).await {
                tracing::error!("Failed to handle init-apply event: {}", e);
            }
            tracing::info!("{route}(InitApply)");
        }
        WatcherEvent::Init => tracing::info!("{route}(init)"),
        WatcherEvent::InitDone => tracing::info!("{route}(initdone)"),
        WatcherEvent::Delete(resource) => {
            if report_deleted {
                let json = wrap_kubeapi_data(resource, "delete");
                if let Err(e) = client.send_request(route, &json).await {
                    tracing::error!("Failed to handle delete event: {}", e);
                }
            }
            tracing::info!("{route}(Delete)");
        }
    }
}

fn wrap_kubeapi_data<T: Serialize>(resource: T, event_type: &str) -> serde_json::Value {
    serde_json::json!({
        "timestamp": chrono::Utc::now().timestamp(),
        "event_type": event_type,
        "json": resource,
    })
}
