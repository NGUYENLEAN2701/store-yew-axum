pub fn format_vnd(amount: i64) -> String {
    let negative = amount < 0;
    let digits = amount.unsigned_abs().to_string();

    let mut grouped = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(c);
    }
    let grouped: String = grouped.chars().rev().collect();

    if negative {
        format!("-{grouped}\u{20ab}")
    } else {
        format!("{grouped}\u{20ab}")
    }
}

pub fn format_timestamp(epoch_seconds: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((epoch_seconds * 1000) as f64));
    format!(
        "{} {}",
        String::from(date.to_locale_date_string("vi-VN", &wasm_bindgen::JsValue::UNDEFINED)),
        String::from(date.to_locale_time_string("vi-VN"))
    )
}
