use std::rc::Rc;

use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use shared::Product;
use yew::prelude::*;

const CART_KEY: &str = "greeniem_cart";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartLine {
    pub product_id: String,
    pub name: String,
    pub price: i64,
    pub image_url: String,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CartState {
    pub lines: Vec<CartLine>,
}

impl CartState {
    pub fn total(&self) -> i64 {
        self.lines.iter().map(|l| l.price * l.quantity).sum()
    }

    pub fn item_count(&self) -> i64 {
        self.lines.iter().map(|l| l.quantity).sum()
    }
}

pub enum CartAction {
    Add(Product, i64),
    SetQuantity(String, i64),
    Remove(String),
    Clear,
    Load(CartState),
}

impl Reducible for CartState {
    type Action = CartAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CartAction::Load(state) => return Rc::new(state),
            _ => {}
        }

        let mut lines = self.lines.clone();
        match action {
            CartAction::Add(product, qty) => {
                if let Some(line) = lines.iter_mut().find(|l| l.product_id == product.id) {
                    line.quantity += qty;
                } else {
                    lines.push(CartLine {
                        product_id: product.id,
                        name: product.name,
                        price: product.price,
                        image_url: product.image_url,
                        quantity: qty,
                    });
                }
            }
            CartAction::SetQuantity(id, qty) => {
                if qty <= 0 {
                    lines.retain(|l| l.product_id != id);
                } else if let Some(line) = lines.iter_mut().find(|l| l.product_id == id) {
                    line.quantity = qty;
                }
            }
            CartAction::Remove(id) => lines.retain(|l| l.product_id != id),
            CartAction::Clear => lines.clear(),
            CartAction::Load(_) => unreachable!("handled above"),
        }
        Rc::new(CartState { lines })
    }
}

pub type CartHandle = UseReducerHandle<CartState>;

#[derive(Properties, PartialEq)]
pub struct CartProviderProps {
    pub children: Children,
}

#[function_component(CartProvider)]
pub fn cart_provider(props: &CartProviderProps) -> Html {
    let cart = use_reducer(CartState::default);

    {
        let cart = cart.clone();
        use_effect_with((), move |_| {
            if let Ok(loaded) = LocalStorage::get::<CartState>(CART_KEY) {
                cart.dispatch(CartAction::Load(loaded));
            }
            || ()
        });
    }

    {
        let cart_state = (*cart).clone();
        use_effect_with(cart_state, move |state| {
            let _ = LocalStorage::set(CART_KEY, state);
            || ()
        });
    }

    html! {
        <ContextProvider<CartHandle> context={cart}>
            { for props.children.iter() }
        </ContextProvider<CartHandle>>
    }
}

#[hook]
pub fn use_cart() -> CartHandle {
    use_context::<CartHandle>().expect("CartProvider not found in component tree")
}
