use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::StatusCode;
use base64::engine::general_purpose;
use base64::Engine;

pub struct MavenAuth;

impl<S> FromRequestParts<S> for MavenAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let authorization_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or((StatusCode::UNAUTHORIZED, "Missing authorization header"))?;

        let auth_str = authorization_header
            .to_str()
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid authorization header"))?;

        let (scheme, credentials) = auth_str.split_once(' ').ok_or((
            StatusCode::BAD_REQUEST,
            "Invalid authorization header format",
        ))?;

        if scheme != "Basic" {
            return Err((
                StatusCode::BAD_REQUEST,
                "Authorization scheme must be Basic",
            ));
        }

        let decoded = general_purpose::STANDARD.decode(credentials).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid base64 in authorization header",
            )
        })?;

        let decoded_str = String::from_utf8(decoded).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Authorization header is not valid UTF-8",
            )
        })?;

        let (username, password) = decoded_str
            .split_once(':')
            .ok_or((StatusCode::BAD_REQUEST, "Invalid basic auth format"))?;

        // TODO - Check if user exists, has perms, etc
        if username == "test" && password == "testpass" {
            Ok(MavenAuth)
        } else {
            Err((StatusCode::UNAUTHORIZED, "Authentication failed"))
        }
    }
}
