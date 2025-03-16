use axum_typed_multipart::{FieldData, TryFromMultipart};
use bigdecimal::BigDecimal;
use diesel::{prelude::AsChangeset, Insertable, Queryable, QueryableByName, Selectable};
use serde_derive::Serialize;
use tempfile::NamedTempFile;

#[derive(Serialize, Queryable, Selectable, Insertable, AsChangeset, QueryableByName, Debug)]
#[diesel(table_name = crate::db::schema::ads)]
pub struct Ad {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub price: BigDecimal,
    pub status: String,
    pub user_email: String,
    pub user_phone: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub top_ad: bool,
    pub images: serde_json::Value,
}

#[derive(TryFromMultipart)]
pub struct AdRequest {
    pub title: String,
    pub description: String,
    pub price: f64,
    pub user_email: String,
    pub user_phone: String,
    pub top_ad: bool,
    pub images: Vec<FieldData<NamedTempFile>>,
    pub image_ids: Vec<String>,
}

pub struct AdContent {
    pub title: String,
    pub description: String,
    pub price: f64,
    pub user_email: String,
    pub user_phone: String,
    pub top_ad: bool,
}
