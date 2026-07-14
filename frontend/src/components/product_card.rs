use shared::Product;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;
use crate::format::format_vnd;

#[derive(Properties, PartialEq)]
pub struct ProductCardProps {
    pub product: Product,
}

#[function_component(ProductCard)]
pub fn product_card(props: &ProductCardProps) -> Html {
    let product = &props.product;
    html! {
        <div class="product-card">
            <Link<Route> to={Route::ProductDetail { id: product.id.clone() }}>
                <img class="product-card-image" src={product.image_url.clone()} alt={product.name.clone()} loading="lazy" />
            </Link<Route>>
            <div class="product-card-body">
                <span class="product-card-category">{ product.category.label() }</span>
                <Link<Route> to={Route::ProductDetail { id: product.id.clone() }} classes="product-card-name">
                    { &product.name }
                </Link<Route>>
                <span class="product-card-price">{ format_vnd(product.price) }</span>
                {
                    if product.stock <= 0 {
                        html! { <span class="out-of-stock">{ "Hết hàng" }</span> }
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}
