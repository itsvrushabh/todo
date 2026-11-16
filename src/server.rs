use crate::models::{TodoCreate, TodoItem, TodoStats, TodoUpdate};
use crate::storage::JsonTodoStorage;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, patch},
    Json, Router,
};
use mime_guess::from_path;
use qrcode::render::unicode;
use qrcode::QrCode;
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Assets;

#[derive(Debug, Deserialize, Default)]
pub struct TodoQueryParams {
    pub status: Option<String>,
    pub search: Option<String>,
    pub priority: Option<String>,
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub storage: Arc<JsonTodoStorage>,
}

pub fn create_router(storage: Arc<JsonTodoStorage>) -> Router {
    Router::new()
        // API routes
        .route("/api/health", get(health_check))
        .route("/api/todos", get(get_todos).post(create_todo))
        .route(
            "/api/todos/{id}",
            get(get_todo).put(update_todo).delete(delete_todo),
        )
        .route("/api/todos/{id}/toggle", patch(toggle_todo))
        .route("/api/todos-completed/clear", delete(clear_completed))
        .route("/api/stats", get(get_stats))
        // Web & PWA routes
        .route("/", get(serve_index))
        .route("/manifest.json", get(serve_manifest))
        .route("/sw.js", get(serve_sw))
        .route("/favicon.ico", get(serve_favicon))
        .route("/style.css", get(serve_style))
        .route("/app.js", get(serve_script))
        .route("/icons/{*path}", get(serve_icons))
        .route("/static/{*path}", get(serve_static))
        .layer(CorsLayer::permissive())
        .with_state(storage)
}

// API Handlers
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

async fn get_todos(
    State(storage): State<Arc<JsonTodoStorage>>,
    Query(params): Query<TodoQueryParams>,
) -> Result<Json<Vec<TodoItem>>, (StatusCode, Json<serde_json::Value>)> {
    let items = storage
        .get_all(
            params.status.as_deref(),
            params.search.as_deref(),
            params.priority.as_deref(),
        )
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        })?;
    Ok(Json(items))
}

async fn create_todo(
    State(storage): State<Arc<JsonTodoStorage>>,
    Json(payload): Json<TodoCreate>,
) -> Result<(StatusCode, Json<TodoItem>), (StatusCode, Json<serde_json::Value>)> {
    if payload.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "detail": "Title cannot be empty or whitespace only" })),
        ));
    }

    let created = storage.create(payload).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(created)))
}

async fn get_todo(
    State(storage): State<Arc<JsonTodoStorage>>,
    Path(id): Path<i64>,
) -> Result<Json<TodoItem>, (StatusCode, Json<serde_json::Value>)> {
    match storage.get_by_id(id) {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Todo #{id} not found") })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

async fn update_todo(
    State(storage): State<Arc<JsonTodoStorage>>,
    Path(id): Path<i64>,
    Json(payload): Json<TodoUpdate>,
) -> Result<Json<TodoItem>, (StatusCode, Json<serde_json::Value>)> {
    match storage.update(id, payload) {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Todo #{id} not found") })),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "detail": e.to_string() })),
        )),
    }
}

async fn toggle_todo(
    State(storage): State<Arc<JsonTodoStorage>>,
    Path(id): Path<i64>,
) -> Result<Json<TodoItem>, (StatusCode, Json<serde_json::Value>)> {
    match storage.toggle(id) {
        Ok(Some(item)) => Ok(Json(item)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Todo #{id} not found") })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

async fn delete_todo(
    State(storage): State<Arc<JsonTodoStorage>>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match storage.delete(id) {
        Ok(true) => Ok(Json(
            json!({ "success": true, "message": format!("Todo #{id} deleted") }),
        )),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "detail": format!("Todo #{id} not found") })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )),
    }
}

async fn clear_completed(
    State(storage): State<Arc<JsonTodoStorage>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let count = storage.clear_completed().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;
    Ok(Json(json!({ "success": true, "cleared_count": count })))
}

async fn get_stats(
    State(storage): State<Arc<JsonTodoStorage>>,
) -> Result<Json<TodoStats>, (StatusCode, Json<serde_json::Value>)> {
    let stats = storage.get_stats().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
    })?;
    Ok(Json(stats))
}

// Static Assets & PWA Handlers
async fn serve_style() -> impl IntoResponse {
    serve_embedded_asset("style.css")
}

async fn serve_script() -> impl IntoResponse {
    serve_embedded_asset("app.js")
}

async fn serve_icons(Path(path): Path<String>) -> impl IntoResponse {
    serve_embedded_asset(&format!("icons/{}", path.trim_start_matches('/')))
}

async fn serve_index() -> impl IntoResponse {
    serve_embedded_asset("index.html")
}

async fn serve_manifest() -> impl IntoResponse {
    match Assets::get("manifest.json") {
        Some(content) => {
            let mut response = Response::new(content.data.into());
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/manifest+json"),
            );
            response
        }
        None => (StatusCode::NOT_FOUND, "Manifest not found").into_response(),
    }
}

async fn serve_sw() -> impl IntoResponse {
    match Assets::get("sw.js") {
        Some(content) => {
            let mut response = Response::new(content.data.into());
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/javascript"),
            );
            response.headers_mut().insert(
                header::HeaderName::from_static("service-worker-allowed"),
                HeaderValue::from_static("/"),
            );
            response
        }
        None => (StatusCode::NOT_FOUND, "Service worker not found").into_response(),
    }
}

async fn serve_favicon() -> impl IntoResponse {
    if let Some(content) = Assets::get("icons/favicon.png") {
        let mut response = Response::new(content.data.into());
        response
            .headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
        return response;
    }
    if let Some(content) = Assets::get("icons/icon.svg") {
        let mut response = Response::new(content.data.into());
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("image/svg+xml"),
        );
        return response;
    }
    (StatusCode::NOT_FOUND, "Favicon not found").into_response()
}

async fn serve_static(Path(path): Path<String>) -> impl IntoResponse {
    serve_embedded_asset(&path)
}

fn serve_embedded_asset(path: &str) -> Response {
    let clean_path = path.trim_start_matches('/');
    match Assets::get(clean_path) {
        Some(content) => {
            let mime = from_path(clean_path).first_or_octet_stream();
            let mut response = Response::new(content.data.into());
            if let Ok(val) = HeaderValue::from_str(mime.as_ref()) {
                response.headers_mut().insert(header::CONTENT_TYPE, val);
            }
            response
        }
        None => (StatusCode::NOT_FOUND, Html("<h1>404 Not Found</h1>")).into_response(),
    }
}

pub fn get_primary_lan_ip() -> Option<String> {
    if let Ok(ip) = local_ip_address::local_ip() {
        if !ip.is_loopback() {
            return Some(ip.to_string());
        }
    }
    None
}

pub fn render_qr_ascii(url: &str) -> Option<String> {
    if let Ok(code) = QrCode::new(url.as_bytes()) {
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .quiet_zone(false)
            .build();
        return Some(image);
    }
    None
}

pub async fn run_server(
    host: &str,
    port: u16,
    storage: Arc<JsonTodoStorage>,
    storage_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router(storage);
    let addr_str = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr_str).await?;

    let lan_ip = get_primary_lan_ip();

    println!("\n{}", "=".repeat(64));
    println!(" 🚀 TaskMaster - Cross-Platform Modern Todo App (Rust)");
    println!("{}", "=".repeat(64));

    if host == "0.0.0.0" {
        println!(" 💻 Local PC Access:     http://localhost:{port}");
        if let Some(ref ip) = lan_ip {
            let mobile_url = format!("http://{ip}:{port}");
            println!(" 📱 Mobile / Wi-Fi LAN:  {mobile_url}");
            if let Some(qr) = render_qr_ascii(&mobile_url) {
                println!("\n 📲 Scan with Android / iPhone / iPad camera to open instantly:\n");
                for line in qr.lines() {
                    println!("    {line}");
                }
                println!();
            }
        } else {
            println!(" 📱 Network (all):       http://0.0.0.0:{port}");
        }
    } else {
        println!(" 🌐 Access URL:          http://{host}:{port}");
    }

    println!(" 📁 Persistent Storage:  {storage_path}");
    println!(" ✨ Tip: Tap 'Install' on Android, iPhone, iPad, Windows, Mac, or Linux");
    println!("        to run TaskMaster as a standalone native app window!");
    println!(" 💡 Press Ctrl+C to stop the server");
    println!("{}\n", "=".repeat(64));

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
