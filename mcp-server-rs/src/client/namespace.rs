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
        let text = super::read_checked(r).await?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    /// Full node definition. Answered as XML and decoded, because the JSON
    /// encoder IS uses for `/invoke` flattens a flow's step tree into the
    /// string `"[INVOKE]"`; the XML encoder keeps the nested `nodes`. Falls
    /// back to the JSON answer if the XML cannot be decoded, so a node is
    /// never unreadable because of the decoder.
    pub async fn node_get(&self, name: &str) -> Result<Value, String> {
        match self.node_get_xml(name).await {
            Ok(v) => Ok(v),
            Err(xml_err) => {
                let mut v = self.node_get_json(name).await?;
                if let Some(o) = v.as_object_mut() {
                    o.insert(
                        "warning".into(),
                        json!(format!(
                            "XML decode failed ({xml_err}); JSON fallback flattens flow.nodes into a string"
                        )),
                    );
                }
                Ok(v)
            }
        }
    }

    /// `getNode` decoded from `IDataXMLCoder` XML (complete flow tree).
    pub(crate) async fn node_get_xml(&self, name: &str) -> Result<Value, String> {
        self.invoke_get_xml("wm.server.ns/getNode", &[("name", name)])
            .await
    }

    /// `getNode` as IS's JSON (cheap existence check: `node` is null when
    /// the node does not exist; flow steps are NOT usable in this form).
    pub(crate) async fn node_get_json(&self, name: &str) -> Result<Value, String> {
        let r = self
            .client
            .get(self.url("/invoke/wm.server.ns/getNode"))
            .query(&[("name", name)])
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let text = super::read_checked(r).await?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    /// `true` when a node of that name is currently loaded.
    pub(crate) async fn node_exists(&self, name: &str) -> Result<bool, String> {
        let v = self.node_get_json(name).await?;
        Ok(v.get("node").is_some_and(|n| !n.is_null()))
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

    /// Create one folder level. Idempotent: an existing folder answers
    /// `status: "exists"` instead of `[ISS.0085.9082] folder node ... exists
    /// in package`, so a provisioning script can be re-run.
    pub async fn folder_create(&self, package: &str, folder_path: &str) -> Result<Value, String> {
        match self.make_node("interface", package, folder_path).await {
            Ok(_) => Ok(json!({"status": "created", "folder": folder_path, "package": package})),
            Err(e) if super::services::is_already_exists(&e) => {
                Ok(json!({"status": "exists", "folder": folder_path, "package": package}))
            }
            Err(e) => Err(e),
        }
    }
}
