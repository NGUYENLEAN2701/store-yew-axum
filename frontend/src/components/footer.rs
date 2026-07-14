use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class="site-footer">
            <p>
                <span class="brand-green">{ "Green" }</span><span class="brand-gray">{ "IEM" }</span>
                { " — IEM, dongle & phụ kiện âm thanh." }
            </p>
            <p class="footer-note">{ "Thanh toán khi nhận hàng (COD) hoặc chuyển khoản ngân hàng." }</p>
        </footer>
    }
}
