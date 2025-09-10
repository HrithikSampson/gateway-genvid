// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "job_type"))]
    pub struct JobType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "payment_enum"))]
    pub struct PaymentEnum;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "progress_enum"))]
    pub struct ProgressEnum;
}

diesel::table! {
    characters (id) {
        id -> Int4,
        user_id -> Int4,
        voice_storage_id -> Nullable<Text>,
        image_storage_id -> Nullable<Text>,
    }
}

diesel::table! {
    dialogue (id) {
        id -> Int4,
        script_id -> Int4,
        character_id -> Int4,
        speech -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::JobType;

    jobs (id) {
        id -> Int4,
        type_of_job -> Nullable<JobType>,
        user_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PaymentEnum;

    payment_history (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        stripe_subscription_id -> Nullable<Varchar>,
        status -> Nullable<PaymentEnum>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ProgressEnum;

    script (id) {
        id -> Int4,
        user_id -> Int4,
        progress -> Nullable<ProgressEnum>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 50]
        auth_type_or_provider -> Nullable<Varchar>,
        refresh_token -> Text,
        credit -> Int4,
        #[max_length = 255]
        name -> Varchar,
        stripe_customer_id -> Nullable<Text>,
        password_hash -> Nullable<Text>,
    }
}

diesel::joinable!(characters -> users (user_id));
diesel::joinable!(dialogue -> characters (character_id));
diesel::joinable!(dialogue -> script (script_id));
diesel::joinable!(jobs -> users (user_id));
diesel::joinable!(payment_history -> users (user_id));
diesel::joinable!(script -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    characters,
    dialogue,
    jobs,
    payment_history,
    script,
    users,
);
