#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub fn now() -> u32 {
    (js_sys::Date::now() / 1000.0) as u32
}

pub fn now() -> u32 {
    use std::time::SystemTime;

    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("shouldn't fail")
        .as_secs() as u32
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub fn now_sec_u64() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
pub fn now_sec_u64() -> u64 {
    use std::time::SystemTime;

    (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
        .expect("shouldn't fail")
        .as_secs()
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub fn now_ms_f64() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
pub fn now_ms_f64() -> f64 {
    use std::time::SystemTime;

    (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
        .expect("shouldn't fail")
        .as_secs_f64()
        * 1000.0
}

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub fn now_ms_u64() -> u64 {
    js_sys::Date::now() as u64
}

#[cfg(not(all(target_arch = "wasm32", feature = "web")))]
pub fn now_ms_u64() -> u64 {
    use std::time::SystemTime;

    let duration =
        (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)).expect("shouldn't fail");
    duration.as_secs() * 1000 + duration.subsec_millis() as u64
}
