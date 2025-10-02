use crate::cache::PackageData;
use crate::error::AppError;
use axum::{
    http::header,
    response::{Html, IntoResponse, Response},
};

/// 返回文件响应
pub fn file_response(package_data: &PackageData, file_path: &str) -> Result<Response, AppError> {
    let file_content = package_data
        .files
        .get(file_path)
        .ok_or_else(|| AppError::NotFound(format!("File '{}' not found", file_path)))?;

    let content_type = get_content_type(file_path);

    Ok(([(header::CONTENT_TYPE, content_type)], file_content.clone()).into_response())
}

/// 返回目录列表
pub fn directory_listing(
    package_data: &PackageData,
    dir_path: &str,
    package_name: &str,
    version: &str,
) -> Result<Response, AppError> {
    let prefix = if dir_path.is_empty() {
        String::new()
    } else {
        format!("{}/", dir_path)
    };

    // 收集目录中的文件
    let mut entries: Vec<(String, bool)> = package_data
        .files
        .keys()
        .filter_map(|path| {
            if path.starts_with(&prefix) {
                let rest = &path[prefix.len()..];
                if !rest.is_empty() {
                    // 检查是否是直接子项
                    if let Some(slash_pos) = rest.find('/') {
                        // 是目录
                        let dir_name = &rest[..slash_pos];
                        Some((dir_name.to_string(), true))
                    } else {
                        // 是文件
                        Some((rest.to_string(), false))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // 去重并排序
    entries.sort();
    entries.dedup();

    // 生成 HTML
    let mut html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Directory listing for {}@{}/{}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 40px;
        }}
        h1 {{
            color: #333;
        }}
        ul {{
            list-style: none;
            padding: 0;
        }}
        li {{
            margin: 5px 0;
        }}
        a {{
            text-decoration: none;
            color: #0066cc;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        .dir {{
            font-weight: bold;
        }}
        .file {{
            color: #555;
        }}
    </style>
</head>
<body>
    <h1>Directory listing for {}@{}/{}</h1>
    <ul>
"#,
        package_name, version, dir_path, package_name, version, dir_path
    );

    // 添加父目录链接
    if !dir_path.is_empty() {
        let parent = if let Some(pos) = dir_path.rfind('/') {
            &dir_path[..pos]
        } else {
            ""
        };
        html.push_str(&format!(
            r#"        <li><a href="/{}@{}/{}/" class="dir">../</a></li>
"#,
            package_name, version, parent
        ));
    }

    for (name, is_dir) in entries {
        let class = if is_dir { "dir" } else { "file" };
        let suffix = if is_dir { "/" } else { "" };
        let link = if dir_path.is_empty() {
            format!("/{}@{}/{}{}", package_name, version, name, suffix)
        } else {
            format!(
                "/{}@{}/{}/{}{}",
                package_name, version, dir_path, name, suffix
            )
        };

        html.push_str(&format!(
            r#"        <li><a href="{}" class="{}">{}{}</a></li>
"#,
            link, class, name, suffix
        ));
    }

    html.push_str(
        r#"    </ul>
</body>
</html>"#,
    );

    Ok(Html(html).into_response())
}

/// 根据文件扩展名获取 Content-Type
fn get_content_type(file_path: &str) -> &'static str {
    let extension = file_path.split('.').next_back().unwrap_or("");

    match extension {
        "js" | "mjs" | "cjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "html" | "htm" => "text/html; charset=utf-8",
        "xml" => "application/xml; charset=utf-8",
        "txt" | "md" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "ts" => "text/typescript; charset=utf-8",
        "tsx" => "text/typescript; charset=utf-8",
        "jsx" => "text/javascript; charset=utf-8",
        "map" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        _ => "application/octet-stream",
    }
}
