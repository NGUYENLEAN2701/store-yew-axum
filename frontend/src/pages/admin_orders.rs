use shared::{AdminMe, Order, OrderStatus, UpdateOrderStatusInput};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::app::Route;
use crate::format::{format_timestamp, format_vnd};
use crate::pages::admin_shell::AdminShell;

fn status_from_value(value: &str) -> OrderStatus {
    match value {
        "confirmed" => OrderStatus::Confirmed,
        "shipped" => OrderStatus::Shipped,
        "completed" => OrderStatus::Completed,
        "cancelled" => OrderStatus::Cancelled,
        _ => OrderStatus::Pending,
    }
}

fn status_value(status: OrderStatus) -> &'static str {
    match status {
        OrderStatus::Pending => "pending",
        OrderStatus::Confirmed => "confirmed",
        OrderStatus::Shipped => "shipped",
        OrderStatus::Completed => "completed",
        OrderStatus::Cancelled => "cancelled",
    }
}

#[function_component(AdminOrders)]
pub fn admin_orders() -> Html {
    let navigator = use_navigator();
    let orders = use_state(|| None::<Vec<Order>>);
    let error = use_state(|| None::<String>);

    let reload = {
        let orders = orders.clone();
        let error = error.clone();
        Callback::from(move |_: ()| {
            let orders = orders.clone();
            let error = error.clone();
            spawn_local(async move {
                match api::get_json::<Vec<Order>>("/api/admin/orders").await {
                    Ok(list) => orders.set(Some(list)),
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

    let make_status_change_cb = {
        let error = error.clone();
        let reload = reload.clone();
        move |order_id: String| {
            let error = error.clone();
            let reload = reload.clone();
            Callback::from(move |e: Event| {
                let select: HtmlSelectElement = e.target_unchecked_into();
                let status = status_from_value(&select.value());
                let error = error.clone();
                let reload = reload.clone();
                let order_id = order_id.clone();
                spawn_local(async move {
                    let input = UpdateOrderStatusInput { status };
                    let result = api::put_json::<UpdateOrderStatusInput, Order>(
                        &format!("/api/admin/orders/{order_id}/status"),
                        &input,
                    )
                    .await;
                    match result {
                        Ok(_) => reload.emit(()),
                        Err(e) => error.set(Some(e.to_string())),
                    }
                });
            })
        }
    };

    html! {
        <AdminShell active={Route::AdminOrders}>
            <h1>{ "Quản lý đơn hàng" }</h1>
            if let Some(err) = error.as_ref() {
                <p class="error-text">{ err }</p>
            }
            {
                match orders.as_ref() {
                    None => html! { <p class="loading-text">{ "Đang tải..." }</p> },
                    Some(list) if list.is_empty() => html! { <p class="empty-state">{ "Chưa có đơn hàng nào." }</p> },
                    Some(list) => html! {
                        <table class="admin-table">
                            <thead>
                                <tr>
                                    <th>{ "Mã đơn" }</th>
                                    <th>{ "Khách hàng" }</th>
                                    <th>{ "SĐT" }</th>
                                    <th>{ "Sản phẩm" }</th>
                                    <th>{ "Tổng tiền" }</th>
                                    <th>{ "Thanh toán" }</th>
                                    <th>{ "Trạng thái" }</th>
                                    <th>{ "Thời gian" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for list.iter().map(|order| {
                                    let on_change = make_status_change_cb(order.id.clone());
                                    let items_summary = order
                                        .items
                                        .iter()
                                        .map(|i| format!("{} x{}", i.name, i.quantity))
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    html! {
                                        <tr>
                                            <td>{ &order.id[order.id.len().saturating_sub(8)..] }</td>
                                            <td>{ &order.customer_name }</td>
                                            <td>{ &order.phone }</td>
                                            <td class="order-items-cell">{ items_summary }</td>
                                            <td>{ format_vnd(order.total) }</td>
                                            <td>
                                                {
                                                    match order.payment_method {
                                                        shared::PaymentMethod::Cod => "COD",
                                                        shared::PaymentMethod::BankTransfer => "Chuyển khoản",
                                                    }
                                                }
                                            </td>
                                            <td>
                                                <select onchange={on_change}>
                                                    { for OrderStatus::all().iter().map(|s| html! {
                                                        <option value={status_value(*s)} selected={*s == order.status}>
                                                            { s.label() }
                                                        </option>
                                                    }) }
                                                </select>
                                            </td>
                                            <td>{ format_timestamp(order.created_at) }</td>
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
