use toml::{from_str, Value, value::Table};
use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BhopGroup {
    pub cmd: Option<String>,
    pub editor: Option<String>,
    pub files: Option<Vec<String>>,
}

impl BhopGroup {
    pub fn from_str(group: &str, toml: &str) -> Option<Self> {
        let table: Table = from_str(toml).unwrap_or(Table::new());
        match table.get(group) {
            Some(t) => match t {
                Value::String(cmd) => {
                    Some(Self {
                        cmd: Some(cmd.to_string()),
                        ..Default::default()
                    })
                },
                Value::Table(t) => {
                    let editor = t.get("editor").map(|v| v.as_str().unwrap().to_string());
                    let files = match t.get("files") {
                        Some(f) => Some(f.as_array()?.iter().map(|v| v.as_str().unwrap().to_string()).collect()),
                        None => None,
                    };
                    Some(Self {
                        cmd: None,
                        editor,
                        files,
                    })
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn from<T: AsRef<Path>>(group: &str, toml_path: T) -> Option<Self> {
        let toml = fs::read_to_string(toml_path).ok()?;
        Self::from_str(group, &toml)
    }
}
