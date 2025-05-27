use crate::auth::backend::Error::VerifyError;
use crate::db::models::User;
use crate::db::schema::users::dsl::*;
use crate::db::types::{DatabasePool, PooledConnection};
use crate::define_permissions;
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, AuthzBackend, UserId};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use password_auth::verify_password;
use serde::Deserialize;
use std::collections::HashSet;
use tokio::task;
use uuid::Uuid;

// Don't log the password hash
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("parent_id", &self.parent_id)
            .field("username", &self.username)
            .field("password", &"[redacted]")
            .finish()
    }
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Uuid {
        self.id
    }

    // We use the password hash so when someone changes
    // their password their session becomes invalid
    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct Backend {
    db: DatabasePool,
}

impl Backend {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub async fn get_connection(&self) -> PooledConnection {
        self.db.get().await.unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[allow(clippy::enum_variant_names)]
    #[error(transparent)]
    VerifyError(#[from] password_auth::VerifyError),

    #[error(transparent)]
    TaskJoin(#[from] task::JoinError),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let mut conn = self.get_connection().await;

        let user = users
            .filter(username.eq(creds.username))
            .first::<User>(&mut conn)
            .await?;

        task::spawn_blocking(
            || match verify_password(creds.password, &user.password_hash) {
                Ok(_) => Ok(Some(user)),
                Err(err) => Err(VerifyError(err)),
            },
        )
        .await?
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let mut conn = self.get_connection().await;

        let user = users
            .filter(id.eq(user_id))
            .first::<User>(&mut conn)
            .await?;

        Ok(Some(user))
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Permission {
    pub name: String,
}

impl From<&str> for Permission {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

define_permissions! {
    deploy_file => "repo.file.deploy",
    delete_file => "repo.file.delete"
}

#[async_trait]
impl AuthzBackend for Backend {
    type Permission = Permission;

    async fn get_user_permissions(
        &self,
        _user: &Self::User,
    ) -> Result<HashSet<Self::Permission>, Self::Error> {
        let mut set = HashSet::new();
        set.insert(Permission::deploy_file());
        set.insert(Permission::delete_file());
        Ok(set)
    }
}
