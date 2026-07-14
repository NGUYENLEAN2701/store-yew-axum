use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <div class="empty-state">
            <h1>{ "404" }</h1>
            <p>{ "Trang bạn tìm không tồn tại." }</p>
            <Link<Route> to={Route::Home}>{ "Về trang chủ" }</Link<Route>>
        </div>
    }
}
