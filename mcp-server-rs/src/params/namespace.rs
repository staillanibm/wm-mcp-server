use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NodeListParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(
        description = "Optional folder path (e.g., \"services.utils\"). Empty = package root."
    )]
    pub folder: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NodeNameParam {
    #[schemars(description = "Full namespace path (e.g., \"claudedemo.services:helloWorld\")")]
    pub name: String,
    #[schemars(
        description = "Package name owning the node. Required by node_delete (the IS resolves the package to take the lock; omitting it fails with a NullPointerException). Ignored by node_get."
    )]
    pub package: Option<String>,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FolderCreateParam {
    #[schemars(description = "Package name")]
    pub package: String,
    #[schemars(description = "Dot-separated path (e.g., \"services\" or \"services.utils\")")]
    pub folder_path: String,
    #[schemars(description = "Target IS instance name (omit for default)")]
    pub instance: Option<String>,
}
