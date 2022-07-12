use diesel::Queryable;

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub joined: String,
    pub modified: String
}
