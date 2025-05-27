use crate::auth::backend::Backend;

pub type AuthSession = axum_login::AuthSession<Backend>;
