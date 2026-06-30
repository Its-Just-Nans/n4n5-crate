//! Sharing web server

use axum::{
    Router,
    extract::{Multipart, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use mime_guess::from_path;
use std::fmt::Write;
use std::{fs, net::UdpSocket, sync::Arc};
use tokio::fs as tokio_fs;

/// Upload dir
const UPLOAD_DIR: &str = "uploads";
/// Port for server
const PORT: u16 = 8000;

/// Server state
#[derive(Clone)]
struct AppState {
    /// Upload dir
    upload_dir: String,
}

/// main share function
/// # Errors
/// Return error if the server fails
pub(crate) async fn main() -> std::io::Result<()> {
    fs::create_dir_all(UPLOAD_DIR)?;

    let state = Arc::new(AppState {
        upload_dir: UPLOAD_DIR.to_string(),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/upload", post(upload))
        .route("/files/{name}", get(download))
        .with_state(state);

    let addr = format!("0.0.0.0:{PORT}");

    println!("==================================================");
    println!(" Axum File Upload Server");
    println!("==================================================");
    println!("Localhost : http://localhost:{PORT}");
    println!("Loopback  : http://127.0.0.1:{PORT}");
    println!("LAN       : http://{}:{PORT}", local_ip()?);
    println!("==================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;
    Ok(())
}

/// Get the local ip
/// # Errors
/// Return errors if cannot connect
fn local_ip() -> std::io::Result<String> {
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.connect("8.8.8.8:80")?;
    let ip = sock.local_addr()?.ip().to_string();
    Ok(ip)
}

/// Show the index
async fn index() -> Html<String> {
    let mut files_html = String::new();

    if let Ok(entries) = fs::read_dir(UPLOAD_DIR) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let _ = write!(files_html, "<li><a href=\"/files/{name}\">{name}</a></li>");
        }
    }

    Html(format!(
        r#"
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Upload Server</title>
<style>
body {{ font-family: Arial; background:#f3f3f3; margin:40px; }}
.container {{ max-width:700px; margin:auto; background:white; padding:20px; border-radius:10px; }}
.drop {{ border:3px dashed #888; padding:40px; text-align:center; margin:20px 0; }}
</style>
</head>
<body>
<div class="container">
<h2>📁 File Upload Server</h2>

<form action="/upload" method="post" enctype="multipart/form-data">
<div class="drop">
<input type="file" name="file" multiple>
</div>
<button type="submit">Upload</button>
</form>

<h3>Files</h3>
<ul>
{files_html}
</ul>
</div>
</body>
</html>
"#
    ))
}

/// Upload function
async fn upload(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> impl IntoResponse {
    loop {
        let Ok(Some(field)) = multipart.next_field().await else {
            break;
        };
        let name = match field.file_name() {
            Some(n) => n.to_string(),
            None => continue,
        };

        let Ok(data) = field.bytes().await else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let path = format!("{}/{}", state.upload_dir, sanitize(&name));

        match tokio_fs::write(path, data).await {
            Ok(b) => b,
            Err(_) => return StatusCode::BAD_REQUEST.into_response(),
        }
    }
    Redirect::to("/").into_response()
}

/// Show files uploaded
async fn download(axum::extract::Path(name): axum::extract::Path<String>) -> impl IntoResponse {
    let path = format!("{UPLOAD_DIR}/{}", sanitize(&name));
    match tokio_fs::read(&path).await {
        Ok(data) => {
            let mime = from_path(&path).first_or_octet_stream();

            match Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(data))
            {
                Ok(resp) => resp,
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }

        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Quick and dirty sanitize
fn sanitize(name: &str) -> String {
    name.replace('/', "_").replace("..", "_")
}
