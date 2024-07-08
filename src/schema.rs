// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (id) {
        id -> Uuid,
        sender_id -> Uuid,
        recipient_id -> Uuid,
        amount -> Float8,
        description -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        balance -> Float8,
        role -> Varchar,
        deleted -> Nullable<Bool>,
        created_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    transactions,
    users,
);
