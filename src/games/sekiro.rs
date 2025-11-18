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


pub unsafe fn sekiro_init(process: &mut Process) -> GameFuncs
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
        fps_patch: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "SEKIRO_FPS_PATCH_ENABLED").expect("Couldn't find SEKIRO_FPS_PATCH_ENABLED").addr, vec![0]),
        fps_limit: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "SEKIRO_FPS_CUSTOM_LIMIT").expect("Couldn't find SEKIRO_FPS_CUSTOM_LIMIT").addr, vec![0]),
        frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "SEKIRO_FRAME_ADVANCE_ENABLED").expect("Couldn't find SEKIRO_FRAME_ADVANCE_ENABLED").addr, vec![0]),
        frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "SEKIRO_FRAME_RUNNING").expect("Couldn't find SEKIRO_FRAME_RUNNING").addr, vec![0]),
        xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "SEKIRO_XINPUT_PATCH_ENABLED").expect("Couldn't find SEKIRO_XINPUT_PATCH_ENABLED").addr, vec![0]),
        xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "SEKIRO_XINPUT_STATE").expect("Couldn't find SEKIRO_XINPUT_STATE").addr, vec![0]),
        input_state: process.scan_rel("input_state", "48 8B 35 ? ? ? ? 44 0F 28 18", 3, 7, vec![0, 0x88, 0x50, 0x190]).expect("Couldn't find input_state pointer"),
        save_active: process.scan_rel("save_active", "48 8b 15 ? ? ? ? 8b 44 24 28 f3 0f 10 44 24 30", 3, 7, vec![0, 0xbf4]).expect("Couldn't find save_active pointer"),
        cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 05 ? ? ? ? 4c 8b f9 48 8b 49 08", 3, 7, vec![0, 0xd4]).expect("Couldn't find cutscene_3d pointer"),
        cutscene_movie: process.scan_rel("cutscene_movie", "80 bf b8 0a 00 00 00 75 3f 48 8b 0d ? ? ? ? 48 85 c9 75 2e 48 8d 0d ? ? ? ? e8 ? ? ? ? 4c 8b c8 4c 8d 05 ? ? ? ? ba b1 00 00 00", 12, 16, vec![0, 0x20]).expect("Couldn't find cutscene_movie pointer"),
        gamepad_index: process.scan_rel("gamepad_index", "4c 8b 05 ? ? ? ? 48 8b f2 48 8b d9 4d 85 c0 75 2e", 3, 7, vec![0, 0x18, 0x10, 0x244]).expect("Couldn't find gamepad_index pointer"),
        gamepad_flags: process.scan_rel("gamepad_flags", "4c 8b 05 ? ? ? ? 48 8b f2 48 8b d9 4d 85 c0 75 2e", 3, 7, vec![0, 0x18, 0x10, 0x2bc]).expect("Couldn't find gamepad_flags pointer"),
    });


    // Return all functions
    let game_funcs = GameFuncs {
        script_start: sekiro_script_start,
        script_end: sekiro_script_end,
        frame_next: sekiro_frame_next,
        frame_start: sekiro_frame_start,
        frame_end: sekiro_frame_end,
        action_fps: sekiro_action_fps,
        flag_frame: sekiro_flag_frame,
        flag_control: sekiro_flag_control,
        flag_cutscene: sekiro_flag_cutscene,
        flag_save: sekiro_flag_save
    };

    return game_funcs;
}

pub unsafe fn sekiro_script_start(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    GAMEPAD_INDEX_ORIG = pointers.gamepad_index.read_i32_rel(None);
    GAMEPAD_FLAGS_ORIG = pointers.gamepad_flags.read_u32_rel(None);

    pointers.xinput_patch.write_u8_rel(None, 1);
}

pub unsafe fn sekiro_script_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    pointers.xinput_patch.write_u8_rel(None, 0);

    pointers.gamepad_index.write_i32_rel(None, GAMEPAD_INDEX_ORIG);
    pointers.gamepad_flags.write_u32_rel(None, GAMEPAD_FLAGS_ORIG);
}

pub unsafe fn sekiro_frame_next(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn sekiro_frame_start(process: &mut Process)
{
}

pub unsafe fn sekiro_frame_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    // Set correct gamepad flags
    pointers.gamepad_index.write_i32_rel(None, 999);
    pointers.gamepad_flags.write_u32_rel(None, 795);

    // Send gamepad input
    let xinput_state_override_buf = &*(&XINPUT_STATE_OVERRIDE as *const XINPUT_STATE as *const [u8; core::mem::size_of::<XINPUT_STATE>()]);
    pointers.xinput_state.write_memory_rel(None, xinput_state_override_buf);
}

pub unsafe fn sekiro_action_fps(process: &mut Process, fps: f32)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.fps_limit.write_f32_rel(None, fps);
}

pub unsafe fn sekiro_flag_frame(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn sekiro_flag_control(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u8_rel(None);
    if input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn sekiro_flag_cutscene(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();

    if pointers.cutscene_3d.read_i8_rel(None) == -7 || pointers.cutscene_movie.read_bool_rel(None) {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn sekiro_flag_save(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) != -1 {
        return true;
    } else {
        return false;
    }
}