use serde_json::{json, Value};

pub const REGISTRY_SOURCE: &str = include_str!("../../tp08-asset-registry/asset-registry-v1.json");
pub const PUBLIC_SOURCE: &str = include_str!("../../tp08-asset-registry/public-manifest-v1.json");
pub const AUTH_SOURCE: &str =
    include_str!("../../tp08-asset-registry/item-authorization-evidence-v1.json");

#[derive(Clone, Debug)]
pub struct Catalog {
    pub registry: Value,
    pub public_manifest: Value,
    pub authorization: Value,
}

impl Catalog {
    pub fn read_only() -> Self {
        Self {
            registry: serde_json::from_str(REGISTRY_SOURCE).expect("registry JSON"),
            public_manifest: serde_json::from_str(PUBLIC_SOURCE).expect("public JSON"),
            authorization: serde_json::from_str(AUTH_SOURCE).expect("authorization JSON"),
        }
    }

    pub fn entries(&self) -> Vec<&Value> {
        self.registry["entries"]
            .as_array()
            .map(|items| items.iter().collect())
            .unwrap_or_default()
    }

    pub fn find(&self, asset_id: &str) -> Option<&Value> {
        self.entries()
            .into_iter()
            .find(|entry| entry["assetId"] == asset_id)
    }

    pub fn public_contains(&self, asset_id: &str) -> bool {
        self.public_manifest["entries"]
            .as_array()
            .map(|items| items.iter().any(|entry| entry["assetId"] == asset_id))
            .unwrap_or(false)
    }

    pub fn summary(&self) -> Value {
        let entries = self.entries();
        let pending = entries
            .iter()
            .filter(|entry| entry["policyState"] == "pending_review")
            .count();
        let dev_only = entries
            .iter()
            .filter(|entry| entry["devOnly"] == true)
            .count();
        let required = entries
            .iter()
            .filter(|entry| entry["required"] == true)
            .count();
        json!({
            "registryEntries": entries.len(),
            "publicEntries": self.public_manifest["entries"].as_array().map(|v| v.len()).unwrap_or(0),
            "authorizationEntries": self.authorization["entries"].as_array().map(|v| v.len()).unwrap_or(0),
            "pendingReview": pending,
            "devOnly": dev_only,
            "required": required,
            "publicOfficialPublishEligible": self.public_manifest["officialPublishEligible"]
        })
    }
}

pub fn resolve(catalog: &Catalog, asset_id: &str, bundle: &str, mode: &str) -> Value {
    if asset_id.is_empty() {
        return json!({ "ok": false, "error": "E_ASSET_NOT_FOUND", "bytesLoaded": 0, "assetReadAttempts": 0 });
    }
    if bundle.is_empty() {
        return json!({ "ok": false, "error": "E_BUNDLE_REQUIRED", "bytesLoaded": 0, "assetReadAttempts": 0 });
    }
    let entry = match catalog.find(asset_id) {
        Some(entry) => entry,
        None => {
            return json!({ "ok": false, "error": "E_ASSET_NOT_FOUND", "bytesLoaded": 0, "assetReadAttempts": 0 })
        }
    };
    let policy = entry["policyState"].as_str().unwrap_or("unknown");
    match mode {
        "public" => {
            if entry["required"] == true && !catalog.public_contains(asset_id) {
                return json!({ "ok": false, "error": "E_REQUIRED_PUBLIC_UNRESOLVED", "authorizationState": policy, "publicManifest": false, "bytesLoaded": 0, "assetReadAttempts": 0, "blockedBeforeRead": true });
            }
            if !catalog.public_contains(asset_id) {
                return json!({ "ok": false, "error": "E_PUBLIC_NOT_PUBLISHED", "authorizationState": policy, "publicManifest": false, "bytesLoaded": 0, "assetReadAttempts": 0, "blockedBeforeRead": true });
            }
            json!({ "ok": true, "error": Value::Null, "authorizationState": policy, "bytesLoaded": 0, "assetReadAttempts": 0, "metadataOnly": true })
        }
        "development" => {
            if policy == "rejected" {
                json!({ "ok": false, "error": "E_DEVELOPMENT_REJECTED", "authorizationState": policy, "bytesLoaded": 0, "assetReadAttempts": 0, "blockedBeforeRead": true })
            } else {
                json!({ "ok": true, "error": Value::Null, "authorizationState": policy, "developmentOnly": policy != "verified_original" && policy != "verified_redistributable", "metadataOnly": true, "bytesLoaded": 0, "assetReadAttempts": 0 })
            }
        }
        "offline" => {
            json!({ "ok": true, "error": Value::Null, "authorizationState": "system_fallback", "fallback": "text_only", "metadataOnly": true, "bytesLoaded": 0, "assetReadAttempts": 0 })
        }
        _ => {
            json!({ "ok": false, "error": "E_MODE_UNSUPPORTED", "bytesLoaded": 0, "assetReadAttempts": 0 })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_read_only_and_public_is_empty() {
        let catalog = Catalog::read_only();
        assert_eq!(catalog.summary()["registryEntries"], 35);
        assert_eq!(catalog.summary()["publicEntries"], 0);
        assert_eq!(catalog.summary()["publicOfficialPublishEligible"], false);
    }

    #[test]
    fn required_public_is_blocked_before_read() {
        let catalog = Catalog::read_only();
        let result = resolve(
            &catalog,
            "wuxian.vendor.shared.babylonjs.current",
            "grey-hive",
            "public",
        );
        assert_eq!(result["error"], "E_REQUIRED_PUBLIC_UNRESOLVED");
        assert_eq!(result["blockedBeforeRead"], true);
        assert_eq!(result["bytesLoaded"], 0);
        assert_eq!(result["assetReadAttempts"], 0);
    }

    #[test]
    fn optional_public_is_not_published() {
        let catalog = Catalog::read_only();
        let result = resolve(
            &catalog,
            "wuxian.image.shared.cenyao_portrait.wan_v1",
            "grey-hive",
            "public",
        );
        assert_eq!(result["error"], "E_PUBLIC_NOT_PUBLISHED");
        assert_eq!(result["bytesLoaded"], 0);
    }

    #[test]
    fn development_pending_is_metadata_only() {
        let catalog = Catalog::read_only();
        let result = resolve(
            &catalog,
            "wuxian.vendor.shared.babylonjs.current",
            "grey-hive-dev",
            "development",
        );
        assert_eq!(result["ok"], true);
        assert_eq!(result["developmentOnly"], true);
        assert_eq!(result["metadataOnly"], true);
        assert_eq!(result["bytesLoaded"], 0);
    }

    #[test]
    fn stable_unknown_and_mode_errors() {
        let catalog = Catalog::read_only();
        assert_eq!(
            resolve(&catalog, "unknown.asset", "grey-hive", "development")["error"],
            "E_ASSET_NOT_FOUND"
        );
        assert_eq!(
            resolve(
                &catalog,
                "wuxian.vendor.shared.babylonjs.current",
                "grey-hive",
                "wat"
            )["error"],
            "E_MODE_UNSUPPORTED"
        );
    }
}
