use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
    #[serde(rename = "requires-python")]
    pub requires_python: Option<String>,
}
impl Default for Project {
    fn default() -> Self {
        Self {
            name: "python_project".to_owned(),
            version: "0.1.0".to_owned(),
            dependencies: Default::default(),
            requires_python: Default::default(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Pdm {
    pub distribution: bool,
}
impl Default for Pdm {
    fn default() -> Self {
        Self {
            distribution: false,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tool {
    pub pdm: Pdm,
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PyProject {
    pub project: Project,
    pub tool: Tool,
}
