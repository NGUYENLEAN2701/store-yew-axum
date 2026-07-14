use axum::{
    extract::{Path, Query, State},
    Json,
};
use bson::{doc, oid::ObjectId};
use futures_util::TryStreamExt;
use serde::Deserialize;
use shared::{Product, ProductInput};

use crate::{error::AppError, models::product::ProductDoc, security::admin_extractor::AdminUser, AppState};

#[derive(Deserialize)]
pub struct ProductQuery {
    pub category: Option<String>,
}

fn validate_product_input(input: &ProductInput) -> Result<(), AppError> {
    if input.name.trim().is_empty() || input.name.len() > 200 {
        return Err(AppError::bad_request("invalid name"));
    }
    if input.description.len() > 5000 {
        return Err(AppError::bad_request("description too long"));
    }
    if input.price < 0 || input.price > 1_000_000_000 {
        return Err(AppError::bad_request("invalid price"));
    }
    if input.stock < 0 || input.stock > 1_000_000 {
        return Err(AppError::bad_request("invalid stock"));
    }
    if input.image_url.len() > 2000
        || !(input.image_url.starts_with("http://") || input.image_url.starts_with("https://"))
    {
        return Err(AppError::bad_request("invalid image url"));
    }
    Ok(())
}

pub async fn list_products(
    State(state): State<AppState>,
    Query(q): Query<ProductQuery>,
) -> Result<Json<Vec<Product>>, AppError> {
    let coll = state.db.collection::<ProductDoc>("products");
    let filter = match q.category {
        Some(cat) => doc! { "category": cat },
        None => doc! {},
    };
    let mut cursor = coll.find(filter).await?;
    let mut items = Vec::new();
    while let Some(doc) = cursor.try_next().await? {
        items.push(doc.into_dto());
    }
    Ok(Json(items))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Product>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("invalid id"))?;
    let coll = state.db.collection::<ProductDoc>("products");
    let doc = coll.find_one(doc! { "_id": oid }).await?;
    match doc {
        Some(d) => Ok(Json(d.into_dto())),
        None => Err(AppError::not_found("product not found")),
    }
}

pub async fn create_product(
    _admin: AdminUser,
    State(state): State<AppState>,
    Json(input): Json<ProductInput>,
) -> Result<Json<Product>, AppError> {
    validate_product_input(&input)?;
    let coll = state.db.collection::<ProductDoc>("products");
    let now = chrono::Utc::now().timestamp();
    let mut doc = ProductDoc::from_input(input, now);
    let result = coll.insert_one(&doc).await?;
    doc.id = result.inserted_id.as_object_id();
    Ok(Json(doc.into_dto()))
}

pub async fn update_product(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<ProductInput>,
) -> Result<Json<Product>, AppError> {
    validate_product_input(&input)?;
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("invalid id"))?;
    let coll = state.db.collection::<ProductDoc>("products");
    let category_bson = bson::to_bson(&input.category).map_err(AppError::internal)?;
    let update = doc! {
        "$set": {
            "name": &input.name,
            "description": &input.description,
            "price": input.price,
            "category": category_bson,
            "image_url": &input.image_url,
            "stock": input.stock,
        }
    };
    coll.update_one(doc! { "_id": oid }, update).await?;
    let updated = coll
        .find_one(doc! { "_id": oid })
        .await?
        .ok_or_else(|| AppError::not_found("product not found"))?;
    Ok(Json(updated.into_dto()))
}

pub async fn delete_product(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("invalid id"))?;
    let coll = state.db.collection::<ProductDoc>("products");
    coll.delete_one(doc! { "_id": oid }).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
