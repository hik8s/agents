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
use tracing::{debug, error};

pub async fn setup_watcher<T>(
    name: &str,
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
    let name = name.to_owned();
    tokio::spawn(async move {
        watcher
            .for_each(|event| async {
                let client = hik8s_client.clone();
                if let Ok(permit) = thread_limit.clone().acquire_owned().await {
                    let name = name.clone();
                    tokio::spawn(async move {
                        match event {
                            Ok(watcher_event) => {
                                handle_event_and_dispatch(
                                    &name,
                                    watcher_event,
                                    client,
                                    route,
                                    report_deleted,
                                )
                                .await
                            }
                            Err(err) => match err {
                                WatcherError::WatchError(res) => match res.code {
                                    403 => {}
                                    _ => error!("Error: {} watcher: {:?}", name, res),
                                },
                                WatcherError::InitialListFailed(kube_error) => match kube_error {
                                    kube::Error::Api(res) => match res.code {
                                        403 => {}
                                        _ => error!("Error: {} watcher: {:?}", name, res),
                                    },
                                    err => error!("Error: {} watcher: {:?}", name, err),
                                },
                                _ => error!("Error: {} watcher: {:?}", name, err),
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
    name: &str,
    event: WatcherEvent<T>,
    client: Hik8sClient,
    route: &str,
    report_deleted: bool,
) {
    match event {
        WatcherEvent::Apply(resource) => {
            let json = wrap_kubeapi_data(resource, "apply");
            if let Err(e) = client.send_request(route, &json).await {
                error!("Failed to handle apply event for resource {name}: {e}");
            }
            debug!("{route}(Apply): {name}");
        }
        WatcherEvent::InitApply(resource) => {
            let json = wrap_kubeapi_data(resource, "initapply");
            if let Err(e) = client.send_request(route, &json).await {
                error!("Failed to handle init-apply event for resource {name}: {e}");
            }
            debug!("{route}(InitApply): {name}");
        }
        WatcherEvent::Init => debug!("{route}(init)"),
        WatcherEvent::InitDone => debug!("{route}(initdone)"),
        WatcherEvent::Delete(resource) => {
            if report_deleted {
                let json = wrap_kubeapi_data(resource, "delete");
                if let Err(e) = client.send_request(route, &json).await {
                    error!("Failed to handle delete event for resource {name}: {e}");
                }
            }
            debug!("{route}(Delete): {name}");
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
