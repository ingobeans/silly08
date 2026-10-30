mod engine;
mod libretro;
use libretro::*;

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {
    // engine::run_thread(cart_data)
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {
    // engine::run_thread(cart_data)
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_api_version() -> u32 {
    return RETRO_API_VERSION;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_controller_port_device(port: u32, device: u32) {
    println!("Plugging device {} into port {}.\n", device, port);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_system_info(info: *mut retro_system_info) {
    unsafe {
        (*info).library_name = "silly08".as_ptr();
        (*info).library_version = env!("CARGO_PKG_VERSION").as_ptr();
        (*info).need_fullpath = true;
        (*info).valid_extensions = "".as_ptr();
    }
}
