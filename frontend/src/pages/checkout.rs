use shared::{Order, OrderInput, OrderItem, PaymentMethod};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::api::{self, ApiClientError};
use crate::captcha_gate::use_captcha_gate;
use crate::cart::{use_cart, CartAction};
use crate::format::format_vnd;

#[function_component(Checkout)]
pub fn checkout() -> Html {
    let cart = use_cart();
    let captcha_gate = use_captcha_gate();

    let customer_name = use_state(String::new);
    let phone = use_state(String::new);
    let address = use_state(String::new);
    let note = use_state(String::new);
    let payment_method = use_state(|| PaymentMethod::Cod);
    let submitting = use_state(|| false);
    let error = use_state(|| None::<String>);
    let placed_order = use_state(|| None::<Order>);

    macro_rules! text_input_cb {
        ($state:ident) => {{
            let state = $state.clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                state.set(input.value());
            })
        }};
    }

    let on_name_input = text_input_cb!(customer_name);
    let on_phone_input = text_input_cb!(phone);
    let on_address_input = text_input_cb!(address);

    let on_note_input = {
        let note = note.clone();
        Callback::from(move |e: InputEvent| {
            let textarea: HtmlTextAreaElement = e.target_unchecked_into();
            note.set(textarea.value());
        })
    };

    let on_payment_cod = {
        let payment_method = payment_method.clone();
        Callback::from(move |_: MouseEvent| payment_method.set(PaymentMethod::Cod))
    };
    let on_payment_bank = {
        let payment_method = payment_method.clone();
        Callback::from(move |_: MouseEvent| payment_method.set(PaymentMethod::BankTransfer))
    };

    let on_submit = {
        let cart = cart.clone();
        let captcha_gate = captcha_gate.clone();
        let customer_name = customer_name.clone();
        let phone = phone.clone();
        let address = address.clone();
        let note = note.clone();
        let payment_method = payment_method.clone();
        let submitting = submitting.clone();
        let error = error.clone();
        let placed_order = placed_order.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if cart.lines.is_empty() {
                error.set(Some("Giỏ hàng đang trống.".to_string()));
                return;
            }

            let items: Vec<OrderItem> = cart
                .lines
                .iter()
                .map(|l| OrderItem {
                    product_id: l.product_id.clone(),
                    name: l.name.clone(),
                    price: l.price,
                    quantity: l.quantity,
                })
                .collect();

            let input = OrderInput {
                customer_name: (*customer_name).clone(),
                phone: (*phone).clone(),
                address: (*address).clone(),
                note: (*note).clone(),
                items,
                payment_method: *payment_method,
            };

            let cart = cart.clone();
            let captcha_gate = captcha_gate.clone();
            let submitting = submitting.clone();
            let error = error.clone();
            let placed_order = placed_order.clone();

            submitting.set(true);
            spawn_local(async move {
                let result = api::post_json::<OrderInput, Order>("/api/orders", &input).await;
                submitting.set(false);
                match result {
                    Ok(order) => {
                        cart.dispatch(CartAction::Clear);
                        placed_order.set(Some(order));
                        error.set(None);
                    }
                    Err(ApiClientError::CaptchaRequired) => {
                        captcha_gate.open();
                        error.set(Some(
                            "Vui lòng xác minh captcha rồi bấm Đặt hàng lại.".to_string(),
                        ));
                    }
                    Err(e) => error.set(Some(e.to_string())),
                }
            });
        })
    };

    if let Some(order) = placed_order.as_ref() {
        return html! {
            <div class="checkout-success">
                <h1>{ "Đặt hàng thành công!" }</h1>
                <p>{ format!("Mã đơn hàng: {}", order.id) }</p>
                <p>{ format!("Tổng tiền: {}", format_vnd(order.total)) }</p>
                {
                    match order.payment_method {
                        PaymentMethod::Cod => html! {
                            <p>{ "Bạn đã chọn thanh toán khi nhận hàng (COD). GreenIEM sẽ liên hệ để xác nhận và giao hàng." }</p>
                        },
                        PaymentMethod::BankTransfer => html! {
                            <div class="bank-info">
                                <p>{ "Vui lòng chuyển khoản theo thông tin sau và ghi chú mã đơn hàng:" }</p>
                                <ul>
                                    <li>{ "Ngân hàng: Vietcombank" }</li>
                                    <li>{ "Số tài khoản: 0123456789" }</li>
                                    <li>{ "Chủ tài khoản: CONG TY GREENIEM" }</li>
                                    <li>{ format!("Nội dung chuyển khoản: {}", order.id) }</li>
                                </ul>
                            </div>
                        },
                    }
                }
            </div>
        };
    }

    html! {
        <div class="checkout-page">
            <div class="checkout-summary">
                <h2>{ "Giỏ hàng" }</h2>
                {
                    if cart.lines.is_empty() {
                        html! { <p class="empty-state">{ "Giỏ hàng của bạn đang trống." }</p> }
                    } else {
                        html! {
                            <>
                                <ul class="cart-lines">
                                    { for cart.lines.iter().map(|line| {
                                        let id = line.product_id.clone();
                                        let cart_remove = cart.clone();
                                        let on_remove = Callback::from(move |_: MouseEvent| {
                                            cart_remove.dispatch(CartAction::Remove(id.clone()));
                                        });
                                        let id = line.product_id.clone();
                                        let cart_qty = cart.clone();
                                        let on_qty_input = Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            if let Ok(qty) = input.value().parse::<i64>() {
                                                cart_qty.dispatch(CartAction::SetQuantity(id.clone(), qty));
                                            }
                                        });
                                        html! {
                                            <li class="cart-line">
                                                <img src={line.image_url.clone()} alt={line.name.clone()} />
                                                <div class="cart-line-info">
                                                    <span>{ &line.name }</span>
                                                    <span>{ format_vnd(line.price) }</span>
                                                </div>
                                                <input
                                                    type="number"
                                                    min="1"
                                                    class="cart-line-qty"
                                                    value={line.quantity.to_string()}
                                                    oninput={on_qty_input}
                                                />
                                                <button type="button" class="link-button" onclick={on_remove}>{ "Xóa" }</button>
                                            </li>
                                        }
                                    }) }
                                </ul>
                                <p class="checkout-total">{ format!("Tổng cộng: {}", format_vnd(cart.total())) }</p>
                            </>
                        }
                    }
                }
            </div>

            <form class="checkout-form" onsubmit={on_submit}>
                <h2>{ "Thông tin giao hàng" }</h2>
                <label for="customer_name">{ "Họ và tên" }</label>
                <input id="customer_name" required=true value={(*customer_name).clone()} oninput={on_name_input} />

                <label for="phone">{ "Số điện thoại" }</label>
                <input id="phone" required=true value={(*phone).clone()} oninput={on_phone_input} />

                <label for="address">{ "Địa chỉ giao hàng" }</label>
                <input id="address" required=true value={(*address).clone()} oninput={on_address_input} />

                <label for="note">{ "Ghi chú (tùy chọn)" }</label>
                <textarea id="note" value={(*note).clone()} oninput={on_note_input} />

                <h2>{ "Phương thức thanh toán" }</h2>
                <div class="payment-options">
                    <button
                        type="button"
                        class={classes!("payment-option", (*payment_method == PaymentMethod::Cod).then_some("active"))}
                        onclick={on_payment_cod}
                    >{ "Thanh toán khi nhận hàng (COD)" }</button>
                    <button
                        type="button"
                        class={classes!("payment-option", (*payment_method == PaymentMethod::BankTransfer).then_some("active"))}
                        onclick={on_payment_bank}
                    >{ "Chuyển khoản ngân hàng" }</button>
                </div>

                if let Some(err) = error.as_ref() {
                    <p class="error-text">{ err }</p>
                }

                <button type="submit" class="btn-primary" disabled={*submitting || cart.lines.is_empty()}>
                    { if *submitting { "Đang xử lý..." } else { "Đặt hàng" } }
                </button>
            </form>
        </div>
    }
}
