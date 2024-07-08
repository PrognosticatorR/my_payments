use actix_web::{web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;

use crate::auth_guard::AuthGuard;
use crate::db::DbPool;
use crate::models::user::*;
use crate::models::user::{FormUser, NewUser, User};
use crate::utils::{self, create_response};

pub async fn register(pool: web::Data<DbPool>, user: web::Json<NewUser>) -> HttpResponse {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return utils::create_response(
                false,
                500,
                Some("Failed to get DB connection"),
                None::<()>,
            )
        }
    };

    let balance = user.balance.unwrap_or(0.00);

    let new_user = NewUser {
        username: user.username.clone(),
        password: user.password.clone(),
        email: user.email.clone(),
        balance: Some(balance),
        role: user.role.clone(),
    };

    match User::create(new_user, &mut conn) {
        Ok(user) => utils::create_response(true, 200, None, Some(user)),
        Err(e) => utils::create_response(
            false,
            500,
            Some(&format!("Error during register: {:?}", e)),
            None::<()>,
        ),
    }
}

pub async fn login(pool: web::Data<DbPool>, user: web::Json<LoginRequest>) -> HttpResponse {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    use crate::schema::users::dsl::{email, users};

    match users.filter(email.eq(&user.email)).first::<User>(&mut conn) {
        Ok(user_record) => {
            if verify(&user.password, &user_record.password).unwrap() {
                let jwt_token = utils::generate_jwt(user_record).unwrap();
                utils::create_response(
                    true,
                    200,
                    None,
                    Some(json!({"token": format!("Bearer {}", jwt_token)})),
                )
            } else {
                utils::create_response(false, 401, Some("Invalid credentials"), None::<()>)
            }
        }
        Err(_) => utils::create_response(false, 400, Some("Invalid credentials"), None::<()>),
    }
}

pub async fn get_profile(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    _auth_guard: AuthGuard,
) -> HttpResponse {
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return utils::create_response(
                false,
                500,
                Some("Failed to get DB connection"),
                None::<()>,
            )
        }
    };

    use crate::schema::users::dsl::{id, users};

    match users
        .filter(id.eq(user_id.into_inner()))
        .first::<User>(&mut conn)
    {
        Ok(user) => utils::create_response(true, 200, None, Some(json!({"user": user}))),
        Err(_) => utils::create_response(false, 404, Some("User not found"), None::<()>),
    }
}

pub async fn update_password(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    req: web::Json<UpdatePasswordRequest>,
    auth_guard: AuthGuard,
) -> HttpResponse {
    if auth_guard.claims.id != *user_id {
        return create_response(false, 401, Some("Unauthorised request!"), None::<()>);
    }
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    use crate::schema::users::dsl::{id, password, users};

    let user = match users
        .filter(id.eq(user_id.into_inner()))
        .first::<User>(&mut conn)
    {
        Ok(user) => user,
        Err(_) => return utils::create_response(false, 404, Some("User not found"), None::<()>),
    };

    if !verify(&req.current_password, &user.password).unwrap() {
        return utils::create_response(
            false,
            401,
            Some("Current password is incorrect"),
            None::<()>,
        );
    }

    let hashed_password = hash(&req.new_password, DEFAULT_COST).unwrap();
    diesel::update(users.filter(id.eq(user.id)))
        .set(password.eq(hashed_password))
        .execute(&mut conn)
        .expect("Error updating password");

    utils::create_response(true, 202, Some("Password update successful"), None::<()>)
}

pub async fn update_profile(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    req: web::Json<FormUser>,
    auth_guard: AuthGuard,
) -> HttpResponse {
    if auth_guard.claims.id != *user_id {
        return create_response(false, 401, Some("Unauthorised request!"), None::<()>);
    }
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    use crate::schema::users::dsl::{email, id, username, users};

    let target = users.filter(id.eq(user_id.into_inner()));

    if let Some(new_username) = &req.username {
        diesel::update(target)
            .set(username.eq(new_username))
            .execute(&mut conn)
            .expect("Error updating username");
    }

    if let Some(new_email) = &req.email {
        diesel::update(target)
            .set(email.eq(new_email))
            .execute(&mut conn)
            .expect("Error updating email");
    }

    utils::create_response(true, 200, Some("Profile updated successfully"), None::<()>)
}

pub async fn update_balance(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    data: web::Json<UpdateUserBalance>,
    auth_guard: AuthGuard,
) -> HttpResponse {
    if !auth_guard.claims.has_role("admin") {
        return create_response(false, 401, Some("Unauthorised request!"), None::<()>);
    }
    let mut conn = pool.get().expect("couldn't get db connection from pool");
    use crate::schema::users;
    // Example of fetching user from the database
    let user = match users::table
        .filter(users::id.eq(*user_id))
        .first::<User>(&mut conn)
    {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::NotFound().finish();
        }
    };

    // Example of how you might handle balance updates
    let updated_balance = match data.transaction_type {
        TransactionType::Withdrawal => user.balance - data.amount,
        TransactionType::Deposit => user.balance + data.amount,
    };

    // Example of updating user's balance in the database
    let update_result = diesel::update(users::table.find(*user_id))
        .set(users::balance.eq(updated_balance))
        .execute(&mut conn);

    match update_result {
        Ok(_) => {
            return utils::create_response(false, 500, Some("User Balance Updated"), None::<()>)
        }
        Err(_) => {
            return utils::create_response(
                false,
                500,
                Some("Failed to get DB connection"),
                None::<()>,
            )
        }
    }
}

pub async fn delete_user(
    pool: web::Data<DbPool>,
    user_id: web::Path<Uuid>,
    auth_guard: AuthGuard,
) -> HttpResponse {
    if auth_guard.claims.id != *user_id && !auth_guard.claims.has_role("admin") {
        return create_response(
            false,
            401,
            Some("Only user and admin can delete!"),
            None::<()>,
        );
    }
    use crate::schema::users;

    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => {
            return create_response(
                false,
                500,
                Some("Failed to get database connection"),
                None::<()>,
            );
        }
    };

    let update = DeleteUser {
        deleted: true,
        deleted_at: Utc::now().naive_utc(),
    };

    let result = conn.transaction(|db_conn| {
        let user = users::table.find(*user_id);
        diesel::update(user).set(&update).execute(db_conn)
    });

    match result {
        Ok(_) => create_response(true, 200, Some("User deleted successfully!"), None::<()>),
        Err(e) => {
            let error_message = format!("Error during deleting user: {:?}", e);
            create_response(false, 400, Some(&error_message), None::<()>)
        }
    }
}
