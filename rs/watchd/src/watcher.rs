use futures::StreamExt;

use kube::runtime::watcher::{watcher, Event as WatcherEvent};
use kube::Api;
use serde::Serialize;
use shared::client::Hik8sClient;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::constant::LOCAL_THREAD_LIMIT;
use tracing::error;

pub async fn setup_watcher<T: kube::Resource>(
    api: Api<T>,
    hik8s_client: Hik8sClient,
    route: &'static str,
    report_deleted: bool,
) -> Result<(), Box<dyn Error>>
where
    T: Debug + Clone + Send + Sync + 'static,
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
                            Err(e) => error!("Watch error, route {}: {:?}", route, e),
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
        WatcherEvent::Apply(resource) | WatcherEvent::InitApply(resource) => {
            if let Err(e) = client.dispatch(resource, route).await {
                tracing::error!("Failed to handle delete event: {}", e);
            }
        }
        WatcherEvent::Init => tracing::info!("{route}(init)"),
        WatcherEvent::InitDone => tracing::info!("{route}(initdone)"),
        WatcherEvent::Delete(resource) => {
            if report_deleted {
                if let Err(e) = client.dispatch(resource, route).await {
                    tracing::error!("Failed to handle delete event: {}", e);
                }
            }
        }
    }
}
