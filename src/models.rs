use crate::schema::users;
use clap::Parser;
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Clone, Parser)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))] // Optional: ensures fields match types in Postgres
pub struct User {
    pub id: i32,
    pub auth_type_or_provider: Option<String>,
    pub refresh_token: String,
    pub credit: i32,
    pub name: String,
    pub stripe_customer_id: Option<String>,
    pub password_hash: Option<String>,
}

use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub auth_type_or_provider: Option<&'a str>,
    pub refresh_token: &'a str,
    pub credit: i32,
    pub name: &'a str,
    pub stripe_customer_id: Option<&'a str>,
    pub password_hash: Option<&'a str>,
}