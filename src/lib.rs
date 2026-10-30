mod engine;
mod libretro;
use std::{
    ffi::c_void,
    ptr::null,
    sync::{Mutex, mpsc::Receiver},
    thread::JoinHandle,
};

use deno_core::error::AnyError;
use libretro::*;

use crate::engine::Message;

static JOIN_HANDLE: Mutex<Option<JoinHandle<Result<(), AnyError>>>> = Mutex::new(None);
static MESSAGE_RX: Mutex<Option<Receiver<Message>>> = Mutex::new(None);

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

#[unsafe(no_mangle)]
pub extern "C" fn retro_load_game(info: *const retro_game_info) -> bool {
    let desc: &mut [retro_input_descriptor] = &mut [
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_LEFT,
            description: "Left".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_UP,
            description: "Up".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_DOWN,
            description: "Down".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: RETRO_DEVICE_JOYPAD,
            index: 0,
            id: RETRO_DEVICE_ID_JOYPAD_RIGHT,
            description: "Right".as_ptr(),
        },
        retro_input_descriptor {
            port: 0,
            device: 0,
            index: 0,
            id: 0,
            description: 0_usize as _,
        },
    ];

    let cb = ENVIRON_CB.lock().unwrap().unwrap();
    unsafe {
        cb(RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS, desc as *mut _ as _);
    }

    let mut fmt = retro_pixel_format::RETRO_PIXEL_FORMAT_RGB565;
    let res = unsafe { cb(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &mut fmt as *mut _ as _) };
    if !res {
        println!("Pixel format is not supported.");
        return false;
    }

    let mut retro_game_path = [0_u8; 4096];
    let mut index = 0;
    unsafe {
        let mut ptr = (*info).path;
        loop {
            if *ptr == 0 {
                break;
            }
            retro_game_path[index] = *ptr;
            index += 1;
            ptr = ptr.add(1);
        }
    }
    let path = String::from_utf8_lossy(&retro_game_path[..index]).to_string();
    println!("Got path: {path}");

    let Ok(data) = std::fs::read(path) else {
        return false;
    };
    let (handle, rx) = engine::run_thread(data);
    *JOIN_HANDLE.lock().unwrap() = Some(handle);
    *MESSAGE_RX.lock().unwrap() = Some(rx);

    //    struct retro_audio_callback audio_cb = { audio_callback, audio_set_state };
    //    use_audio_cb = ENVIRON_CB(RETRO_ENVIRONMENT_SET_AUDIO_CALLBACK, &audio_cb);

    //    check_variables();

    //    (void)info;
    return true;
}

// callback setters

static ENVIRON_CB: Mutex<Option<EnvironmentFn>> = Mutex::new(None);
static VIDEO_CB: Mutex<Option<VideoRefreshFn>> = Mutex::new(None);
static AUDIO_CB: Mutex<Option<AudioSampleFn>> = Mutex::new(None);
static AUDIO_BATCH_CB: Mutex<Option<AudioBatchFn>> = Mutex::new(None);
static INPUT_POLL_CB: Mutex<Option<InputPollFn>> = Mutex::new(None);
static INPUT_STATE_CB: Mutex<Option<InputStateFn>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_video_refresh(cb: VideoRefreshFn) {
    *VIDEO_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample(cb: AudioSampleFn) {
    *AUDIO_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_audio_sample_batch(cb: AudioBatchFn) {
    *AUDIO_BATCH_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_poll(cb: InputPollFn) {
    *INPUT_POLL_CB.lock().unwrap() = Some(cb);
}
#[unsafe(no_mangle)]
pub extern "C" fn retro_set_input_state(cb: InputStateFn) {
    *INPUT_STATE_CB.lock().unwrap() = Some(cb);
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_environment(cb: EnvironmentFn) {
    *ENVIRON_CB.lock().unwrap() = Some(cb);

    //    if (cb(RETRO_ENVIRONMENT_GET_LOG_INTERFACE, &logging))
    //       log_cb = logging.log;
    //    else
    //       log_cb = fallback_log;

    //    static const struct retro_controller_description controllers[] = {
    //       { "Nintendo DS", RETRO_DEVICE_SUBCLASS(RETRO_DEVICE_JOYPAD, 0) },
    //    };

    //    static const struct retro_controller_info ports[] = {
    //       { controllers, 1 },
    //       { NULL, 0 },
    //    };

    //    unsafe {
    //     cb(RETRO_ENVIRONMENT_SET_CONTROLLER_INFO, (void*)ports);
    //    }
}

// function skeletons

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unload_game() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_run() {}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize_size() -> usize {
    return 0;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize(_data_: *const c_void, _size: usize) -> bool {
    return false;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unserialize(_data_: *const c_void, _size: usize) -> bool {
    return false;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_data(_id: u32) -> *const c_void {
    return null();
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_size(_id: u32) -> usize {
    return 0;
}
