#![allow(dead_code, non_camel_case_types)]

// This file contains libretro bindings that i manually ported from C from:
// https://github.com/libretro/libretro-common/blob/master/include/libretro.h

use std::ffi::c_void;

pub const RETRO_API_VERSION: u32 = 1;
pub const RETRO_DEVICE_JOYPAD: u32 = 1;

// Stolen from [slot](https://github.com/BrandonKowalski/slot)
pub type EnvironmentFn = unsafe extern "C" fn(u32, *mut c_void) -> bool;
pub type VideoRefreshFn = unsafe extern "C" fn(*const c_void, u32, u32, usize);
pub type AudioSampleFn = unsafe extern "C" fn(i16, i16);
pub type AudioBatchFn = unsafe extern "C" fn(*const i16, usize) -> usize;
pub type InputPollFn = unsafe extern "C" fn();
pub type InputStateFn = unsafe extern "C" fn(u32, u32, u32, u32) -> i16;

#[repr(C)]
pub struct retro_input_descriptor {
    /* Associates given parameters with a description. */
    pub port: u32,
    pub device: u32,
    pub index: u32,
    pub id: u32,

    /**
    Human readable description for parameters.
    The pointer must remain valid until
    retro_unload_game() is called. */
    pub description: *const u8,
}
impl retro_input_descriptor {
    pub fn null() -> Self {
        Self {
            port: 0,
            device: 0,
            index: 0,
            id: 0,
            description: 0_usize as _,
        }
    }
}

#[repr(C)]
pub struct retro_game_info {
    /**
    Path to game, UTF-8 encoded.
    Sometimes used as a reference for building other paths.
    May be NULL if game was loaded from stdin or similar,
    but in this case some cores will be unable to load `data`.
    So, it is preferable to fabricate something here instead
    of passing NULL, which will help more cores to succeed.
    retro_system_info::need_fullpath requires
    that this path is valid. */
    pub path: *const u8,
    /**
    Memory buffer of loaded game. Will be NULL
    if need_fullpath was set. */
    pub data: *const c_void,
    /** Size of memory buffer. */
    pub size: usize,
    /** String of implementation specific meta-data. */
    pub meta: *const u8,
}

#[repr(C)]
pub struct retro_game_geometry {
    /**
    Nominal video width of game, in pixels.
    This will typically be the emulated platform's native video width
    (or its smallest, if the original hardware supports multiple resolutions).
         */
    pub base_width: u32,

    /**
    Nominal video height of game, in pixels.
    This will typically be the emulated platform's native video height
    (or its smallest, if the original hardware supports multiple resolutions).
         */
    pub base_height: u32,

    /**
    Maximum possible width of the game screen, in pixels.
    This will typically be the emulated platform's maximum video width.
    For cores that emulate platforms with multiple screens (such as the Nintendo DS),
    this should assume the core's widest possible screen layout (e.g. side-by-side).
    For cores that support upscaling the resolution,
    this should assume the highest supported scale factor is active.
         */
    pub max_width: u32,

    /**
    Maximum possible height of the game screen, in pixels.
    This will typically be the emulated platform's maximum video height.
    For cores that emulate platforms with multiple screens (such as the Nintendo DS),
    this should assume the core's tallest possible screen layout (e.g. vertical).
    For cores that support upscaling the resolution,
    this should assume the highest supported scale factor is active.
         */
    pub max_height: u32, /* Maximum possible height of game. */

    /**
    Nominal aspect ratio of game.
    If zero or less,
    an aspect ratio of <tt>base_width / base_height</tt> is assumed.

    @note A frontend may ignore this setting.
         */
    pub aspect_ratio: f32,
}

#[repr(C)]
pub struct retro_system_timing {
    /** Video output refresh rate, in frames per second. */
    pub fps: f32,

    /** The audio output sample rate, in Hz. */
    pub sample_rate: f32,
}

#[repr(C)]
pub struct retro_system_av_info {
    /** Parameters describing the size and shape of the video frame. */
    pub geometry: retro_game_geometry,

    /** Parameters describing the timing of the video and audio. */
    pub timing: retro_system_timing,
}

#[repr(C)]
pub struct retro_system_info {
    /**
    Descriptive name of the library.

    @note Should not contain any version numbers, etc.
         */
    pub library_name: *const u8,

    /**
    Descriptive version of the core.
         */
    pub library_version: *const u8,

    /**
    A pipe-delimited string list of file extensions that this core can load, e.g. "bin|rom|iso".
    Typically used by a frontend for filtering or core selection.
         */
    pub valid_extensions: *const u8,

    /** Libretro cores that need to have direct access to their content
    files, including cores which use the path of the content files to
    determine the paths of other files, should set need_fullpath to true.

    Cores should strive for setting need_fullpath to false,
    as it allows the frontend to perform patching, etc.

    If need_fullpath is true and retro_load_game() is called:
       - retro_game_info::path is guaranteed to have a valid path
       - retro_game_info::data and retro_game_info::size are invalid

    If need_fullpath is false and retro_load_game() is called:
       - retro_game_info::path may be NULL
       - retro_game_info::data and retro_game_info::size are guaranteed
         to be valid

    See also:
       - RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY
       - RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY
         */
    pub need_fullpath: bool,

    /* If true, the frontend is not allowed to extract any archives before
    loading the real content.
    Necessary for certain libretro implementations that load games
    from zipped archives. */
    pub block_extract: bool,
}

/**
 * The pixel format used for rendering.
 * @see RETRO_ENVIRONMENT_SET_PIXEL_FORMAT
 */
pub enum retro_pixel_format {
    /**
     * 0RGB1555, native endian.
     * Used as the default if \c RETRO_ENVIRONMENT_SET_PIXEL_FORMAT is not called.
     * The most significant bit must be set to 0.
     * @deprecated This format remains supported to maintain compatibility.
     * New code should use <tt>RETRO_PIXEL_FORMAT_RGB565</tt> instead.
     * @see RETRO_PIXEL_FORMAT_RGB565
     */
    RETRO_PIXEL_FORMAT_0RGB1555 = 0,

    /**
     * XRGB8888, native endian.
     * The most significant byte (the <tt>X</tt>) is ignored.
     */
    RETRO_PIXEL_FORMAT_XRGB8888 = 1,

    /**
     * RGB565, native endian.
     * This format is recommended if 16-bit pixels are desired,
     * as it is available on a variety of devices and APIs.
     */
    RETRO_PIXEL_FORMAT_RGB565 = 2,

    /**
     * XRGB2101010, native endian.
     * 32-bit packed: 2 ignored high bits followed by 10-bit R, G, B
     * (i.e. bits [29:20]=R, [19:10]=G, [9:0]=B; the top 2 bits are ignored).
     * Intended for cores that decode 10-bit-per-channel content (e.g. HDR10
     * sources) and want to pass it through without narrowing to 8 bits.
     *
     * A frontend is not required to render this natively: if the active video
     * driver does not support a 10-bit source surface, the frontend transparently
     * down-converts to XRGB8888, so a core may rely on SET_PIXEL_FORMAT accepting
     * this value but should not assume the display path is 10-bit end to end.
     */
    RETRO_PIXEL_FORMAT_XRGB2101010 = 3,

    /**
     * HDR10: PQ-encoded Rec.2020, 10 bits per channel, native endian.
     *
     * Bit layout is identical to #RETRO_PIXEL_FORMAT_XRGB2101010 -- 2 ignored
     * high bits then 10-bit R, G, B (bits [29:20]=R, [19:10]=G, [9:0]=B) --
     * but the *encoding* differs, and that is the whole point of a separate
     * value: the samples are SMPTE ST.2084 (PQ) over Rec.2020 primaries,
     * covering 0..10000 nits absolute, exactly as HDR10 video does.
     *
     * XRGB2101010 is 10-bit SDR: the frontend treats 1.0 as paper white and
     * cannot represent anything brighter, so a core has no way to make a
     * highlight exceed the SDR white level.  With this format the core
     * chooses absolute luminance per pixel, so specular highlights, muzzle
     * flashes, explosions and emissive surfaces can sit well above paper
     * white while the rest of the image stays where it was.
     *
     * A frontend that accepts this format MUST pass the samples through to an
     * HDR10 (PQ / Rec.2020) swapchain without re-encoding them: no inverse
     * tonemap, no Rec.709->Rec.2020 rotation, no paper-white scaling, since
     * the core has already applied all of it.  For scRGB output the frontend must instead decode PQ, rotate Rec.2020
     * to linear Rec.709 and scale to the platform reference white, without
     * a second creative curve. A frontend without either path must reject the format from SET_PIXEL_FORMAT rather than
     * silently down-converting -- PQ samples interpreted as SDR look badly
     * wrong, so the usual transparent narrowing is not safe here.  Cores
     * should therefore keep an SDR path and fall back when this is refused.
     *
     * Cores should query #RETRO_ENVIRONMENT_GET_HDR_METADATA to learn
     * the luminance the user considers "SDR white" and map their normal
     * output to it; content authored above that value is what produces the
     * HDR effect.
     */
    RETRO_PIXEL_FORMAT_HDR10_2101010 = 4,

    /**
     * FP16 linear scRGB with Rec.709/sRGB primaries and D65 white.
     *
     * Each pixel is four IEEE 754 binary16 samples in memory order R, G, B, A,
     * with each 16-bit sample in native byte order. Alpha is ignored; cores
     * should set it to 1.0. Each pixel occupies 8 bytes; pitch is in bytes.
     * RGB samples must be finite. Negative values and values above 1.0 are
     * valid and must not be clamped during upload or intermediate storage.
     *
     * RGB 1.0 represents 80 cd/m^2, independent of frontend paper white and
     * platform output units. There is no gamma/PQ transfer to decode. Cores
     * can use GET_HDR_METADATA paper_white_nits / 80.0 for their SDR white.
     *
     * An accepting frontend must preserve linear HDR values, converting
     * primaries, transfer and reference-white units on the GPU as required
     * by its output. Do not apply SDR expansion or a second paper-white
     * scale. Unsupported paths must reject SET_PIXEL_FORMAT; cores must
     * check its result and retain an SDR fallback. The corresponding
     * source_formats bit is RETRO_HDR_SOURCE_FORMAT_SCRGB_RGBA16F.
     */
    RETRO_PIXEL_FORMAT_SCRGB_RGBA16F = 5,

    /**
     * @private Defined to ensure that <tt>sizeof(retro_pixel_format) == sizeof(int)</tt>.
     * Do not use.
     */
    RETRO_PIXEL_FORMAT_UNKNOWN = i32::MAX as _,
}

/**
 * Sets the internal pixel format used by the frontend for rendering.
 * The default pixel format is \c RETRO_PIXEL_FORMAT_0RGB1555 for compatibility reasons,
 * although it's considered deprecated and shouldn't be used by new code.
 *
 * @param[in] data <tt>const enum retro_pixel_format *</tt>.
 * Pointer to the pixel format to use.
 * @returns \c true if the pixel format was set successfully,
 * \c false if it's not supported or this callback is unavailable.
 * @note This function should be called inside \c retro_load_game()
 * or <tt>retro_get_system_av_info()</tt>.
 * @see retro_pixel_format
 */
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: u32 = 10;

/**
 * Sets an array of input descriptors for the frontend
 * to present to the user for configuring the core's controls.
 *
 * This function can be called at any time,
 * preferably early in the core's life cycle.
 * Ideally, no later than \c retro_load_game().
 *
 * @param[in] data <tt>const struct retro_input_descriptor *</tt>.
 * An array of input descriptors terminated by one whose
 * \c retro_input_descriptor::description field is set to \c NULL.
 * Behavior is undefined if \c NULL.
 * @return \c true if the environment call is recognized.
 * @see retro_input_descriptor
 */
pub const RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS: u32 = 11;

/** The equivalent of the SNES controller's south face button. */
pub const RETRO_DEVICE_ID_JOYPAD_B: u32 = 0;

/** The equivalent of the SNES controller's west face button. */
pub const RETRO_DEVICE_ID_JOYPAD_Y: u32 = 1;

/** The equivalent of the SNES controller's left-center button. */
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: u32 = 2;

/** The equivalent of the SNES controller's right-center button. */
pub const RETRO_DEVICE_ID_JOYPAD_START: u32 = 3;

/** Up on the RetroPad's D-pad. */
pub const RETRO_DEVICE_ID_JOYPAD_UP: u32 = 4;

/** Down on the RetroPad's D-pad. */
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: u32 = 5;

/** Left on the RetroPad's D-pad. */
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: u32 = 6;

/** Right on the RetroPad's D-pad. */
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: u32 = 7;

/** The equivalent of the SNES controller's east face button. */
pub const RETRO_DEVICE_ID_JOYPAD_A: u32 = 8;

/** The equivalent of the SNES controller's north face button. */
pub const RETRO_DEVICE_ID_JOYPAD_X: u32 = 9;

/** The equivalent of the SNES controller's left shoulder button. */
pub const RETRO_DEVICE_ID_JOYPAD_L: u32 = 10;

/** The equivalent of the SNES controller's right shoulder button. */
pub const RETRO_DEVICE_ID_JOYPAD_R: u32 = 11;

/** The equivalent of the PlayStation's rear left shoulder button. */
pub const RETRO_DEVICE_ID_JOYPAD_L2: u32 = 12;

/** The equivalent of the PlayStation's rear right shoulder button. */
pub const RETRO_DEVICE_ID_JOYPAD_R2: u32 = 13;

/**
 * The equivalent of the PlayStation's left analog stick button,
 * although the actual button need not be in this position.
 */
pub const RETRO_DEVICE_ID_JOYPAD_L3: u32 = 14;

/**
 * The equivalent of the PlayStation's right analog stick button,
 * although the actual button need not be in this position.
 */
pub const RETRO_DEVICE_ID_JOYPAD_R3: u32 = 15;

/**
 * Represents a bitmask that describes the state of all \c RETRO_DEVICE_ID_JOYPAD button constants,
 * rather than the state of a single button.
 *
 * @see RETRO_ENVIRONMENT_GET_INPUT_BITMASKS
 * @see RETRO_DEVICE_JOYPAD
 */
pub const RETRO_DEVICE_ID_JOYPAD_MASK: u32 = 256;

/* Index / Id values for ANALOG device. */
pub const RETRO_DEVICE_INDEX_ANALOG_LEFT: u32 = 0;
pub const RETRO_DEVICE_INDEX_ANALOG_RIGHT: u32 = 1;
pub const RETRO_DEVICE_INDEX_ANALOG_BUTTON: u32 = 2;
pub const RETRO_DEVICE_ID_ANALOG_X: u32 = 0;
pub const RETRO_DEVICE_ID_ANALOG_Y: u32 = 1;

/* Id values for MOUSE. */
pub const RETRO_DEVICE_ID_MOUSE_X: u32 = 0;
pub const RETRO_DEVICE_ID_MOUSE_Y: u32 = 1;
pub const RETRO_DEVICE_ID_MOUSE_LEFT: u32 = 2;
pub const RETRO_DEVICE_ID_MOUSE_RIGHT: u32 = 3;
pub const RETRO_DEVICE_ID_MOUSE_WHEELUP: u32 = 4;
pub const RETRO_DEVICE_ID_MOUSE_WHEELDOWN: u32 = 5;
pub const RETRO_DEVICE_ID_MOUSE_MIDDLE: u32 = 6;
pub const RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELUP: u32 = 7;
pub const RETRO_DEVICE_ID_MOUSE_HORIZ_WHEELDOWN: u32 = 8;
pub const RETRO_DEVICE_ID_MOUSE_BUTTON_4: u32 = 9;
pub const RETRO_DEVICE_ID_MOUSE_BUTTON_5: u32 = 10;
