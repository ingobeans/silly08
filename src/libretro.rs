// https://github.com/libretro/libretro-common/blob/d9597c359a5ca8399a4e244f816e7e975a92a2aa/include/libretro.h#L97
pub const RETRO_API_VERSION: u32 = 1;

#[repr(C)]
pub struct retro_game_geometry {
    /**
     * Nominal video width of game, in pixels.
     * This will typically be the emulated platform's native video width
     * (or its smallest, if the original hardware supports multiple resolutions).
     */
    pub base_width: u32,

    /**
     * Nominal video height of game, in pixels.
     * This will typically be the emulated platform's native video height
     * (or its smallest, if the original hardware supports multiple resolutions).
     */
    pub base_height: u32,

    /**
     * Maximum possible width of the game screen, in pixels.
     * This will typically be the emulated platform's maximum video width.
     * For cores that emulate platforms with multiple screens (such as the Nintendo DS),
     * this should assume the core's widest possible screen layout (e.g. side-by-side).
     * For cores that support upscaling the resolution,
     * this should assume the highest supported scale factor is active.
     */
    pub max_width: u32,

    /**
     * Maximum possible height of the game screen, in pixels.
     * This will typically be the emulated platform's maximum video height.
     * For cores that emulate platforms with multiple screens (such as the Nintendo DS),
     * this should assume the core's tallest possible screen layout (e.g. vertical).
     * For cores that support upscaling the resolution,
     * this should assume the highest supported scale factor is active.
     */
    pub max_height: u32, /* Maximum possible height of game. */

    /**
     * Nominal aspect ratio of game.
     * If zero or less,
     * an aspect ratio of <tt>base_width / base_height</tt> is assumed.
     *
     * @note A frontend may ignore this setting.
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
     * Descriptive name of the library.
     *
     * @note Should not contain any version numbers, etc.
     */
    pub library_name: *const u8,

    /**
     * Descriptive version of the core.
     */
    pub library_version: *const u8,

    /**
     * A pipe-delimited string list of file extensions that this core can load, e.g. "bin|rom|iso".
     * Typically used by a frontend for filtering or core selection.
     */
    pub valid_extensions: *const u8,

    /* Libretro cores that need to have direct access to their content
     * files, including cores which use the path of the content files to
     * determine the paths of other files, should set need_fullpath to true.
     *
     * Cores should strive for setting need_fullpath to false,
     * as it allows the frontend to perform patching, etc.
     *
     * If need_fullpath is true and retro_load_game() is called:
     *    - retro_game_info::path is guaranteed to have a valid path
     *    - retro_game_info::data and retro_game_info::size are invalid
     *
     * If need_fullpath is false and retro_load_game() is called:
     *    - retro_game_info::path may be NULL
     *    - retro_game_info::data and retro_game_info::size are guaranteed
     *      to be valid
     *
     * See also:
     *    - RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY
     *    - RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY
     */
    pub need_fullpath: bool,

    /* If true, the frontend is not allowed to extract any archives before
     * loading the real content.
     * Necessary for certain libretro implementations that load games
     * from zipped archives. */
    pub block_extract: bool,
}
