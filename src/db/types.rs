use deadpool::managed::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

pub type DatabasePool = Pool<
    AsyncDieselConnectionManager<AsyncPgConnection>,
    Object<AsyncDieselConnectionManager<AsyncPgConnection>>,
>;
pub type PooledConnection = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;
