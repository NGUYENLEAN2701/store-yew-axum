use shared::{Category, Product};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api;
use crate::components::ProductCard;

#[derive(Clone, Copy, PartialEq)]
enum Filter {
    All,
    Category(Category),
}

#[function_component(Home)]
pub fn home() -> Html {
    let products = use_state(|| None::<Vec<Product>>);
    let error = use_state(|| None::<String>);
    let filter = use_state(|| Filter::All);

    {
        let products = products.clone();
        let error = error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match api::get_json::<Vec<Product>>("/api/products").await {
                    Ok(list) => products.set(Some(list)),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
            || ()
        });
    }

    let make_filter_cb = |f: Filter| {
        let filter = filter.clone();
        Callback::from(move |_: MouseEvent| filter.set(f))
    };

    let filtered: Vec<Product> = products
        .as_ref()
        .map(|list| {
            list.iter()
                .filter(|p| match *filter {
                    Filter::All => true,
                    Filter::Category(c) => p.category == c,
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    html! {
        <div class="home-page">
            <section class="hero">
                <h1><span class="brand-green">{ "Green" }</span><span class="brand-gray">{ "IEM" }</span></h1>
                <p>{ "IEM, dongle DAC/AMP và phụ kiện âm thanh — chọn đúng gu nghe của bạn." }</p>
            </section>

            <div class="category-tabs">
                <button
                    class={classes!("tab", (*filter == Filter::All).then_some("active"))}
                    onclick={make_filter_cb(Filter::All)}
                >{ "Tất cả" }</button>
                <button
                    class={classes!("tab", (*filter == Filter::Category(Category::Iem)).then_some("active"))}
                    onclick={make_filter_cb(Filter::Category(Category::Iem))}
                >{ Category::Iem.label() }</button>
                <button
                    class={classes!("tab", (*filter == Filter::Category(Category::Dongle)).then_some("active"))}
                    onclick={make_filter_cb(Filter::Category(Category::Dongle))}
                >{ Category::Dongle.label() }</button>
                <button
                    class={classes!("tab", (*filter == Filter::Category(Category::Accessory)).then_some("active"))}
                    onclick={make_filter_cb(Filter::Category(Category::Accessory))}
                >{ Category::Accessory.label() }</button>
            </div>

            {
                if let Some(err) = error.as_ref() {
                    html! { <p class="error-text">{ err }</p> }
                } else if products.is_none() {
                    html! { <p class="loading-text">{ "Đang tải sản phẩm..." }</p> }
                } else if filtered.is_empty() {
                    html! { <p class="empty-state">{ "Chưa có sản phẩm trong danh mục này." }</p> }
                } else {
                    html! {
                        <div class="product-grid">
                            { for filtered.iter().map(|p| html! { <ProductCard product={p.clone()} /> }) }
                        </div>
                    }
                }
            }
        </div>
    }
}
