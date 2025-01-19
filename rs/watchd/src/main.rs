use k8s_openapi::api::core::v1::Event;
use kube::{api::DynamicObject, Api, Client};
use shared::{client::Hik8sClient, tracing::setup_tracing};
use std::error::Error;
use tracing::warn;
use watchd::{
    constant::{ROUTE_CUSTOM_RESOURCE, ROUTE_EVENT},
    customresource::{get_api_resource, list_crds, verify_access},
    kubeapi::KubeApi,
    watcher::setup_watcher,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_tracing()?;

    // Create kube apiserver client
    let kube_client = Client::try_default().await?;

    // Create Hik8sClient
    let hik8s_client = Hik8sClient::new(true).unwrap();

    // Setup Event watcher
    let event_api = Api::<Event>::all(kube_client.clone());
    setup_watcher("Event", event_api, hik8s_client.clone(), ROUTE_EVENT, false).await?;

    // Setup Resource watcher
    for resource in KubeApi::new_all(&kube_client) {
        resource.setup_watcher(hik8s_client.clone()).await?;
    }

    // Setup CustomResource watcher
    let mut failed_cr_names = vec![];
    for cr in list_crds(kube_client.clone(), true).await? {
        if let Some(api_resource) = get_api_resource(&cr) {
            let dynamic_api = Api::<DynamicObject>::all_with(kube_client.clone(), &api_resource);
            let name_with_group = format!("{}/{}", api_resource.group, api_resource.plural);
            if (verify_access(&dynamic_api).await).is_ok() {
                setup_watcher(
                    &name_with_group,
                    dynamic_api,
                    hik8s_client.clone(),
                    ROUTE_CUSTOM_RESOURCE,
                    true,
                )
                .await?;
            } else {
                failed_cr_names.push(name_with_group);
            };
        }
    }

    if !failed_cr_names.is_empty() {
        warn!(
            "Failed to setup watchers for CustomResources: {:?}",
            failed_cr_names.join(", ")
        );
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}
