use crate::db::types::DatabasePool;
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use std::env;

#[derive(Clone)]
pub struct App {
    pub db: DatabasePool,
}

impl App {
    pub async fn new() -> Self {
        Self {
            db: {
                let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

                let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
                Pool::builder(config).build().unwrap()
            },
        }
    }
}
