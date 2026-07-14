use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use shared::{Order as OrderDto, OrderInput, OrderItem, OrderStatus, PaymentMethod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub customer_name: String,
    pub phone: String,
    pub address: String,
    pub note: String,
    pub items: Vec<OrderItem>,
    pub total: i64,
    pub payment_method: PaymentMethod,
    pub status: OrderStatus,
    pub created_at: i64,
}

impl OrderDoc {
    pub fn from_input(input: OrderInput, total: i64, created_at: i64) -> Self {
        Self {
            id: None,
            customer_name: input.customer_name,
            phone: input.phone,
            address: input.address,
            note: input.note,
            items: input.items,
            total,
            payment_method: input.payment_method,
            status: OrderStatus::Pending,
            created_at,
        }
    }

    pub fn into_dto(self) -> OrderDto {
        OrderDto {
            id: self.id.map(|i| i.to_hex()).unwrap_or_default(),
            customer_name: self.customer_name,
            phone: self.phone,
            address: self.address,
            note: self.note,
            items: self.items,
            total: self.total,
            payment_method: self.payment_method,
            status: self.status,
            created_at: self.created_at,
        }
    }
}
