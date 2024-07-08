use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use diesel::Connection;
use uuid::Uuid;

use crate::auth_guard::AuthGuard;
use crate::db::DbPool;
use crate::errors::AppError;
use crate::models::{
    transaction::{NewTransaction, Transaction},
    user::User,
};
use crate::schema::users;
use crate::utils::{self, create_response};

pub async fn create_transaction(
    pool: web::Data<DbPool>,
    transaction: web::Json<NewTransaction>,
    auth_guard: AuthGuard,
) -> HttpResponse {
    if auth_guard.claims.id != transaction.sender_id {
        return create_response(
            false,
            400,
            Some("Only can Transfer from own account"),
            None::<()>,
        );
    }
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let transaction_result: Result<Transaction, AppError> = conn.transaction(|conn| {
        let sender = users::table
            .find(transaction.sender_id)
            .first::<User>(conn)
            .map_err(AppError::from)?;

        users::table
            .find(transaction.recipient_id)
            .first::<User>(conn)
            .map_err(AppError::from)?;

        if sender.balance < transaction.amount {
            return Err(AppError::InsufficientBalance);
        }

        // Deduct amount from sender
        diesel::update(users::table.find(transaction.sender_id))
            .set(users::balance.eq(users::balance - transaction.amount))
            .execute(conn)?;

        // Add amount to receiver
        diesel::update(users::table.find(transaction.recipient_id))
            .set(users::balance.eq(users::balance + transaction.amount))
            .execute(conn)?;

        Transaction::create(transaction.into_inner(), conn).map_err(AppError::from)
    });

    match transaction_result {
        Ok(transaction) => {
            utils::create_response(true, 200, Some("Transaction Successful"), Some(transaction))
        }
        Err(AppError::InsufficientBalance) => {
            utils::create_response(false, 400, Some("Insuffient Balance"), None::<()>)
        }
        Err(AppError::DatabaseError(diesel::result::Error::NotFound)) => {
            utils::create_response(false, 404, Some("Not Found"), None::<()>)
        }
        Err(_) => utils::create_response(false, 500, Some("Something went wrong!"), None::<()>),
    }
}

pub async fn get_transaction(
    pool: web::Data<DbPool>,
    transaction_id: web::Path<Uuid>,
    _auth_guard: AuthGuard,
) -> HttpResponse {
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match Transaction::get_by_id(transaction_id.into_inner(), &mut conn) {
        Ok(transaction) => utils::create_response(true, 200, None, Some(transaction)),
        Err(_) => utils::create_response(false, 404, Some("Transaction Not Found"), None::<()>),
    }
}

pub async fn list_transactions(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    auth_guard: AuthGuard,
) -> HttpResponse {
    let user_id = user_id.into_inner();
    if auth_guard.claims.id != user_id.clone() {
        return create_response(
            false,
            400,
            Some("Only can see own transaction!"),
            None::<()>,
        );
    }
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    match Transaction::list_by_user(user_id, &mut conn) {
        Ok(transactions) => utils::create_response(true, 200, None, Some(transactions)),
        Err(_) => utils::create_response(false, 500, Some("Something went wrong!"), None::<()>),
    }
}
