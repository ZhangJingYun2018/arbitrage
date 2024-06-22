use std::sync::Arc;

use axum::{Extension, Json};
use sqlx::{MySql, Pool};

pub async fn handler() -> &'static str {
    "Hello, world!"
}
