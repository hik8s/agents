use futures::StreamExt;
use kube::api::{ApiResource, DynamicObject};

use kube::runtime::{reflector, watcher};
use kube::{Api, Client};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::resource::handle_resource_event;

pub async fn setup_watcher<T: Clone + std::fmt::Debug + Send + Sync + 'static>(
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

pub async fn setup_dynamic_watcher(
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
