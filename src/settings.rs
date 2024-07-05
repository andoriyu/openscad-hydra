use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;

use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
    script_path: String,
    module_name: String,
    params: BTreeMap<String, Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompiledProfile {
    name: String,
    script_path: String,
    module_name: String,
    param_keys: Vec<String>,
    params: Vec<Value>,
}

impl Settings {
    pub fn compile_profiles(self) -> HashMap<String, HashSet<CompiledProfile>> {
        self.profiles
            .into_iter()
            .map(|(profile_name, profile)| {
                let keys: Vec<_> = profile.params.keys().cloned().collect();
                let profiles = profile
                    .params
                    .values()
                    .cloned()
                    .map(IntoIterator::into_iter)
                    .multi_cartesian_product()
                    .into_iter()
                    .map(|params| CompiledProfile {
                        name: profile_name.clone(),
                        param_keys: keys.clone(),
                        script_path: profile.script_path.clone(),
                        module_name: profile.module_name.clone(),
                        params,
                    })
                    .collect();
                (profile_name, profiles)
            })
            .collect()
    }
}

impl CompiledProfile {
    pub fn write_script<W: Write>(&self, out: &mut W) -> anyhow::Result<()> {
        writeln!(out, "include <{}>", &self.script_path)?;

        let params_string = self
            .param_keys
            .iter()
            .zip(self.params.iter())
            .map(|(key, value)| {
                let value = value.to_string();
                format!("{key}={value}")
            })
            .join(", ");
        write!(out, "{}({});", &self.module_name, params_string)?;
        Ok(())
    }

    pub fn output_filename(&self) -> PathBuf {
        let base = self.params.iter().join("_");
        let path = {
            let mut p = PathBuf::new();
            p.push(base);
            p.set_extension("3mf");
            p
        };
        path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn params(&self) -> &[Value] {
        self.params.as_slice()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use std::str::from_utf8;

    use serde_json::json;

    use super::*;

    #[test]
    fn it_parses() {
        let input = r#"
{
  "profiles": {
    "metric_bolts": {
      "script_path": "./label.scad",
      "module_name": "drawLabel",
      "params": {
        "width": [2, 2.5, 3, 4],
        "length": [8, 12, 16, 20],
        "head_type": ["hex", "philips"],
        "cs": [true, false]
      }
    }
  }
}
        "#;

        let _settings: Settings = serde_json::from_str(&input).unwrap();
    }

    #[test]
    fn it_compiles_options() {
        let input = r#"
{
  "profiles": {
    "metric_bolts": {
      "script_path": "./label.scad",
      "module_name": "drawLabel",
      "params": {
        "width": [2],
        "length": [8, 12],
        "head_type": ["hex", "philips"]
      }
    }
  }
}
        "#;
        let expected: HashSet<Value> = HashSet::from([
            json!([2, 8, "hex"]),
            json!([2, 8, "philips"]),
            json!([2, 12, "hex"]),
            json!([2, 12, "philips"]),
        ]);

        let expected: HashSet<_> = expected
            .into_iter()
            .map(|params| CompiledProfile {
                name: String::from("metric_bolts"),
                param_keys: vec![
                    String::from("width"),
                    String::from("length"),
                    String::from("head_type"),
                ],
                params: params.as_array().cloned().unwrap(),
                module_name: String::from("drawLabel"),
                script_path: String::from("./label.scad"),
            })
            .collect();

        let settings: Settings = serde_json::from_str(&input).unwrap();

        let profiles = settings.compile_profiles();
        let metric_bolts = &profiles["metric_bolts"];

        assert_eq!(expected.len(), metric_bolts.len());

        for profile in profiles.values().flatten() {
            assert_eq!("metric_bolts", profile.name);
            assert_eq!("./label.scad", profile.script_path);
            assert_eq!("drawLabel", profile.module_name);
        }
    }

    #[test]
    fn it_writes_openscad_template() {
        let profile = CompiledProfile {
            name: String::from("metric_bolts"),
            param_keys: vec![
                String::from("width"),
                String::from("length"),
                String::from("head_type"),
                String::from("cs"),
            ],
            params: json!([2, 8, "hex", true]).as_array().cloned().unwrap(),
            module_name: String::from("drawLabel"),
            script_path: String::from("./label.scad"),
        };
        let mut buf = Vec::new();
        let excted_str = r#"include <./label.scad>
drawLabel(width=2, length=8, head_type="hex", cs=true);"#;

        let _ = profile.write_script(&mut buf).unwrap();

        let actual = from_utf8(buf.as_slice()).unwrap();
        assert_eq!(excted_str, actual);
    }
}
