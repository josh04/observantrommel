const LOG_ENABLED: bool = true;

pub fn log_s(msg: String) {
    log(msg.as_str());
}

pub fn log(msg: &str) {
    if LOG_ENABLED {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        web_sys::console::log_1(&msg.to_string().into());
        
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        println!("{}", msg);
    }
}

