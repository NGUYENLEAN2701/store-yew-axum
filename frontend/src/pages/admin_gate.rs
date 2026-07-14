use shared::{AdminCredentials, AdminMe, SetupStatus};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::{self, ApiClientError};
use crate::app::Route;
use crate::captcha_gate::use_captcha_gate;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Loading,
    Setup,
    Login,
}

#[function_component(AdminGate)]
pub fn admin_gate() -> Html {
    let mode = use_state(|| Mode::Loading);
    let username = use_state(String::new);
    let password = use_state(String::new);
    let confirm_password = use_state(String::new);
    let error = use_state(|| None::<String>);
    let submitting = use_state(|| false);
    let captcha_gate = use_captcha_gate();
    let navigator = use_navigator();

    {
        let mode = mode.clone();
        let navigator = navigator.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if api::get_json::<AdminMe>("/api/admin/me").await.is_ok() {
                    if let Some(nav) = navigator {
                        nav.push(&Route::AdminProducts);
                    }
                    return;
                }
                match api::get_json::<SetupStatus>("/api/setup/status").await {
                    Ok(status) if status.needs_setup => mode.set(Mode::Setup),
                    _ => mode.set(Mode::Login),
                }
            });
            || ()
        });
    }

    let on_username_input = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };
    let on_password_input = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };
    let on_confirm_input = {
        let confirm_password = confirm_password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            confirm_password.set(input.value());
        })
    };

    let on_submit = {
        let mode = mode.clone();
        let username = username.clone();
        let password = password.clone();
        let confirm_password = confirm_password.clone();
        let error = error.clone();
        let submitting = submitting.clone();
        let captcha_gate = captcha_gate.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let current_mode = *mode;

            if current_mode == Mode::Setup && *password != *confirm_password {
                error.set(Some("Mật khẩu xác nhận không khớp.".to_string()));
                return;
            }

            let credentials = AdminCredentials {
                username: (*username).clone(),
                password: (*password).clone(),
            };
            let mode = mode.clone();
            let error = error.clone();
            let submitting = submitting.clone();
            let captcha_gate = captcha_gate.clone();
            let navigator = navigator.clone();

            submitting.set(true);
            spawn_local(async move {
                let path = if current_mode == Mode::Setup {
                    "/api/admin/setup"
                } else {
                    "/api/admin/login"
                };
                let result = api::post_json::<AdminCredentials, AdminMe>(path, &credentials).await;
                submitting.set(false);
                match result {
                    Ok(_) => {
                        if let Some(nav) = navigator {
                            nav.push(&Route::AdminProducts);
                        }
                    }
                    Err(ApiClientError::CaptchaRequired) => {
                        captcha_gate.open();
                        error.set(Some(
                            "Vui lòng xác minh captcha rồi thử lại.".to_string(),
                        ));
                    }
                    Err(e) => {
                        error.set(Some(e.to_string()));
                        // If setup already happened elsewhere in the meantime, fall back to login.
                        if current_mode == Mode::Setup {
                            mode.set(Mode::Login);
                        }
                    }
                }
            });
        })
    };

    match *mode {
        Mode::Loading => html! { <p class="loading-text">{ "Đang kiểm tra..." }</p> },
        Mode::Setup => html! {
            <div class="admin-gate">
                <h1>{ "Khởi tạo tài khoản quản trị" }</h1>
                <p>{ "Chưa có tài khoản admin nào. Hãy tạo tài khoản đầu tiên để quản lý GreenIEM." }</p>
                <form onsubmit={on_submit}>
                    <label for="username">{ "Tên đăng nhập" }</label>
                    <input id="username" required=true value={(*username).clone()} oninput={on_username_input} />
                    <label for="password">{ "Mật khẩu (tối thiểu 8 ký tự)" }</label>
                    <input id="password" type="password" required=true value={(*password).clone()} oninput={on_password_input} />
                    <label for="confirm_password">{ "Xác nhận mật khẩu" }</label>
                    <input id="confirm_password" type="password" required=true value={(*confirm_password).clone()} oninput={on_confirm_input} />
                    if let Some(err) = error.as_ref() {
                        <p class="error-text">{ err }</p>
                    }
                    <button type="submit" class="btn-primary" disabled={*submitting}>
                        { if *submitting { "Đang tạo..." } else { "Tạo tài khoản" } }
                    </button>
                </form>
            </div>
        },
        Mode::Login => html! {
            <div class="admin-gate">
                <h1>{ "Đăng nhập quản trị" }</h1>
                <form onsubmit={on_submit}>
                    <label for="username">{ "Tên đăng nhập" }</label>
                    <input id="username" required=true value={(*username).clone()} oninput={on_username_input} />
                    <label for="password">{ "Mật khẩu" }</label>
                    <input id="password" type="password" required=true value={(*password).clone()} oninput={on_password_input} />
                    if let Some(err) = error.as_ref() {
                        <p class="error-text">{ err }</p>
                    }
                    <button type="submit" class="btn-primary" disabled={*submitting}>
                        { if *submitting { "Đang đăng nhập..." } else { "Đăng nhập" } }
                    </button>
                </form>
            </div>
        },
    }
}
