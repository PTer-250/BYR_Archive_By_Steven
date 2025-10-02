use crate::cache::{CacheManager, PackageData};
use crate::error::AppError;
use crate::npm;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

/// 解析路径为 (包名, 版本, 文件路径)
pub fn parse_path(path: &str) -> Result<(String, Option<String>, Option<String>), AppError> {
    let path = path.trim_start_matches('/');

    if path.is_empty() {
        return Err(AppError::InvalidRequest("Empty path".to_string()));
    }

    // 检查是否包含 @version
    if let Some(at_pos) = path.find('@') {
        // 处理 scoped package (@scope/package@version/file)
        if at_pos == 0 {
            // scoped package
            if let Some(second_at) = path[1..].find('@') {
                let second_at = second_at + 1;
                let package_name = path[..second_at].to_string();
                let rest = &path[second_at + 1..];

                if let Some(slash_pos) = rest.find('/') {
                    let version = rest[..slash_pos].to_string();
                    let file_path = rest[slash_pos + 1..].to_string();
                    return Ok((package_name, Some(version), Some(file_path)));
                } else {
                    return Ok((package_name, Some(rest.to_string()), None));
                }
            } else {
                // scoped package without version
                if let Some(slash_pos) = path.find('/') {
                    let second_slash = path[slash_pos + 1..].find('/');
                    if let Some(pos) = second_slash {
                        let package_end = slash_pos + 1 + pos;
                        let package_name = path[..package_end].to_string();
                        let file_path = path[package_end + 1..].to_string();
                        return Ok((package_name, None, Some(file_path)));
                    }
                }
                // 只有包名
                return Ok((path.to_string(), None, None));
            }
        } else {
            // 非 scoped package (package@version/file)
            let package_name = path[..at_pos].to_string();
            let rest = &path[at_pos + 1..];

            if let Some(slash_pos) = rest.find('/') {
                let version = rest[..slash_pos].to_string();
                let file_path = rest[slash_pos + 1..].to_string();
                return Ok((package_name, Some(version), Some(file_path)));
            } else {
                return Ok((package_name, Some(rest.to_string()), None));
            }
        }
    }

    // 没有 @version 的情况
    if let Some(slash_pos) = path.find('/') {
        let package_name = path[..slash_pos].to_string();
        let file_path = path[slash_pos + 1..].to_string();
        Ok((package_name, None, Some(file_path)))
    } else {
        Ok((path.to_string(), None, None))
    }
}

/// 获取包文件（带缓存）
pub async fn fetch_package(
    client: &Client,
    _registry: &str,
    package_name: &str,
    version: &str,
    metadata: &Value,
    cache: &CacheManager,
) -> Result<Arc<PackageData>, AppError> {
    let cache_key = format!("package:{}@{}", package_name, version);

    // 检查缓存
    if let Some(cached) = cache.get_package(&cache_key).await {
        tracing::debug!("Package cache hit for {}@{}", package_name, version);
        return Ok(cached);
    }

    // 从元信息中获取 tarball URL
    let tarball_url = metadata
        .get("versions")
        .and_then(|v| v.get(version))
        .and_then(|v| v.get("dist"))
        .and_then(|d| d.get("tarball"))
        .and_then(|t| t.as_str())
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Version {} not found for {}",
                version, package_name
            ))
        })?;

    // 下载并解压
    let files = npm::download_and_extract_tarball(client, tarball_url).await?;

    // 获取 package.json
    let package_json_str = files
        .get("package.json")
        .ok_or_else(|| AppError::InternalError("package.json not found in tarball".to_string()))?;

    let package_json: Value = serde_json::from_slice(package_json_str)?;

    let package_data = PackageData {
        files,
        package_json,
    };

    // 缓存结果
    cache.set_package(cache_key, package_data.clone()).await;

    Ok(Arc::new(package_data))
}

/// 解析入口文件
pub fn resolve_entry_file(package_data: &PackageData) -> Result<String, AppError> {
    let pkg_json = &package_data.package_json;

    // 1. 检查 jsdelivr 字段
    if let Some(jsdelivr) = pkg_json.get("jsdelivr").and_then(|v| v.as_str()) {
        let normalized = jsdelivr.trim_start_matches("./");
        return Ok(normalized.to_string());
    }

    // 2. 检查 exports["."]
    if let Some(exports) = pkg_json.get("exports") {
        if let Some(dot_export) = exports.get(".") {
            // 如果是对象，查找 "default" 字段
            if let Some(obj) = dot_export.as_object() {
                if let Some(default) = obj.get("default").and_then(|v| v.as_str()) {
                    let normalized = default.trim_start_matches("./");
                    return Ok(normalized.to_string());
                }
            }
            // 如果是字符串，直接使用
            if let Some(s) = dot_export.as_str() {
                let normalized = s.trim_start_matches("./");
                return Ok(normalized.to_string());
            }
        }
    }

    // 3. 使用 main 字段
    if let Some(main) = pkg_json.get("main").and_then(|v| v.as_str()) {
        let normalized = main.trim_start_matches("./");
        return Ok(normalized.to_string());
    }

    // 4. 默认尝试 index.js
    if package_data.files.contains_key("index.js") {
        return Ok("index.js".to_string());
    }

    Err(AppError::NotFound(
        "No entry file found in package.json".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        // 简单包名
        let (pkg, ver, file) = parse_path("react").unwrap();
        assert_eq!(pkg, "react");
        assert_eq!(ver, None);
        assert_eq!(file, None);

        // 包名 + 版本
        let (pkg, ver, file) = parse_path("react@18.0.0").unwrap();
        assert_eq!(pkg, "react");
        assert_eq!(ver, Some("18.0.0".to_string()));
        assert_eq!(file, None);

        // 包名 + 版本 + 文件
        let (pkg, ver, file) = parse_path("react@18.0.0/index.js").unwrap();
        assert_eq!(pkg, "react");
        assert_eq!(ver, Some("18.0.0".to_string()));
        assert_eq!(file, Some("index.js".to_string()));

        // scoped 包名
        let (pkg, ver, file) = parse_path("@vue/runtime-core@3.3.4/index.js").unwrap();
        assert_eq!(pkg, "@vue/runtime-core");
        assert_eq!(ver, Some("3.3.4".to_string()));
        assert_eq!(file, Some("index.js".to_string()));
    }
}
