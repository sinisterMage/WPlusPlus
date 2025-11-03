use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::{anyhow, Result};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use tokio::runtime::Runtime;

use crate::api::IngotAPI;
use crate::config::WppConfig;


/// Global cache for resolved dependencies
lazy_static::lazy_static! {
    static ref RESOLVED: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

/// Represents a resolved dependency (used for tree display & caching)
#[derive(Clone, Debug)]
pub struct ModuleRecord {
    pub name: String,
    pub version: String,
    pub checksum: String,
}

/// Core WMS dependency resolver
pub struct WmsResolver {
    pub api: Arc<IngotAPI>,
}

impl WmsResolver {
    pub fn new(api: Arc<IngotAPI>) -> Self {
        Self { api }
    }

    /// Entry point: resolve dependencies recursively and in parallel
    pub fn resolve_all(&self, project_root: &Path) -> Result<Vec<ModuleRecord>> {
        let start = Instant::now();

        let config_path = project_root.join("wpp.config.hs");
        let cfg = WppConfig::load(&config_path)
            .map_err(|e| anyhow!("Failed to parse config: {e}"))?;

        let deps = cfg.flags.iter()
            .filter(|f| f.starts_with("--dep="))
            .map(|f| f.trim_start_matches("--dep=").to_string())
            .collect::<Vec<_>>();

        if deps.is_empty() {
            println!("🟢 No dependencies found in wpp.config.hs");
            return Ok(vec![]);
        }

        println!("📦 Found {} dependencies", deps.len());

        let runtime = Runtime::new()?;
        let results = runtime.block_on(self.download_all_parallel(deps))?;

        println!(
            "✅ All dependencies resolved in {:.2?}",
            start.elapsed()
        );
        Ok(results)
    }

    /// Concurrent download using rayon threads
    async fn download_all_parallel(&self, deps: Vec<String>) -> Result<Vec<ModuleRecord>> {
    use std::sync::Arc;
    use std::sync::Mutex;

    let records = Arc::new(Mutex::new(Vec::new()));

    // Use regular iterator instead of parallel to avoid Rayon/Tokio deadlock
    deps.iter().for_each(|dep| {
        let api = self.api.clone();
        let dep_name = dep.clone();
        let records = Arc::clone(&records);

        // Skip if already resolved
        if RESOLVED.lock().unwrap().contains(&dep_name) {
            return;
        }

        let res: Result<ModuleRecord> = tokio::task::block_in_place(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match api.get_package(&dep_name, "latest").await {
                    Ok(pkg) => {
                        println!("⬇️  Resolving {} v{}", pkg.name, pkg.version);

                        for f in pkg.files {
                            let resp = reqwest::get(&f.downloadUrl).await?;
                            if resp.status().is_success() {
                                let bytes = resp.bytes().await?;
                                let install_dir = Path::new(".wpp_packages").join(&pkg.name);
                                fs::create_dir_all(&install_dir)?;
                                let dest = install_dir.join(&f.filename);
                                fs::write(&dest, &bytes)?;
                                println!("✅ Saved {}", dest.display());
                            }
                        }

                        let mut hasher = Sha256::new();
                        hasher.update(pkg.name.as_bytes());
                        let checksum = format!("sha256:{:x}", hasher.finalize());

                        crate::update_simula_lock(&pkg.name, &pkg.version, &checksum)
                            .unwrap_or_else(|e| eprintln!("⚠️ Lock update failed: {e}"));

                        RESOLVED.lock().unwrap().insert(pkg.name.clone());
                        Ok(ModuleRecord {
                            name: pkg.name,
                            version: pkg.version,
                            checksum,
                        })
                    }
                    Err(e) => Err(anyhow!("❌ Failed to resolve {}: {}", dep_name, e)),
                }
            })
        });

        match res {
            Ok(record) => {
                records.lock().unwrap().push(record);
            }
            Err(e) => eprintln!("⚠️ Dependency failed: {e}"),
        }
    });

    // Collect final results
    let records = Arc::try_unwrap(records)
        .unwrap()
        .into_inner()
        .unwrap();

    Ok(records)
}

}
