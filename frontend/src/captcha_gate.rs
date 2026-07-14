use gloo_net::http::Request;
use shared::{CaptchaAnswer, CaptchaChallenge};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Default)]
struct CaptchaGateState {
    open: bool,
    challenge: Option<CaptchaChallenge>,
    error: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct CaptchaGateHandle(UseStateHandle<CaptchaGateState>);

impl CaptchaGateHandle {
    /// Opens the "prove you're not a bot" modal and fetches a fresh math challenge.
    /// Call this whenever an API call comes back with `ApiClientError::CaptchaRequired`.
    pub fn open(&self) {
        let state = self.0.clone();
        state.set(CaptchaGateState {
            open: true,
            challenge: None,
            error: None,
        });
        let state_for_task = self.0.clone();
        spawn_local(async move {
            if let Ok(resp) = Request::get("/api/captcha/challenge").send().await {
                if resp.ok() {
                    if let Ok(challenge) = resp.json::<CaptchaChallenge>().await {
                        state_for_task.set(CaptchaGateState {
                            open: true,
                            challenge: Some(challenge),
                            error: None,
                        });
                    }
                }
            }
        });
    }
}

#[derive(Properties, PartialEq)]
pub struct CaptchaGateProviderProps {
    pub children: Children,
}

#[function_component(CaptchaGateProvider)]
pub fn captcha_gate_provider(props: &CaptchaGateProviderProps) -> Html {
    let state = use_state(CaptchaGateState::default);
    let handle = CaptchaGateHandle(state.clone());

    let answer = use_state(String::new);
    let submitting = use_state(|| false);

    let on_input = {
        let answer = answer.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            answer.set(input.value());
        })
    };

    let on_submit = {
        let state = state.clone();
        let answer = answer.clone();
        let submitting = submitting.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let Some(challenge) = state.challenge.clone() else {
                return;
            };
            let state = state.clone();
            let answer_value = (*answer).clone();
            let submitting = submitting.clone();
            submitting.set(true);
            spawn_local(async move {
                let payload = CaptchaAnswer {
                    token: challenge.token.clone(),
                    answer: answer_value,
                };
                let result = Request::post("/api/captcha/verify").json(&payload);
                let result = match result {
                    Ok(builder) => builder.send().await,
                    Err(e) => {
                        submitting.set(false);
                        state.set(CaptchaGateState {
                            open: true,
                            challenge: Some(challenge),
                            error: Some(format!("Lỗi: {e}")),
                        });
                        return;
                    }
                };
                submitting.set(false);
                match result {
                    Ok(resp) if resp.ok() => {
                        state.set(CaptchaGateState::default());
                    }
                    Ok(_) => {
                        state.set(CaptchaGateState {
                            open: true,
                            challenge: Some(challenge),
                            error: Some("Sai đáp án, vui lòng thử lại.".to_string()),
                        });
                    }
                    Err(e) => {
                        state.set(CaptchaGateState {
                            open: true,
                            challenge: Some(challenge),
                            error: Some(format!("Lỗi kết nối: {e}")),
                        });
                    }
                }
            });
        })
    };

    html! {
        <ContextProvider<CaptchaGateHandle> context={handle}>
            { for props.children.iter() }
            if state.open {
                <div class="captcha-overlay">
                    <div class="captcha-modal">
                        <h3>{ "Xác minh bạn không phải robot" }</h3>
                        <p>{ "Hệ thống phát hiện có nhiều yêu cầu truy cập trong thời gian ngắn. Vui lòng trả lời câu hỏi bên dưới để tiếp tục sử dụng." }</p>
                        {
                            if let Some(challenge) = state.challenge.clone() {
                                html! {
                                    <form onsubmit={on_submit.clone()}>
                                        <label class="captcha-question">{ challenge.question }</label>
                                        <input
                                            type="text"
                                            inputmode="numeric"
                                            autocomplete="off"
                                            value={(*answer).clone()}
                                            oninput={on_input.clone()}
                                        />
                                        {
                                            if let Some(err) = &state.error {
                                                html! { <p class="error-text">{ err }</p> }
                                            } else {
                                                html! {}
                                            }
                                        }
                                        <button type="submit" disabled={*submitting}>
                                            { if *submitting { "Đang kiểm tra..." } else { "Xác nhận" } }
                                        </button>
                                    </form>
                                }
                            } else {
                                html! { <p>{ "Đang tải câu hỏi..." }</p> }
                            }
                        }
                    </div>
                </div>
            }
        </ContextProvider<CaptchaGateHandle>>
    }
}

#[hook]
pub fn use_captcha_gate() -> CaptchaGateHandle {
    use_context::<CaptchaGateHandle>().expect("CaptchaGateProvider not found in component tree")
}
