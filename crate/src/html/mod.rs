use actix_files::NamedFile;
use std::path::{Path, PathBuf};
use actix_web::{HttpRequest, Result};

/// Devuelve el archivo HTML de logs para ser servido desde un servidor externo.
pub async fn serve_logs_html(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/html/requests_history_logs.html");
    Ok(NamedFile::open(path)?)
}