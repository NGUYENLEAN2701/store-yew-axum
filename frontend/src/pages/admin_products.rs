use shared::{AdminMe, Category, Product, ProductInput};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::{self, ApiClientError};
use crate::app::Route;
use crate::captcha_gate::use_captcha_gate;
use crate::format::format_vnd;
use crate::pages::admin_shell::AdminShell;

fn category_from_value(value: &str) -> Category {
    match value {
        "dongle" => Category::Dongle,
        "accessory" => Category::Accessory,
        _ => Category::Iem,
    }
}

#[derive(Clone, PartialEq, Default)]
struct FormState {
    editing_id: Option<String>,
    name: String,
    description: String,
    price: String,
    category: Category,
    image_url: String,
    stock: String,
}

impl FormState {
    fn from_product(p: &Product) -> Self {
        Self {
            editing_id: Some(p.id.clone()),
            name: p.name.clone(),
            description: p.description.clone(),
            price: p.price.to_string(),
            category: p.category,
            image_url: p.image_url.clone(),
            stock: p.stock.to_string(),
        }
    }
}

#[function_component(AdminProducts)]
pub fn admin_products() -> Html {
    let navigator = use_navigator();
    let captcha_gate = use_captcha_gate();
    let products = use_state(|| None::<Vec<Product>>);
    let error = use_state(|| None::<String>);
    let status = use_state(|| None::<String>);
    let form = use_state(FormState::default);
    let submitting = use_state(|| false);

    let reload = {
        let products = products.clone();
        let error = error.clone();
        Callback::from(move |_: ()| {
            let products = products.clone();
            let error = error.clone();
            spawn_local(async move {
                match api::get_json::<Vec<Product>>("/api/products").await {
                    Ok(list) => products.set(Some(list)),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    {
        let navigator = navigator.clone();
        let reload = reload.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if api::get_json::<AdminMe>("/api/admin/me").await.is_err() {
                    if let Some(nav) = navigator {
                        nav.push(&Route::AdminGate);
                    }
                    return;
                }
                reload.emit(());
            });
            || ()
        });
    }

    macro_rules! field_cb {
        ($field:ident) => {{
            let form = form.clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let mut next = (*form).clone();
                next.$field = input.value();
                form.set(next);
            })
        }};
    }

    let on_name_input = field_cb!(name);
    let on_price_input = field_cb!(price);
    let on_image_url_input = field_cb!(image_url);
    let on_stock_input = field_cb!(stock);

    let on_description_input = {
        let form = form.clone();
        Callback::from(move |e: InputEvent| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            let mut next = (*form).clone();
            next.description = textarea.value();
            form.set(next);
        })
    };

    let on_category_change = {
        let form = form.clone();
        Callback::from(move |e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            let mut next = (*form).clone();
            next.category = category_from_value(&select.value());
            form.set(next);
        })
    };

    let on_cancel_edit = {
        let form = form.clone();
        Callback::from(move |_: MouseEvent| form.set(FormState::default()))
    };

    let on_submit = {
        let form = form.clone();
        let error = error.clone();
        let status = status.clone();
        let submitting = submitting.clone();
        let reload = reload.clone();
        let captcha_gate = captcha_gate.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let current = (*form).clone();

            let (Ok(price), Ok(stock)) = (
                current.price.trim().parse::<i64>(),
                current.stock.trim().parse::<i64>(),
            ) else {
                error.set(Some("Giá và tồn kho phải là số nguyên.".to_string()));
                return;
            };

            let input = ProductInput {
                name: current.name.clone(),
                description: current.description.clone(),
                price,
                category: current.category,
                image_url: current.image_url.clone(),
                stock,
            };

            let editing_id = current.editing_id.clone();
            let form = form.clone();
            let error = error.clone();
            let status = status.clone();
            let submitting = submitting.clone();
            let reload = reload.clone();
            let captcha_gate = captcha_gate.clone();

            submitting.set(true);
            spawn_local(async move {
                let result = match &editing_id {
                    Some(id) => {
                        api::put_json::<ProductInput, Product>(&format!("/api/admin/products/{id}"), &input)
                            .await
                    }
                    None => api::post_json::<ProductInput, Product>("/api/admin/products", &input).await,
                };
                submitting.set(false);
                match result {
                    Ok(_) => {
                        status.set(Some(if editing_id.is_some() {
                            "Đã cập nhật sản phẩm.".to_string()
                        } else {
                            "Đã thêm sản phẩm mới.".to_string()
                        }));
                        error.set(None);
                        form.set(FormState::default());
                        reload.emit(());
                    }
                    Err(ApiClientError::CaptchaRequired) => captcha_gate.open(),
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    let make_edit_cb = {
        let form = form.clone();
        move |p: Product| {
            let form = form.clone();
            Callback::from(move |_: MouseEvent| form.set(FormState::from_product(&p)))
        }
    };

    let make_delete_cb = {
        let error = error.clone();
        let status = status.clone();
        let reload = reload.clone();
        move |id: String| {
            let error = error.clone();
            let status = status.clone();
            let reload = reload.clone();
            Callback::from(move |_: MouseEvent| {
                let error = error.clone();
                let status = status.clone();
                let reload = reload.clone();
                let id = id.clone();
                spawn_local(async move {
                    match api::delete_empty(&format!("/api/admin/products/{id}")).await {
                        Ok(_) => {
                            status.set(Some("Đã xóa sản phẩm.".to_string()));
                            reload.emit(());
                        }
                        Err(e) => error.set(Some(e.to_string())),
                    }
                });
            })
        }
    };

    let is_editing = form.editing_id.is_some();

    html! {
        <AdminShell active={Route::AdminProducts}>
            <h1>{ "Quản lý sản phẩm" }</h1>

            <form class="admin-form" onsubmit={on_submit}>
                <h2>{ if is_editing { "Sửa sản phẩm" } else { "Thêm sản phẩm mới" } }</h2>
                <label for="name">{ "Tên sản phẩm" }</label>
                <input id="name" required=true value={form.name.clone()} oninput={on_name_input} />

                <label for="description">{ "Mô tả" }</label>
                <textarea id="description" value={form.description.clone()} oninput={on_description_input} />

                <div class="form-row">
                    <div>
                        <label for="price">{ "Giá (VND)" }</label>
                        <input id="price" required=true value={form.price.clone()} oninput={on_price_input} />
                    </div>
                    <div>
                        <label for="stock">{ "Tồn kho" }</label>
                        <input id="stock" required=true value={form.stock.clone()} oninput={on_stock_input} />
                    </div>
                </div>

                <label for="category">{ "Danh mục" }</label>
                <select id="category" onchange={on_category_change}>
                    <option value="iem" selected={form.category == Category::Iem}>{ "IEM" }</option>
                    <option value="dongle" selected={form.category == Category::Dongle}>{ "Dongle" }</option>
                    <option value="accessory" selected={form.category == Category::Accessory}>{ "Phụ kiện" }</option>
                </select>

                <label for="image_url">{ "URL ảnh sản phẩm" }</label>
                <input id="image_url" required=true value={form.image_url.clone()} oninput={on_image_url_input} />

                if let Some(err) = error.as_ref() {
                    <p class="error-text">{ err }</p>
                }
                if let Some(msg) = status.as_ref() {
                    <p class="success-text">{ msg }</p>
                }

                <div class="form-actions">
                    <button type="submit" class="btn-primary" disabled={*submitting}>
                        { if *submitting { "Đang lưu..." } else if is_editing { "Cập nhật" } else { "Thêm sản phẩm" } }
                    </button>
                    if is_editing {
                        <button type="button" class="btn-secondary" onclick={on_cancel_edit}>{ "Hủy" }</button>
                    }
                </div>
            </form>

            <h2>{ "Danh sách sản phẩm" }</h2>
            {
                match products.as_ref() {
                    None => html! { <p class="loading-text">{ "Đang tải..." }</p> },
                    Some(list) if list.is_empty() => html! { <p class="empty-state">{ "Chưa có sản phẩm." }</p> },
                    Some(list) => html! {
                        <table class="admin-table">
                            <thead>
                                <tr>
                                    <th>{ "Tên" }</th>
                                    <th>{ "Danh mục" }</th>
                                    <th>{ "Giá" }</th>
                                    <th>{ "Tồn kho" }</th>
                                    <th></th>
                                </tr>
                            </thead>
                            <tbody>
                                { for list.iter().cloned().map(|p| {
                                    let edit_cb = make_edit_cb(p.clone());
                                    let delete_cb = make_delete_cb(p.id.clone());
                                    html! {
                                        <tr>
                                            <td>{ &p.name }</td>
                                            <td>{ p.category.label() }</td>
                                            <td>{ format_vnd(p.price) }</td>
                                            <td>{ p.stock }</td>
                                            <td class="table-actions">
                                                <button type="button" class="link-button" onclick={edit_cb}>{ "Sửa" }</button>
                                                <button type="button" class="link-button danger" onclick={delete_cb}>{ "Xóa" }</button>
                                            </td>
                                        </tr>
                                    }
                                }) }
                            </tbody>
                        </table>
                    },
                }
            }
        </AdminShell>
    }
}
