use cargo_toml::{Manifest, Value};

fn extract_string(v: &Value) -> String {
  if let Value::String(v) = v {
    v.clone()
  } else {
    panic!("metadata.sandboxes contains non strings");
  }
}

pub fn get_sandboxes(manifest: &Manifest) -> Vec<String> {
  let metadata = manifest.package.as_ref().unwrap().metadata.as_ref();
  if metadata.is_none() {
    return Vec::new();
  }

  if let Value::Table(metadata) = metadata.unwrap() {
    let sandboxes_metadata = metadata.get("sandboxes");
    if sandboxes_metadata.is_none() {
      return Vec::new();
    }

    if let Value::Array(vec) = sandboxes_metadata.unwrap() {
      return vec.iter().map(extract_string).collect();
    }
  }

  return Vec::new();
}
