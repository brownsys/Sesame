use cargo_toml::{Manifest, Value};

fn extract_library(v: &Value) -> String {
  if let Value::Table(v) = v {
    let path = v.get("path");
    if !path.is_none() {
      if let Value::String(path) = path.unwrap() {
        return path.clone();
      }
    }
  }
  panic!("workspace.metadata.dylint.libraries has unrecoganizable options");
}

pub fn get_dylinting_libraries(manifest: &Manifest) -> Vec<String> {
  let workspace = manifest.workspace.as_ref();
  if workspace.is_none() {
    return Vec::new();
  }

  let metadata = workspace.unwrap().metadata.as_ref();
  if metadata.is_none() {
    return Vec::new();
  }

  if let Value::Table(metadata) = metadata.unwrap() {
    let dylint = metadata.get("dylint");
    if dylint.is_none() {
      return Vec::new();
    }

    if let Value::Table(dylint) = dylint.unwrap() {
      let dylint_libraries = dylint.get("libraries");
      if dylint_libraries.is_none() {
        return Vec::new();
      }
      
      if let Value::Array(vec) = dylint_libraries.unwrap() {
        return vec.iter().map(extract_library).collect();
      }
    }
  }

  return Vec::new();
}
