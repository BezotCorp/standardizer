use crate::configuration::DeletionPolicy;
use serde_json::Value;
use std::{error::Error, fs, io::ErrorKind, path::Path};

#[derive(Debug, Default)]
pub(in crate::configuration) struct Settings {
    deletion: Option<DeletionPolicy>,
}

impl Settings {
    pub(in crate::configuration) fn load(path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(Self::default());
            }
            Err(error) => return Err(error.into()),
        };

        Self::parse(&content).map_err(|error| format!("{}: {error}", path.display()).into())
    }

    fn parse(content: &str) -> Result<Self, Box<dyn Error>> {
        let value: Value = serde_json::from_str(content)?;

        let object = value
            .as_object()
            .ok_or("configuration must be a JSON object")?;

        for key in object.keys() {
            if key != "deletion" {
                return Err(format!("unknown configuration property '{key}'").into());
            }
        }

        let deletion = match object.get("deletion") {
            None => None,
            Some(Value::String(value)) if value == "ignore" => Some(DeletionPolicy::Ignore),
            Some(value) => {
                return Err(
                    format!("invalid value {value} for 'deletion': expected \"ignore\"").into(),
                );
            }
        };

        Ok(Self { deletion })
    }

    pub(in crate::configuration) fn merge(self, fallback: Self) -> Self {
        Self {
            deletion: self.deletion.or(fallback.deletion),
        }
    }

    pub(in crate::configuration) fn deletion(&self) -> DeletionPolicy {
        self.deletion.unwrap_or_default()
    }
}
