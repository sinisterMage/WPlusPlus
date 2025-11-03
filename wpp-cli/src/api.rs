use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub firstName: String,
    pub lastName: String,
        #[serde(default)]
    pub isTeamMember: bool,
        #[serde(default)]
    pub isOSSContributor: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
        #[serde(default)]
    pub user: Option<UserInfo>,
}

#[derive(Deserialize, Debug)]
pub struct PackageFile {
    pub id: String,
    pub filename: String,
    pub fileType: Option<String>,
    pub platform: Option<String>,
    pub architecture: Option<String>,
    pub fileSize: Option<u64>,
    pub checksum: Option<String>,
    pub downloadUrl: String,
}

#[derive(Deserialize, Debug)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub readme: Option<String>,
    pub files: Vec<PackageFile>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub license: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub readme: Option<String>,
    pub is_public: Option<bool>,
}

/// File information with metadata for publishing
#[derive(Debug, Clone)]
pub struct FileWithMetadata {
    pub path: String,
    pub file_type: Option<String>,
    pub platform: Option<String>,
    pub architecture: Option<String>,
}

pub struct IngotAPI {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl IngotAPI {
   pub fn auth_header(&self) -> Option<(String, String)> {
        self.token
            .as_ref()
            .map(|t| ("Authorization".to_string(), format!("Bearer {}", t.trim())))
    }

    pub fn new(base_url: &str, token: Option<String>) -> Self {
        // Automatically load from ~/.wpp_token if not passed via env
        let token = token.or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            let token_path = format!("{}/.wpp_token", home);
            std::fs::read_to_string(&token_path).ok().map(|s| s.trim().to_string())
        });

        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            token,
        }
    }

    /// ✅ Verify login token
    pub async fn verify_login(&self) -> Result<LoginResponse> {
        let url = format!("{}/api/cli/login", self.base_url);
        let mut req = self.client.get(&url);
        if let Some((k, v)) = self.auth_header() {
            req = req.header(k, v);
        }

        let res = req.send().await?;
        if res.status() == StatusCode::UNAUTHORIZED {
            return Err(anyhow!("Invalid or expired API token"));
        }

        Ok(res.json::<LoginResponse>().await?)
    }

    /// 📦 Install package by name/version
    pub async fn get_package(&self, name: &str, version: &str) -> Result<PackageInfo> {
    let url = format!("{}/api/cli/packages/{}/{}", self.base_url, name, version);
    let mut req = self.client.get(&url);
    if let Some((k, v)) = self.auth_header() {
        req = req.header(k, v);
    }

    let res = req.send().await?;
    let status = res.status();

    match status {
        StatusCode::OK => Ok(res.json::<PackageInfo>().await?),
        StatusCode::NOT_FOUND => Err(anyhow!("❌ Package not found: {}/{}", name, version)),
        StatusCode::FORBIDDEN => Err(anyhow!("🔒 Private package — authentication required")),
        _ => {
            let body = res.text().await.unwrap_or_default();
            Err(anyhow!("Unexpected response {}: {}", status, body))
        }
    }
}


    /// 🚀 Publish a new package
    pub async fn publish_package(&self, metadata: &PublishMetadata, files: Vec<FileWithMetadata>) -> Result<()> {
        let url = format!("{}/api/cli/publish", self.base_url);
        let mut form = reqwest::multipart::Form::new()
            .text("metadata", serde_json::to_string(metadata)?);

        for file_info in files {
            let filename = file_info.path.clone();
            let bytes = fs::read(&file_info.path)?;
            let mut part = reqwest::multipart::Part::bytes(bytes).file_name(filename);

            // Add file metadata as headers
            if let Some(file_type) = &file_info.file_type {
                part = part.mime_str(&format!("application/x-wpp-{}", file_type))?;
            }

            // Note: The API backend should extract platform/arch from multipart metadata
            // For now, we'll pass it through the filename or rely on server-side detection
            form = form.part("files[]", part);

            // Optionally include metadata as separate form field (if API supports it)
            if file_info.file_type.is_some() || file_info.platform.is_some() {
                let file_meta = serde_json::json!({
                    "filename": std::path::Path::new(&file_info.path).file_name()
                        .and_then(|n| n.to_str()).unwrap_or(""),
                    "fileType": file_info.file_type,
                    "platform": file_info.platform,
                    "architecture": file_info.architecture,
                });
                form = form.text(format!("file_metadata[{}]", file_info.path), file_meta.to_string());
            }
        }

        let mut req = self.client.post(&url).multipart(form);
        if let Some((k, v)) = self.auth_header() {
            req = req.header(k, v);
        }
        let response = req.send().await?;
let status = response.status();
let text = response.text().await.unwrap_or_default();


match status {
    StatusCode::OK | StatusCode::CREATED => {
        println!("✅ Package published successfully!");
        Ok(())
    }
    StatusCode::UNAUTHORIZED => Err(anyhow!("Unauthorized — check your API token")),
    StatusCode::BAD_REQUEST => Err(anyhow!("Validation error or version already exists")),
    _ => Err(anyhow!(format!(
        "Unexpected response while publishing (status: {}, body: {})",
        status, text
    ))),
}
    }
}
impl Default for PublishMetadata {
    fn default() -> Self {
        Self {
            name: "unknown".into(),
            version: "0.1.0".into(),
            description: Some("Published via W++ Ingot CLI".into()),
            license: Some("MIT".into()),
            category: Some("utilities".into()),
            tags: Some(vec![]),
            readme: Some("# No README provided".into()),
            is_public: Some(true),
        }
    }
}