use futures::StreamExt;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::{ApiResource, DynamicObject, GroupVersionKind, ListParams};
use kube::runtime::watcher::Event as WatcherEvent;
use kube::runtime::{reflector, watcher};
use kube::{Api, Client};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
mod process;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::try_default().await?;

    // Setup Pod watcher
    setup_watcher::<Event>(client.clone()).await?;
    // setup_watcher::<CustomResourceDefinition>(client.clone()).await?;

    // list_all_custom_resources(client).await?;
    // Keep the main thread alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
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

async fn list_all_custom_resources(client: Client) -> Result<(), Box<dyn Error>> {
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
                        for cr in custom_resources {
                            println!(
                                "Group: {}, Version: {}, Plural: {} CRD: {}, CR: {}",
                                group,
                                version_name,
                                plural_name,
                                crd_name,
                                cr.metadata.name.unwrap_or_default()
                            );
                            setup_dynamic_watcher(client.clone(), &api_resource).await?;
                            // println!(
                            //     "Found Custom Resource: {}",
                            //     cr.metadata.name.unwrap_or_default()
                            // );
                        }
                    }
                    Err(e) => eprintln!("Failed to list resources for {}: {:?}", crd_name, e),
                }
            }
        }
    }

    Ok(())
}

fn print_resource_changes<T>(
    resource: &T,
    resource_state: &Arc<Mutex<HashMap<String, T>>>,
    update: bool,
    delete: bool,
    print: bool,
    operation: &str,
) where
    T: kube::Resource + Clone + serde::Serialize,
{
    let uid = resource.meta().uid.clone().unwrap_or_default();
    let name = resource.meta().name.clone().unwrap_or_default();
    println!("{operation} '{}' with uid {}", name, uid);

    let mut resource_state = resource_state.lock().unwrap();
    if let Some(old_resource) = resource_state.get(&uid) {
        if print {
            diff_resource_json::<T>(old_resource, resource);
        }
    }
    if update {
        resource_state.insert(uid.clone(), resource.clone());
    }
    if delete {
        resource_state.remove(&uid);
    }
}

fn handle_resource_event<T: kube::Resource + Clone + serde::Serialize>(
    event: WatcherEvent<T>,
    state: &Arc<Mutex<HashMap<String, T>>>,
) where
    T: kube::Resource + Clone + serde::Serialize,
{
    match event {
        WatcherEvent::Apply(resource) => {
            print_resource_changes(&resource, state, true, false, true, "Apply");
        }
        WatcherEvent::Delete(resource) => {
            print_resource_changes(&resource, state, false, true, true, "Delete");
        }
        WatcherEvent::Init => {
            println!("Init");
        }
        WatcherEvent::InitApply(resource) => {
            print_resource_changes(&resource, state, true, false, false, "InitApply");
        }
        WatcherEvent::InitDone => {
            println!("Init done");
        }
    }
}

async fn setup_watcher<T: Clone + std::fmt::Debug + Send + Sync + 'static>(
    client: Client,
) -> Result<(), Box<dyn Error>>
where
    T: kube::Resource + std::fmt::Debug + Clone + Send + Sync + 'static,
    T: for<'de> serde::Deserialize<'de>,
    T: serde::Serialize,
    <T as kube::Resource>::DynamicType: Default + Eq + Hash + Clone,
{
    let api: Api<T> = Api::all(client);
    let (reader, writer) = reflector::store();
    let rf = reflector(writer, watcher(api, Default::default()));

    let state = Arc::new(Mutex::new(HashMap::new()));
    let state_clone = Arc::clone(&state);

    // Poll the stream to keep the store up-to-date
    tokio::spawn(async move {
        rf.for_each(|event| {
            let state = Arc::clone(&state_clone);
            async move {
                if let Ok(event) = event {
                    handle_resource_event(event, &state);
                }
            }
        })
        .await;
    });

    // Example: Access cached data from the reader (store)
    Ok(())
}

async fn setup_dynamic_watcher(
    client: Client,
    api_resource: &ApiResource,
) -> Result<(), Box<dyn Error>> {
    let dynamic_api: Api<DynamicObject> = Api::all_with(client, api_resource);

    let state = Arc::new(Mutex::new(HashMap::<String, DynamicObject>::new()));
    let state_clone = Arc::clone(&state);

    // Setup watcher without store
    let watcher = watcher(dynamic_api, watcher::Config::default());

    tokio::spawn(async move {
        watcher
            .for_each(|event| {
                let state = Arc::clone(&state_clone);
                async move {
                    if let Ok(event) = event {
                        handle_resource_event(event, &state);
                    }
                }
            })
            .await;
    });

    Ok(())
}

use serde_json::Value;

fn compare_and_print(path: &str, old: &Value, new: &Value) {
    if old != new {
        println!("  {} changed from {:?} to {:?}", path, old, new);
    }
}

fn diff_value(path: &str, old: &Value, new: &Value) {
    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            for (key, old_value) in old_map {
                let new_value = new_map.get(key).unwrap_or(&Value::Null);
                let new_path = format!("{}.{}", path, key);
                diff_value(&new_path, old_value, new_value);
            }
            for (key, new_value) in new_map {
                if !old_map.contains_key(key) {
                    let new_path = format!("{}.{}", path, key);
                    diff_value(&new_path, &Value::Null, new_value);
                }
            }
        }
        (Value::Array(old_array), Value::Array(new_array)) => {
            for (index, (old_value, new_value)) in old_array.iter().zip(new_array).enumerate() {
                let new_path = format!("{}[{}]", path, index);
                diff_value(&new_path, old_value, new_value);
            }
        }
        _ => compare_and_print(path, old, new),
    }
}

fn diff_resource_json<T: serde::Serialize>(old_resource: &T, new_resource: &T) {
    let old_value = serde_json::to_value(old_resource).unwrap();
    let new_value = serde_json::to_value(new_resource).unwrap();
    diff_value("", &old_value, &new_value);
}
