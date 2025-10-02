use crate::cache::CacheManager;
use crate::error::AppError;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

/// 获取包的元信息
pub async fn fetch_package_metadata(
    client: &Client,
    registry: &str,
    package_name: &str,
    cache: &CacheManager,
) -> Result<Arc<Value>, AppError> {
    let cache_key = format!("metadata:{}", package_name);

    // 检查缓存
    if let Some(cached) = cache.get_metadata(&cache_key).await {
        tracing::debug!("Metadata cache hit for {}", package_name);
        return Ok(cached);
    }

    // 从 registry 获取
    tracing::debug!("Fetching metadata for {} from {}", package_name, registry);

    let url = format!("{}/{}", registry, package_name);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(AppError::NotFound(format!(
            "Package '{}' not found",
            package_name
        )));
    }

    let metadata: Value = response.json().await?;

    // 缓存结果
    cache.set_metadata(cache_key, metadata.clone()).await;

    Ok(Arc::new(metadata))
}

/// 下载并解析 tarball
pub async fn download_and_extract_tarball(
    client: &Client,
    tarball_url: &str,
) -> Result<std::collections::HashMap<String, Vec<u8>>, AppError> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    tracing::debug!("Downloading tarball from {}", tarball_url);

    let response = client.get(tarball_url).send().await?;
    let bytes = response.bytes().await?;

    tracing::debug!("Extracting tarball ({} bytes)", bytes.len());

    let tar = GzDecoder::new(&bytes[..]);
    let mut archive = Archive::new(tar);

    let mut files = std::collections::HashMap::new();

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let is_file = entry.header().entry_type().is_file();

        // npm tarball 中的文件都在 "package/" 目录下
        let path_str = path.to_string_lossy();
        if let Some(stripped) = path_str.strip_prefix("package/") {
            if is_file {
                let mut contents = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut contents)?;
                files.insert(stripped.to_string(), contents);
                tracing::trace!("Extracted file: {}", stripped);
            }
        }
    }

    tracing::debug!("Extracted {} files", files.len());

    Ok(files)
}
