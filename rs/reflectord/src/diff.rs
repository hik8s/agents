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

pub fn diff_resource_json<T: serde::Serialize>(old_resource: &T, new_resource: &T) {
    let old_value = serde_json::to_value(old_resource).unwrap();
    let new_value = serde_json::to_value(new_resource).unwrap();
    diff_value("", &old_value, &new_value);
}
