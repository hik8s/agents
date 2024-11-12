use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{ApiResource, DynamicObject, GroupVersionKind, ListParams, ObjectList};

use kube::{Api, Client};
use std::error::Error;

use crate::watcher::setup_dynamic_watcher;

pub async fn list_all_custom_resources(
    client: Client,
) -> Result<ObjectList<DynamicObject>, Box<dyn Error>> {
    // Step 1: List all CRDs
    let crds: Api<CustomResourceDefinition> = Api::all(client.clone());
    let mut crd_list = crds.list(&ListParams::default()).await?;

    // Step 2: Sort CRDs by group
    crd_list
        .items
        .sort_by(|a, b| a.spec.group.cmp(&b.spec.group));
    // Step 2: Iterate over each CRD and list its custom resources
    for crd in crd_list {
        if let Some(crd_name) = crd.metadata.name {
            let group = crd.spec.group;
            if let Some(version) = crd.spec.versions.first() {
                let version_name = &version.name;
                let plural_name = &crd.spec.names.plural;

                // println!("Listing Custom Resources for CRD: {}", crd_name);

                // Step 3: Dynamically create an API object for the custom resource
                let gvk = GroupVersionKind::gvk(&group, version_name, &crd.spec.names.kind);
                let api_resource = ApiResource::from_gvk(&gvk);
                let dynamic_api: Api<DynamicObject> = Api::all_with(client.clone(), &api_resource);

                // Step 4: List all instances of this custom resource
                match dynamic_api.list(&ListParams::default()).await {
                    Ok(custom_resources) => {
                        for cr in &custom_resources {
                            println!(
                                "Group: {}, Version: {}, Plural: {} CRD: {}, CR: {}",
                                group,
                                version_name,
                                plural_name,
                                crd_name,
                                cr.metadata.name.clone().unwrap_or_default()
                            );
                            setup_dynamic_watcher(client.clone(), &api_resource).await?;
                            // println!(
                            //     "Found Custom Resource: {}",
                            //     cr.metadata.name.unwrap_or_default()
                            // );
                        }
                        // return Ok(custom_resources);
                    }
                    Err(e) => {
                        return Err(
                            format!("Failed to list resources for {}: {:?}", crd_name, e).into(),
                        )
                    }
                }
            }
        }
    }

    Err(format!("Failed to list resources").into())
}

async fn list_crds(client: Client) -> Result<(), Box<dyn Error>> {
    // Define an API object for CustomResourceDefinitions (CRDs)
    let crds: Api<CustomResourceDefinition> = Api::all(client);

    // List all CRDs
    let lp = ListParams::default();
    let crd_list = crds.list(&lp).await?;

    // Print each CRD's name
    for crd in crd_list {
        if let Some(crd_name) = crd.metadata.name {
            println!("Found CRD: {}", crd_name);
        }
    }

    Ok(())
}
