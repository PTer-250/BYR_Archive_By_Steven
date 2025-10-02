use moka::future::Cache;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct CacheManager {
    // 元信息缓存 (5分钟)
    metadata_cache: Cache<String, Arc<Value>>,
    // 包文件缓存 (1小时)
    package_cache: Cache<String, Arc<PackageData>>,
}

#[derive(Clone)]
pub struct PackageData {
    pub files: HashMap<String, Vec<u8>>,
    pub package_json: Value,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            metadata_cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(300)) // 5 minutes
                .build(),
            package_cache: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(3600)) // 1 hour
                .build(),
        }
    }

    pub async fn get_metadata(&self, key: &str) -> Option<Arc<Value>> {
        self.metadata_cache.get(key).await
    }

    pub async fn set_metadata(&self, key: String, value: Value) {
        self.metadata_cache.insert(key, Arc::new(value)).await;
    }

    pub async fn get_package(&self, key: &str) -> Option<Arc<PackageData>> {
        self.package_cache.get(key).await
    }

    pub async fn set_package(&self, key: String, value: PackageData) {
        self.package_cache.insert(key, Arc::new(value)).await;
    }
}
