use crate::app::App;
use crate::auth::backend::Backend;
use crate::auth::basic_auth::MavenAuth;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, put};
use axum::Router;
use axum_login::login_required;
use futures_util::StreamExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use tracing::error;

pub fn router() -> Router<App> {
    Router::new()
        .route("/maven/{repository}/{*gav}", put(post::deploy_file))
        .route("/maven/{repository}/{*gav}", delete(delete::delete_file))
        .route_layer(login_required!(Backend, login_url = "/login"))
        .route("/maven/{repository}/{*gav}", get(get::find_file))
}

mod get {
    use super::*;

    pub async fn find_file(Path((repository, gav)): Path<(String, String)>) -> impl IntoResponse {
        let path = &format!("/home/ithundxr/Projects/OtherCode/axolotl/test/{}", gav);

        // TODO - Private repos & fix path

        match File::open(&path).await {
            Ok(file) => {
                let stream = ReaderStream::new(file);
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "application/octet-stream")
                    .body(Body::from_stream(stream))
                    .unwrap()
            },
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

mod post {
    use super::*;

    pub async fn deploy_file(
        Path((repository, gav)): Path<(String, String)>,
        _: MavenAuth,
        body: Body,
    ) -> StatusCode {
        let mut data_stream = body.into_data_stream();

        let path = &format!("/home/ithundxr/Projects/OtherCode/axolotl/test/{}", gav);

        // TODO - Setup prevention of redeployment, private repos & fix path

        if let Some(parent) = std::path::Path::new(path).parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                error!("Failed to create directory: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }

        let mut file = match File::create(&path).await {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create file at {path}: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            },
        };

        while let Some(chunk) = data_stream.next().await {
            match chunk {
                Ok(data) => {
                    if let Err(e) = file.write_all(&data).await {
                        error!("Failed to write chunk to file: {e}");
                        return StatusCode::INTERNAL_SERVER_ERROR;
                    }
                },
                Err(e) => {
                    error!("Error reading body: {e}");
                    return StatusCode::INTERNAL_SERVER_ERROR;
                },
            }
        }

        StatusCode::OK
    }
}

mod delete {
    use super::*;

    pub async fn delete_file(
        Path((repository, gav)): Path<(String, String)>,
        _: MavenAuth,
    ) -> StatusCode {
        let path = &format!("/home/ithundxr/Projects/OtherCode/axolotl/test/{}", gav);

        // TODO - Private repo support, fix path, add better error handling/logging

        match tokio::fs::remove_file(path).await {
            Ok(_) => StatusCode::OK,
            Err(e) => {
                error!("Failed to delete file: {e}");
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }
}
