fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // do your stuff here
    log::info!("Hello, ego!");
    // "HELLO" bytecode
    let mut vm = self_vm::new(vec![1, 5, 3, 5, 0, 0, 0, 72, 69, 76, 76, 79, 2, 1, 0, 0, 0]);
    vm.run(&vec![]);
}
