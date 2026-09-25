use chrono::{DateTime, Utc};
use crate::domain::primitives::{UserId, Username};

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub uid: UserId,
    pub name: Username,
    pub created_at: DateTime<Utc>
}
