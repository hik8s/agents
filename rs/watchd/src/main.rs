use kube::{api::DynamicObject, Api, Client};
use shared::{client::Hik8sClient, tracing::setup_tracing};
use std::{collections::HashMap, error::Error};
use tracing::{info, warn};
use watchd::{
    constant::{CLUSTER_ROLE_NAME, ROUTE_CUSTOM_RESOURCE},
    customresource::{get_api_resource, list_crds},
    kubeapi::KubeApiResource,
    watcher::setup_watcher,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_tracing()?;

    // Create clients
    let kubeapi_client = Client::try_default().await?;
    let hik8s_client = Hik8sClient::new(false).unwrap();

    // Setup resource watcher
    let mut failed_resource_names = vec![];
    for resource in KubeApiResource::new_all(&kubeapi_client) {
        let name = resource.to_string();
        resource
            .setup_watcher(hik8s_client.clone())
            .await
            .inspect_err(|_| failed_resource_names.push(name))
            .ok();
    }
    if !failed_resource_names.is_empty() {
        warn!("{}", format_rbac_error(failed_resource_names.join(", ")));
    }

    // Setup custom resource watcher
    let mut grouped_resources: HashMap<String, Vec<String>> = HashMap::new();

    for cr in list_crds(kubeapi_client.clone(), true).await? {
        if let Some(api_resource) = get_api_resource(&cr) {
            let dynamic_api = Api::<DynamicObject>::all_with(kubeapi_client.clone(), &api_resource);

            let name_with_group = format!("{}/{}", api_resource.group, api_resource.plural);
            setup_watcher(
                name_with_group.clone(),
                dynamic_api,
                hik8s_client.clone(),
                ROUTE_CUSTOM_RESOURCE,
                true,
            )
            .await
            .inspect_err(|_| {
                grouped_resources
                    .entry(api_resource.group)
                    .or_default()
                    .push(api_resource.plural);
            })
            .ok();
        }
    }

    if !grouped_resources.is_empty() {
        let flattened: Vec<String> = grouped_resources
            .iter()
            .flat_map(|(group, resources)| {
                resources
                    .iter()
                    .map(|resource| format!("{}/{}", group.clone(), resource))
            })
            .collect();
        warn!("{}", format_rbac_error(flattened.join(", ")));
        info!("{}", format_rbac_update(grouped_resources));
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

fn format_rbac_error(affected_resources: String) -> String {
    format!(
        r#"Failed to setup watchers for resources: {affected_resources}. \
Check if RBAC containes ["get", "list", "watch"] permissions."#
    )
}
fn format_rbac_update(resources: HashMap<String, Vec<String>>) -> String {
    let mut result = format!(
        "To fix this RBAC access, add the following rules to the ClusterRole {CLUSTER_ROLE_NAME}:\nrules:"
    );
    for (group, resources) in resources {
        result.push_str(&format!(
            r#"
- apiGroups: ["{}"]
  resources: ["{}"]
  verbs: ["get", "list", "watch"]"#,
            group,
            resources.join(r#"", ""#)
        ));
    }
    result
}
