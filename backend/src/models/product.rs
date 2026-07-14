use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use shared::{Category, Product as ProductDto, ProductInput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub category: Category,
    pub image_url: String,
    pub stock: i64,
    pub created_at: i64,
}

impl ProductDoc {
    pub fn from_input(input: ProductInput, created_at: i64) -> Self {
        Self {
            id: None,
            name: input.name,
            description: input.description,
            price: input.price,
            category: input.category,
            image_url: input.image_url,
            stock: input.stock,
            created_at,
        }
    }

    pub fn into_dto(self) -> ProductDto {
        ProductDto {
            id: self.id.map(|i| i.to_hex()).unwrap_or_default(),
            name: self.name,
            description: self.description,
            price: self.price,
            category: self.category,
            image_url: self.image_url,
            stock: self.stock,
            created_at: self.created_at,
        }
    }
}
