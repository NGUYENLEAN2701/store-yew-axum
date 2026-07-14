use shared::Product;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::app::Route;
use crate::cart::{use_cart, CartAction};
use crate::format::format_vnd;

#[derive(Properties, PartialEq)]
pub struct ProductDetailProps {
    pub id: String,
}

#[function_component(ProductDetail)]
pub fn product_detail(props: &ProductDetailProps) -> Html {
    let product = use_state(|| None::<Product>);
    let error = use_state(|| None::<String>);
    let quantity = use_state(|| 1i64);
    let added = use_state(|| false);
    let cart = use_cart();
    let navigator = use_navigator();

    {
        let product = product.clone();
        let error = error.clone();
        let id = props.id.clone();
        use_effect_with(id.clone(), move |id| {
            let id = id.clone();
            product.set(None);
            spawn_local(async move {
                match api::get_json::<Product>(&format!("/api/products/{id}")).await {
                    Ok(p) => product.set(Some(p)),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
            || ()
        });
    }

    let on_quantity_input = {
        let quantity = quantity.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<i64>() {
                quantity.set(value.max(1));
            }
        })
    };

    let on_add_to_cart = {
        let cart = cart.clone();
        let product = product.clone();
        let quantity = quantity.clone();
        let added = added.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(p) = product.as_ref() {
                cart.dispatch(CartAction::Add(p.clone(), *quantity));
                added.set(true);
            }
        })
    };

    let on_buy_now = {
        let cart = cart.clone();
        let product = product.clone();
        let quantity = quantity.clone();
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(p) = product.as_ref() {
                cart.dispatch(CartAction::Add(p.clone(), *quantity));
                if let Some(nav) = &navigator {
                    nav.push(&Route::Checkout);
                }
            }
        })
    };

    if let Some(err) = error.as_ref() {
        return html! { <p class="error-text">{ err }</p> };
    }

    let Some(p) = product.as_ref() else {
        return html! { <p class="loading-text">{ "Đang tải sản phẩm..." }</p> };
    };

    html! {
        <div class="product-detail">
            <img class="product-detail-image" src={p.image_url.clone()} alt={p.name.clone()} />
            <div class="product-detail-info">
                <span class="product-card-category">{ p.category.label() }</span>
                <h1>{ &p.name }</h1>
                <p class="product-detail-price">{ format_vnd(p.price) }</p>
                <p class="product-detail-description">{ &p.description }</p>
                <p class="product-detail-stock">
                    { if p.stock > 0 { format!("Còn {} sản phẩm", p.stock) } else { "Hết hàng".to_string() } }
                </p>
                <div class="quantity-row">
                    <label for="qty">{ "Số lượng" }</label>
                    <input id="qty" type="number" min="1" value={quantity.to_string()} oninput={on_quantity_input} />
                </div>
                <div class="product-detail-actions">
                    <button class="btn-secondary" onclick={on_add_to_cart} disabled={p.stock <= 0}>{ "Thêm vào giỏ" }</button>
                    <button class="btn-primary" onclick={on_buy_now} disabled={p.stock <= 0}>{ "Mua ngay" }</button>
                </div>
                if *added {
                    <p class="success-text">{ "Đã thêm vào giỏ hàng." }</p>
                }
            </div>
        </div>
    }
}
