// src/models/user.rs
use crate::schema::users;
use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Insertable, Queryable)]
#[diesel(table_name= users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub balance: f64,
    pub role: String,
    #[serde(skip_serializing)]
    pub deleted: Option<bool>,
    pub created_at: NaiveDateTime,
    #[serde(skip_serializing)]
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Insertable, Serialize)]
#[diesel(table_name= users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub balance: Option<f64>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize,Serialize, AsChangeset)]
#[diesel(table_name= users)]
pub struct FormUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, AsChangeset)]
#[diesel(table_name=users)]
pub struct DeleteUser {
    pub deleted_at: NaiveDateTime,
    pub deleted: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionType {
    Withdrawal,
    Deposit,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserBalance {
    pub amount: f64,
    pub transaction_type: TransactionType,
}

impl User {
    pub fn create(new_user: NewUser, conn: &mut PgConnection) -> QueryResult<User> {
        let hashed_password = hash(new_user.password, DEFAULT_COST).unwrap();
        let new_user = NewUser {
            username: new_user.username,
            email: new_user.email,
            password: hashed_password,
            role: new_user.role,
            balance: new_user.balance,
        };

        let inserted_user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(conn)?;

        Ok(inserted_user)
    }
}
