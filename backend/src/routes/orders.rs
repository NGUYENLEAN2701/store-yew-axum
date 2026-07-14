use axum::{
    extract::{Path, State},
    Json,
};
use bson::{doc, oid::ObjectId};
use futures_util::TryStreamExt;
use shared::{Order, OrderInput, UpdateOrderStatusInput};

use crate::{
    error::AppError, models::order::OrderDoc, models::product::ProductDoc,
    security::admin_extractor::AdminUser, AppState,
};

fn validate_order_input(input: &OrderInput) -> Result<(), AppError> {
    if input.customer_name.trim().is_empty() || input.customer_name.len() > 200 {
        return Err(AppError::bad_request("invalid customer_name"));
    }
    if input.phone.trim().is_empty() || input.phone.len() > 30 {
        return Err(AppError::bad_request("invalid phone"));
    }
    if input.address.trim().is_empty() || input.address.len() > 500 {
        return Err(AppError::bad_request("invalid address"));
    }
    if input.note.len() > 1000 {
        return Err(AppError::bad_request("note too long"));
    }
    if input.items.is_empty() || input.items.len() > 100 {
        return Err(AppError::bad_request("invalid items"));
    }
    for item in &input.items {
        if item.quantity <= 0 || item.quantity > 1000 {
            return Err(AppError::bad_request("invalid quantity"));
        }
    }
    Ok(())
}

pub async fn create_order(
    State(state): State<AppState>,
    Json(input): Json<OrderInput>,
) -> Result<Json<Order>, AppError> {
    validate_order_input(&input)?;

    // Re-price every line item from the database instead of trusting client-supplied
    // prices, so a tampered checkout payload can't undercharge an order.
    let products = state.db.collection::<ProductDoc>("products");
    let mut priced_items = Vec::new();
    let mut total: i64 = 0;
    for item in &input.items {
        let oid = ObjectId::parse_str(&item.product_id)
            .map_err(|_| AppError::bad_request("invalid product id"))?;
        let product = products
            .find_one(doc! { "_id": oid })
            .await?
            .ok_or_else(|| AppError::bad_request("product not found"))?;
        total += product.price * item.quantity;
        priced_items.push(shared::OrderItem {
            product_id: item.product_id.clone(),
            name: product.name,
            price: product.price,
            quantity: item.quantity,
        });
    }

    let now = chrono::Utc::now().timestamp();
    let mut input = input;
    input.items = priced_items;
    let mut doc = OrderDoc::from_input(input, total, now);
    let coll = state.db.collection::<OrderDoc>("orders");
    let result = coll.insert_one(&doc).await?;
    doc.id = result.inserted_id.as_object_id();
    Ok(Json(doc.into_dto()))
}

pub async fn list_orders(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<Order>>, AppError> {
    let coll = state.db.collection::<OrderDoc>("orders");
    let mut cursor = coll.find(doc! {}).sort(doc! { "created_at": -1 }).await?;
    let mut items = Vec::new();
    while let Some(doc) = cursor.try_next().await? {
        items.push(doc.into_dto());
    }
    Ok(Json(items))
}

pub async fn update_order_status(
    _admin: AdminUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateOrderStatusInput>,
) -> Result<Json<Order>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request("invalid id"))?;
    let coll = state.db.collection::<OrderDoc>("orders");
    let status_bson = bson::to_bson(&input.status).map_err(AppError::internal)?;
    coll.update_one(doc! { "_id": oid }, doc! { "$set": { "status": status_bson } })
        .await?;
    let updated = coll
        .find_one(doc! { "_id": oid })
        .await?
        .ok_or_else(|| AppError::not_found("order not found"))?;
    Ok(Json(updated.into_dto()))
}
