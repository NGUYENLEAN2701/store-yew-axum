mod api;
mod app;
mod captcha_gate;
mod cart;
mod components;
mod format;
mod pages;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
