use yew::prelude::*;
use yew_router::prelude::*;

use crate::captcha_gate::CaptchaGateProvider;
use crate::cart::CartProvider;
use crate::components::{Footer, Navbar};
use crate::pages::{
    AdminGate, AdminOrders, AdminProducts, Checkout, Home, NotFound, ProductDetail,
};

#[derive(Clone, Routable, PartialEq, Debug)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/product/:id")]
    ProductDetail { id: String },
    #[at("/checkout")]
    Checkout,
    #[at("/console")]
    AdminGate,
    #[at("/console/products")]
    AdminProducts,
    #[at("/console/orders")]
    AdminOrders,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn is_admin_route(route: &Route) -> bool {
    matches!(
        route,
        Route::AdminGate | Route::AdminProducts | Route::AdminOrders
    )
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home /> },
        Route::ProductDetail { id } => html! { <ProductDetail id={id} /> },
        Route::Checkout => html! { <Checkout /> },
        Route::AdminGate => html! { <AdminGate /> },
        Route::AdminProducts => html! { <AdminProducts /> },
        Route::AdminOrders => html! { <AdminOrders /> },
        Route::NotFound => html! { <NotFound /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <CaptchaGateProvider>
                <CartProvider>
                    <Shell />
                </CartProvider>
            </CaptchaGateProvider>
        </BrowserRouter>
    }
}

#[function_component(Shell)]
fn shell() -> Html {
    let route = use_route::<Route>();
    let admin_area = route.as_ref().map(is_admin_route).unwrap_or(false);

    html! {
        <div class="page">
            if !admin_area {
                <Navbar />
            }
            <main class="page-content">
                <Switch<Route> render={switch} />
            </main>
            if !admin_area {
                <Footer />
            }
        </div>
    }
}
