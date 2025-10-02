use crate::error::AppError;
use serde_json::Value;

/// 解析版本（支持语义化版本）
pub fn resolve_version(metadata: &Value, version_str: Option<&str>) -> Result<String, AppError> {
    match version_str {
        None => {
            // 使用 latest 标签
            metadata
                .get("dist-tags")
                .and_then(|tags| tags.get("latest"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| AppError::NotFound("No latest version found".to_string()))
        }
        Some(v) => {
            // 检查是否是 dist-tags
            if let Some(tags) = metadata.get("dist-tags") {
                if let Some(tag_version) = tags.get(v).and_then(|v| v.as_str()) {
                    return Ok(tag_version.to_string());
                }
            }

            // 检查是否是精确版本
            if metadata
                .get("versions")
                .and_then(|versions| versions.get(v))
                .is_some()
            {
                return Ok(v.to_string());
            }

            // 尝试解析语义化版本范围
            match parse_semver_range(metadata, v) {
                Some(version) => Ok(version),
                None => Err(AppError::NotFound(format!(
                    "No matching version found for '{}'",
                    v
                ))),
            }
        }
    }
}

/// 解析语义化版本范围
fn parse_semver_range(metadata: &Value, range_str: &str) -> Option<String> {
    use node_semver::{Range, Version};

    let versions = metadata.get("versions")?.as_object()?;

    // 解析范围
    let range = Range::parse(range_str).ok()?;

    // 收集所有符合条件的版本
    let mut matching_versions: Vec<Version> = versions
        .keys()
        .filter_map(|v| Version::parse(v).ok())
        .filter(|v| range.satisfies(v))
        .collect();

    // 排序并返回最新版本
    matching_versions.sort();
    matching_versions.last().map(|v| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_version() {
        let metadata = json!({
            "dist-tags": {
                "latest": "1.2.3",
                "next": "2.0.0-beta.1"
            },
            "versions": {
                "1.0.0": {},
                "1.1.0": {},
                "1.2.0": {},
                "1.2.3": {},
                "2.0.0-beta.1": {}
            }
        });

        // 测试 latest
        assert_eq!(resolve_version(&metadata, None).unwrap(), "1.2.3");

        // 测试精确版本
        assert_eq!(resolve_version(&metadata, Some("1.1.0")).unwrap(), "1.1.0");

        // 测试 dist-tag
        assert_eq!(
            resolve_version(&metadata, Some("next")).unwrap(),
            "2.0.0-beta.1"
        );

        // 测试范围
        assert_eq!(resolve_version(&metadata, Some("^1.0.0")).unwrap(), "1.2.3");
    }
}
