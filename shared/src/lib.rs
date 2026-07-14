use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    #[default]
    Iem,
    Dongle,
    Accessory,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Iem => "iem",
            Category::Dongle => "dongle",
            Category::Accessory => "accessory",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Category::Iem => "IEM",
            Category::Dongle => "Dongle",
            Category::Accessory => "Phụ kiện",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Price in whole VND (no decimals).
    pub price: i64,
    pub category: Category,
    pub image_url: String,
    pub stock: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductInput {
    pub name: String,
    pub description: String,
    pub price: i64,
    pub category: Category,
    pub image_url: String,
    pub stock: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cod,
    BankTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Completed,
    Cancelled,
}

impl OrderStatus {
    pub fn label(&self) -> &'static str {
        match self {
            OrderStatus::Pending => "Chờ xử lý",
            OrderStatus::Confirmed => "Đã xác nhận",
            OrderStatus::Shipped => "Đang giao",
            OrderStatus::Completed => "Hoàn tất",
            OrderStatus::Cancelled => "Đã hủy",
        }
    }

    pub fn all() -> [OrderStatus; 5] {
        [
            OrderStatus::Pending,
            OrderStatus::Confirmed,
            OrderStatus::Shipped,
            OrderStatus::Completed,
            OrderStatus::Cancelled,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub name: String,
    pub price: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderInput {
    pub customer_name: String,
    pub phone: String,
    pub address: String,
    pub note: String,
    pub items: Vec<OrderItem>,
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateOrderStatusInput {
    pub status: OrderStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetupStatus {
    pub needs_setup: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminMe {
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    pub token: String,
    pub question: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptchaAnswer {
    pub token: String,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    #[serde(default)]
    pub captcha_required: bool,
}
