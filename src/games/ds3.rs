use mem_rs::prelude::*;
use windows::Win32::UI::Input::XboxController::*;


use crate::games::shared::*;

use crate::utils::input::*;
use crate::utils::mem::*;


struct GamePointers {
    fps_patch: Pointer,
    fps_limit: Pointer,
    frame_advance: Pointer,
    frame_running: Pointer,
    xinput_patch: Pointer,
    xinput_state: Pointer,
    input_state: Pointer,
    save_active: Pointer,
    cutscene_3d: Pointer,
    cutscene_movie: Pointer,
    gamepad_index: Pointer,
    gamepad_flags: Pointer,
}

static mut POINTERS: Option<GamePointers> = None;


// Gamepad stuff
static mut GAMEPAD_INDEX_ORIG: i32 = 0;
static mut GAMEPAD_FLAGS_ORIG: u32 = 0;


pub unsafe fn ds3_init(process: &mut Process) -> GameFuncs
{
    // Refresh process
    process.refresh().expect("Failed to refresh process");


    // Inject DLLs
    let soulmods_module = inject_soulmods(process);
    let soulstas_patches_module = inject_soulstas_patches(process);

    // Get exports
    let soulmods_exports: Vec<ModuleExport> = get_exports(soulmods_module.unwrap());
    let soulstas_patches_exports: Vec<ModuleExport> = get_exports(soulstas_patches_module.unwrap());


    // Get all necessary memory pointers
    POINTERS = Some(GamePointers {
        fps_patch: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "DS3_FPS_PATCH_ENABLED").expect("Couldn't find DS3_FPS_PATCH_ENABLED").addr, vec![0]),
        fps_limit: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "DS3_FPS_CUSTOM_LIMIT").expect("Couldn't find DS3_FPS_CUSTOM_LIMIT").addr, vec![0]),
        frame_advance: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "DS3_FRAME_ADVANCE_ENABLED").expect("Couldn't find DS3_FRAME_ADVANCE_ENABLED").addr, vec![0]),
        frame_running: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "DS3_FRAME_RUNNING").expect("Couldn't find DS3_FRAME_RUNNING").addr, vec![0]),
        xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS3_XINPUT_PATCH_ENABLED").expect("Couldn't find DS3_XINPUT_PATCH_ENABLED").addr, vec![0]),
        xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS3_XINPUT_STATE").expect("Couldn't find DS3_XINPUT_STATE").addr, vec![0]),
        input_state: process.scan_rel("input_state", "48 8B 1D ? ? ? 04 48 8B F9 48 85 DB ? ? 8B 11 85 D2 ? ? 8D", 3, 7, vec![0, 0x80, 0x50, 0x180]).expect("Couldn't find input_state pointer"),
        save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? 48 8b 48 10 48 85 c9 74 08 0f b6 81 f4", 3, 7, vec![0, 0xd70]).expect("Couldn't find save_active pointer"),
        cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 05 ? ? ? ? 48 85 c0 74 37", 3, 7, vec![0, 0x14c]).expect("Couldn't find cutscene_3d pointer"),
        cutscene_movie: process.scan_rel("cutscene_movie", "48 8b 0d ? ? ? ? e8 ? ? ? ? 84 c0 74 07 c6 83 c8 00 00 00 01", 3, 7, vec![0, 0x15]).expect("Couldn't find cutscene_movie pointer"),
        gamepad_index: process.scan_rel("gamepad_index", "41 0f 28 c9 e8 ? ? ? ? 48 8b 0d", 12, 16, vec![0, 0x18, 0x10, 0x24c]).expect("Couldn't find gamepad_index pointer"),
        gamepad_flags: process.scan_rel("gamepad_flags", "41 0f 28 c9 e8 ? ? ? ? 48 8b 0d", 12, 16, vec![0, 0x18, 0x10, 0x2c4]).expect("Couldn't find gamepad_flags pointer"),
    });


    // Return all functions
    let game_funcs = GameFuncs {
        script_start: ds3_script_start,
        script_end: ds3_script_end,
        frame_next: ds3_frame_next,
        frame_start: ds3_frame_start,
        frame_end: ds3_frame_end,
        action_fps: ds3_action_fps,
        flag_frame: ds3_flag_frame,
        flag_control: ds3_flag_control,
        flag_cutscene: ds3_flag_cutscene,
        flag_save: ds3_flag_save
    };

    return game_funcs;
}

pub unsafe fn ds3_script_start(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    GAMEPAD_INDEX_ORIG = pointers.gamepad_index.read_i32_rel(None);
    GAMEPAD_FLAGS_ORIG = pointers.gamepad_flags.read_u32_rel(None);

    pointers.xinput_patch.write_u8_rel(None, 1);
}

pub unsafe fn ds3_script_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    pointers.xinput_patch.write_u8_rel(None, 0);

    pointers.gamepad_index.write_i32_rel(None, GAMEPAD_INDEX_ORIG);
    pointers.gamepad_flags.write_u32_rel(None, GAMEPAD_FLAGS_ORIG);
}

pub unsafe fn ds3_frame_next(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn ds3_frame_start(process: &mut Process)
{
}

pub unsafe fn ds3_frame_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    // Set correct gamepad flags
    pointers.gamepad_index.write_i32_rel(None, 999);
    pointers.gamepad_flags.write_u32_rel(None, 795);

    // Send gamepad input
    let xinput_state_override_buf = &*(&XINPUT_STATE_OVERRIDE as *const XINPUT_STATE as *const [u8; core::mem::size_of::<XINPUT_STATE>()]);
    pointers.xinput_state.write_memory_rel(None, xinput_state_override_buf);
}

pub unsafe fn ds3_action_fps(process: &mut Process, fps: f32)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.fps_limit.write_f32_rel(None, fps);
}

pub unsafe fn ds3_flag_frame(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn ds3_flag_control(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u32_rel(None);
    if input_state >> 1 & 1 == 1 && input_state >> 16 & 1 == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn ds3_flag_cutscene(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();

    if pointers.cutscene_3d.read_i8_rel(None) == -7 || pointers.cutscene_movie.read_bool_rel(None) {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn ds3_flag_save(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) != -1 {
        return true;
    } else {
        return false;
    }
}