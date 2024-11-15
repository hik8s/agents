use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{ApiResource, GroupVersionKind, ListParams, ObjectList};

use kube::{Api, Client};
use std::error::Error;

pub fn get_api_resource(crd: &CustomResourceDefinition) -> Option<ApiResource> {
    let group = &crd.spec.group;

    // Get latest version
    // TODO: handle multiple versions?
    let version = crd.spec.versions.first()?;
    let version_name = &version.name;

    // Create API resource
    let gvk = GroupVersionKind::gvk(group, version_name, &crd.spec.names.kind);
    let api_resource = ApiResource::from_gvk(&gvk);

    Some(api_resource)
}

pub async fn list_crds(
    client: Client,
    sort: bool,
) -> Result<ObjectList<CustomResourceDefinition>, Box<dyn Error>> {
    // Define an API object for CustomResourceDefinitions (CRDs)
    let crds: Api<CustomResourceDefinition> = Api::all(client);

    // List all CRDs
    let lp = ListParams::default();
    let mut crd_list = crds.list(&lp).await?;

    if sort {
        crd_list
            .items
            .sort_by(|a, b| a.spec.group.cmp(&b.spec.group));
    }
    Ok(crd_list)
}
