use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;
use crate::cart::use_cart;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let cart = use_cart();
    let count = cart.item_count();

    html! {
        <header class="navbar">
            <div class="navbar-inner">
                <Link<Route> to={Route::Home} classes="brand">
                    <span class="brand-green">{ "Green" }</span><span class="brand-gray">{ "IEM" }</span>
                </Link<Route>>
                <nav class="nav-links">
                    <Link<Route> to={Route::Home}>{ "Sản phẩm" }</Link<Route>>
                    <Link<Route> to={Route::Checkout} classes="cart-link">
                        { "Giỏ hàng" }
                        if count > 0 {
                            <span class="cart-badge">{ count }</span>
                        }
                    </Link<Route>>
                </nav>
            </div>
        </header>
    }
}
