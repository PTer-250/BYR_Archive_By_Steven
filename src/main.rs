use axum::{
    extract::{Path, State},
    response::{Html, Response},
    routing::get,
    Router,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod error;
mod npm;
mod package;
mod response;
mod semver_utils;

use cache::CacheManager;
use error::AppError;

#[derive(Clone)]
struct AppState {
    cache: Arc<CacheManager>,
    registry: String,
    http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "byr_jsdelivr=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 读取环境变量
    let registry =
        std::env::var("REGISTRY").unwrap_or_else(|_| "https://registry.npmjs.org".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    tracing::info!("Using npm registry: {}", registry);

    // 初始化应用状态
    let cache = Arc::new(CacheManager::new());
    let http_client = reqwest::Client::builder()
        .user_agent("byr-jsdelivr/0.1.0")
        .build()
        .expect("Failed to create HTTP client");

    let state = AppState {
        cache,
        registry,
        http_client,
    };

    // 构建路由
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/*path", get(package_handler))
        .with_state(state);

    // 启动服务器
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

async fn root_handler() -> Html<&'static str> {
    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>BYR jsDelivr Service</title>
        </head>
        <body>
            <h1>BYR jsDelivr Service</h1>
            <p>A minimal jsDelivr-like CDN service for npm packages.</p>
            <h2>Usage:</h2>
            <ul>
                <li><code>/package</code> - Get the entry file of the latest version</li>
                <li><code>/package@version</code> - Get the entry file of a specific version</li>
                <li><code>/package@version/</code> - List directory contents</li>
                <li><code>/package@version/path/to/file.js</code> - Get a specific file</li>
            </ul>
            <h2>Examples:</h2>
            <ul>
                <li><a href="/react">/react</a></li>
                <li><a href="/vue@3.3.4/">/vue@3.3.4/</a></li>
                <li><a href="/lodash@4.17.21/lodash.js">/lodash@4.17.21/lodash.js</a></li>
            </ul>
        </body>
        </html>
        "#,
    )
}

async fn package_handler(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    tracing::debug!("Handling request for path: {}", path);

    // 解析路径
    let (package_name, version_str, file_path) = package::parse_path(&path)?;

    tracing::debug!(
        "Parsed: package={}, version={:?}, file={:?}",
        package_name,
        version_str,
        file_path
    );

    // 获取包的元信息
    let metadata = npm::fetch_package_metadata(
        &state.http_client,
        &state.registry,
        &package_name,
        &state.cache,
    )
    .await?;

    // 解析版本
    let version = semver_utils::resolve_version(&metadata, version_str.as_deref())?;

    tracing::debug!("Resolved version: {}", version);

    // 获取包文件
    let package_data = package::fetch_package(
        &state.http_client,
        &state.registry,
        &package_name,
        &version,
        &metadata,
        &state.cache,
    )
    .await?;

    // 根据请求类型返回不同内容
    match file_path {
        None => {
            // 返回入口文件
            let entry_file = package::resolve_entry_file(&package_data)?;
            response::file_response(&package_data, &entry_file)
        }
        Some(ref p) if p.ends_with('/') || p.is_empty() => {
            // 返回目录列表
            let dir_path = p.trim_end_matches('/');
            response::directory_listing(&package_data, dir_path, &package_name, &version)
        }
        Some(ref p) => {
            // 返回指定文件
            response::file_response(&package_data, p)
        }
    }
}
