use crate::schema::{transactions, users};
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, PartialEq)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: uuid::Uuid,
    pub sender_id: uuid::Uuid,
    pub recipient_id: uuid::Uuid,
    pub amount: f64,
    pub description: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub amount: f64,
    pub description: String,
}
#[derive(Queryable, Serialize)]
pub struct TransactionWithDetails {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub recipient_id: Uuid,
    pub amount: f64,
    pub sender_username: String,
    pub sender_email: String,
    pub recipient_username: String,
    pub recipient_email: String,
}
impl Transaction {
    pub fn create(
        new_transaction: NewTransaction,
        conn: &mut PgConnection,
    ) -> QueryResult<Transaction> {
        diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .get_result(conn)
    }
    pub fn get_by_id(
        transaction_id: Uuid,
        conn: &mut PgConnection,
    ) -> QueryResult<TransactionWithDetails> {
        // Define aliases for the users table
        let (sender_alias, recipient_alias) = diesel::alias!(users as sender, users as recipient);

        transactions::table
            .filter(transactions::id.eq(transaction_id))
            .inner_join(sender_alias.on(transactions::sender_id.eq(sender_alias.field(users::id))))
            .inner_join(
                recipient_alias.on(transactions::recipient_id.eq(recipient_alias.field(users::id))),
            )
            .select((
                transactions::id,
                transactions::sender_id,
                transactions::recipient_id,
                transactions::amount,
                sender_alias.field(users::username),
                sender_alias.field(users::email),
                recipient_alias.field(users::username),
                recipient_alias.field(users::email),
            ))
            .get_result(conn)
    }
    pub fn list_by_user(user_id: Uuid, conn: &mut PgConnection) -> QueryResult<Vec<Transaction>> {
        transactions::table
            .filter(transactions::sender_id.eq(user_id))
            .load::<Transaction>(conn)
    }
}
