use kube::runtime::watcher::Event as WatcherEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

pub fn handle_resource_event<T: kube::Resource + Clone + serde::Serialize>(
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
