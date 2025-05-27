use diesel::{Queryable, Selectable};
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub username: String,
    pub password_hash: String,
}
