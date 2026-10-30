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

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_system_av_info(info: *mut retro_system_av_info) {
    unsafe {
        (*info).geometry.base_width = 128;
        (*info).geometry.base_height = 128;
        (*info).geometry.max_width = 128;
        (*info).geometry.max_height = 128;
        (*info).geometry.aspect_ratio = 1.0;
    }
}
