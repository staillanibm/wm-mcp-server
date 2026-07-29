use serde_json::{Value, json};

impl super::ISClient {
    // ── Namespace / Node Management ────────────────────────────────────

    pub async fn node_list(&self, package: &str, interface: &str) -> Result<Value, String> {
        let mut params = vec![("package", package)];
        if !interface.is_empty() {
            params.push(("interface", interface));
        }
        let r = self
            .client
            .get(self.url("/invoke/wm.server.ns/getNodeList"))
            .query(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn node_get(&self, name: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.ns/getNode"))
            .query(&[("name", name)])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        r.error_for_status_ref().map_err(|e| e.to_string())?;
        r.json().await.map_err(|e| e.to_string())
    }

    pub async fn node_delete(&self, name: &str, package: Option<&str>) -> Result<Value, String> {
        // node_pkg is mandatory: wm.server.nsimpl.deleteNode resolves the package to
        // take the lock, and throws a NullPointerException when it is missing.
        let mut body = json!({"node_nsName": name});
        if let Some(pkg) = package {
            body["node_pkg"] = json!(pkg);
        }
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/deleteNode"))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        // The IS answers HTTP 200 with status "true"/"false": a refused delete must
        // not be reported as a success, so surface its actual response.
        let resp: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if resp.get("status").and_then(|s| s.as_str()) == Some("false") {
            return Err(format!("IS refused the delete: {text}"));
        }
        Ok(resp)
    }

    pub async fn folder_create(&self, package: &str, folder_path: &str) -> Result<Value, String> {
        let r = self
            .client
            .post(self.url("/invoke/wm.server.ns/makeNode"))
            .json(&json!({
                "node_type": "interface",
                "node_nsName": folder_path,
                "node_pkg": package,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        super::read_checked(r).await?;
        Ok(json!({"status": "created", "folder": folder_path, "package": package}))
    }
}
