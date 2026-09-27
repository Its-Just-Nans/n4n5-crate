//! Sharing web server
use std::io;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{fs, net::UdpSocket, sync::Arc};
use std::{net::SocketAddr, time::SystemTime};

use axum::{
    Router,
    extract::{ConnectInfo, DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use docling::SourceDocument;
use tokio::fs as tokio_fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

/// Upload dir
const UPLOAD_DIR: &str = "/tmp";
/// Port for server
const PORT: u16 = 8000;

/// Server state
#[derive(Clone)]
struct AppState {
    /// Upload dir
    upload_dir: PathBuf,
    /// out file
    out_file: Option<PathBuf>,
}

/// main share function
/// # Errors
/// Return error if the server fails
pub async fn cli_main() -> std::io::Result<()> {
    let state = Arc::new(AppState {
        upload_dir: PathBuf::from(UPLOAD_DIR),
        out_file: Some(PathBuf::from("Ocr.txt")),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/ocr", post(ocr))
        .layer(DefaultBodyLimit::max(128 * 1024 * 1024)) // 128Mib
        .with_state(state);

    let addr = format!("0.0.0.0:{PORT}");

    println!("==================================================");
    println!(" OCR Server");
    println!("==================================================");
    println!("Localhost : http://localhost:{PORT}");
    println!("Loopback  : http://127.0.0.1:{PORT}");
    println!("LAN       : http://{}:{PORT}", local_ip()?);
    println!("==================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
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

/// INDEX template
fn fill_template(content: &str) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
    <meta charset="utf-8">
    <title>Upload Server</title>
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{ font-family: Arial; background:#f3f3f3; margin:40px; }}
        .container {{ max-width:700px; margin:auto; background:white; padding:20px; border-radius:10px; }}
        .drop {{ border:3px dashed #888; padding:40px; text-align:center; margin:20px 0; }}
    </style>
    </head>
    <body>
        <div class="container">
            <h2>📁 OCR Server - uses HTTP (without S)</h2>
            <form action="/ocr" method="post" enctype="multipart/form-data">
                <div class="drop">
                    <input type="file" name="file" multiple>
                </div>
                <button type="submit">Upload</button>
            </form>
            {content}
        </div>
    </body>
    </html>
"#
    )
}

/// Show the index
async fn index() -> Html<String> {
    Html(fill_template(""))
}

/// Upload function
#[allow(clippy::too_many_lines)]
async fn ocr(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let new_field = match multipart.next_field().await {
        Ok(f) => f,
        Err(err) => {
            let msg = "Error reading multipart field";
            eprintln!("{} - {msg}: {err}", addr.ip());
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    };
    let Some(field) = new_field else {
        return (StatusCode::BAD_REQUEST, "No field").into_response();
    };
    let name = {
        let duration = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => format!("{}", n.as_secs()),
            Err(_e) => "no_timestamp".to_string(),
        };
        format!("text_{duration}.png")
    };

    if name.is_empty() {
        let msg = "Empty file name";
        eprintln!("{} - {msg}", addr.ip());
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }
    let data = match field.bytes().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("{} - Failed to read bytes of {name}: {e}", addr.ip());
            return (
                StatusCode::BAD_REQUEST,
                "Failed to read bytes - file too big?",
            )
                .into_response();
        }
    };
    if data.is_empty() {
        return (StatusCode::BAD_REQUEST, "File is empty").into_response();
    }
    let path = state.upload_dir.join(name);

    println!(
        "{}: receiving {}: {} bytes",
        addr.ip(),
        path.display(),
        data.len()
    );
    if let Err(err) = fs::create_dir_all(&state.upload_dir) {
        eprintln!(
            "Failed to create the upload folder {}: {err}",
            state.upload_dir.display()
        );
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    if tokio_fs::try_exists(&path).await.is_ok_and(|res| res) {
        let msg = format!("Path already exists: '{}'", path.display());
        eprintln!("{} - {msg}", addr.ip());
        return (StatusCode::BAD_REQUEST, msg).into_response();
    }
    match tokio_fs::write(&path, data).await {
        Ok(b) => b,
        Err(err) => {
            let msg = format!("Failed to write '{}'", path.display());
            eprintln!("{} - {msg}: {err}", addr.ip());
            return (StatusCode::BAD_REQUEST, msg).into_response();
        }
    }
    match run_tesseract(&path).await {
        Ok(text) => {
            if let Some(output_file) = &state.out_file {
                // Append the OCR response to the text file.
                let Ok(mut file) = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(output_file)
                    .await
                else {
                    return (StatusCode::BAD_REQUEST, "Cannot open output_file").into_response();
                };

                let Ok(()) = file.write_all(text.as_bytes()).await else {
                    return (StatusCode::BAD_REQUEST, "Cannot open output_file").into_response();
                };
                let Ok(()) = file.write_all(b"\n").await else {
                    return (StatusCode::BAD_REQUEST, "Cannot open output_file").into_response();
                };
            }

            if let Err(_err) = tokio::fs::remove_file(path).await {
                return (StatusCode::BAD_REQUEST, "Cannot remove file").into_response();
            }
            let content =
                format!(r#"<textarea style="width: 100%; height: 100vh">{text}</textarea>"#);
            (StatusCode::OK, Html(fill_template(&content))).into_response()
        }
        Err(err) => {
            let msg = format!("Failed to run tesseract '{}'", path.display());
            eprintln!("{} - {msg}: {err}", addr.ip());
            (StatusCode::BAD_REQUEST, msg).into_response()
        }
    }
}

/// run tesseract
/// # Errors
/// Errors if fails to run tesseract
async fn run_tesseract(image_path: &Path) -> io::Result<String> {
    let result = timeout(
        Duration::from_secs(30),
        Command::new("tesseract")
            .arg(image_path)
            .arg("stdout")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "tesseract timed out"))??;

    if !result.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }

    let text = String::from_utf8_lossy(&result.stdout).to_string();

    // Send/return the response.
    Ok(text)
}
//
// /// run docling
// /// # Errors
// /// Errors if fails
// async fn run_docling(image_path: &Path) -> io::Result<String> {
//     use docling::DocumentConverter;
//
//     let text = timeout(Duration::from_secs(30), async {
//         let converter = DocumentConverter::new().strict(true);
//         let result = converter
//             .convert(SourceDocument::from_file(image_path).unwrap())
//             .unwrap();
//         Ok::<_, io::Error>(result.document.export_to_markdown())
//     })
//     .await
//     .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "tesseract timed out"))??;
//
//     // Send/return the response.
//     Ok(text)
// }
