fn main() {
    if matches!(
        std::env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("android")
    ) {
        // Read gamepad events before NativeActivity's touch and keyboard adapter.
        println!("cargo:rustc-link-arg-cdylib=-Wl,--wrap=AInputQueue_getEvent");
    }
}
