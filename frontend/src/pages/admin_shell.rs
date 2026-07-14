use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api;
use crate::app::Route;

#[derive(Properties, PartialEq)]
pub struct AdminShellProps {
    pub active: Route,
    pub children: Children,
}

#[function_component(AdminShell)]
pub fn admin_shell(props: &AdminShellProps) -> Html {
    let navigator = use_navigator();

    let on_logout = {
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            let navigator = navigator.clone();
            spawn_local(async move {
                let _ = api::post_empty("/api/admin/logout").await;
                if let Some(nav) = navigator {
                    nav.push(&Route::AdminGate);
                }
            });
        })
    };

    html! {
        <div class="admin-shell">
            <aside class="admin-sidebar">
                <div class="admin-brand">
                    <span class="brand-green">{ "Green" }</span><span class="brand-gray">{ "IEM" }</span>
                    <span class="admin-tag">{ "Quản trị" }</span>
                </div>
                <nav>
                    <Link<Route>
                        to={Route::AdminProducts}
                        classes={classes!((props.active == Route::AdminProducts).then_some("active"))}
                    >{ "Sản phẩm" }</Link<Route>>
                    <Link<Route>
                        to={Route::AdminOrders}
                        classes={classes!((props.active == Route::AdminOrders).then_some("active"))}
                    >{ "Đơn hàng" }</Link<Route>>
                </nav>
                <button type="button" class="link-button" onclick={on_logout}>{ "Đăng xuất" }</button>
            </aside>
            <section class="admin-content">
                { for props.children.iter() }
            </section>
        </div>
    }
}
