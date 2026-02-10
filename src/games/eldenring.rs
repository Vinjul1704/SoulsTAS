use mem_rs::prelude::*;
use std::path::PathBuf;
use windows::Win32::UI::Input::XboxController::*;

use crate::games::shared::*;

use crate::utils::input::*;
use crate::utils::mem::*;
use crate::utils::version::*;

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
    gamepad_index: Pointer,
    gamepad_flags: Pointer,
}

static mut POINTERS: Option<GamePointers> = None;

// Gamepad stuff
static mut GAMEPAD_INDEX_ORIG: i32 = 0;
static mut GAMEPAD_FLAGS_ORIG: u32 = 0;

pub unsafe fn eldenring_init(process: &mut Process) -> GameFuncs {
    // Refresh process
    process.refresh().expect("Failed to refresh process");

    // Inject DLLs
    let soulmods_module = inject_soulmods(process);
    let soulstas_patches_module = inject_soulstas_patches(process);

    // Get exports
    let soulmods_exports: Vec<ModuleExport> = get_exports(soulmods_module.unwrap());
    let soulstas_patches_exports: Vec<ModuleExport> = get_exports(soulstas_patches_module.unwrap());

    // Determine playerins offset based depending on version
    let process_version = Version::from_file_version_info(PathBuf::from(process.get_path()));
    let playerins_offset: usize = if process_version
        >= (Version {
            major: 1,
            minor: 7,
            build: 0,
            revision: 0,
        }) {
        // 1.07.0+
        0x1E508
    } else {
        0x18468
    };

    // Get all necessary memory pointers
    POINTERS = Some(GamePointers {
        fps_patch: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "ER_FPS_PATCH_ENABLED").expect("Couldn't find ER_FPS_PATCH_ENABLED").addr, vec![0]),
        fps_limit: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "ER_FPS_CUSTOM_LIMIT").expect("Couldn't find ER_FPS_CUSTOM_LIMIT").addr, vec![0]),
        frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_FRAME_ADVANCE_ENABLED").expect("Couldn't find ER_FRAME_ADVANCE_ENABLED").addr, vec![0]),
        frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_FRAME_RUNNING").expect("Couldn't find ER_FRAME_RUNNING").addr, vec![0]),
        xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_XINPUT_PATCH_ENABLED").expect("Couldn't find ER_XINPUT_PATCH_ENABLED").addr, vec![0]),
        xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_XINPUT_STATE").expect("Couldn't find ER_XINPUT_STATE").addr, vec![0]),
        input_state: process.scan_rel("input_state", "48 8B 05 ? ? ? ? 48 85 C0 74 0F 48 39 88", 3, 7, vec![0, playerins_offset, 0x58, 0xe8]).expect("Couldn't find input_state pointer"),
        save_active: process.scan_rel("save_active", "4c 8b 0d ? ? ? ? 0f b6 d8 49 8b 69 08 48 8d 8d b0 02 00 00", 3, 7, vec![0, 0x8, 0x8]).expect("Couldn't find save_active pointer"),
        cutscene_3d: process.scan_rel("cutscene_3d", "48 8B 05 ? ? ? ? 48 85 C0 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 05 ? ? ? ? 80 B8 ? ? ? ? 00 75 4F 48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D", 3, 7, vec![0, 0xE1]).expect("Couldn't find cutscene_3d pointer"),
        gamepad_index: process.scan_rel("gamepad_index", "48 8b 1d ? ? ? ? 8b f2 48 8b f9 48 85 db 75 2e", 3, 7, vec![0, 0x18, 0x10, 0x894]).expect("Couldn't find gamepad_index pointer"),
        gamepad_flags: process.scan_rel("gamepad_flags", "48 8b 1d ? ? ? ? 8b f2 48 8b f9 48 85 db 75 2e", 3, 7, vec![0, 0x18, 0x10, 0x90c]).expect("Couldn't find gamepad_flags pointer"),
    });

    // Return all functions
    let game_funcs = GameFuncs {
        script_start: eldenring_script_start,
        script_end: eldenring_script_end,
        frame_next: eldenring_frame_next,
        frame_start: eldenring_frame_start,
        frame_end: eldenring_frame_end,
        action_fps: eldenring_action_fps,
        flag_frame: eldenring_flag_frame,
        flag_ingame: eldenring_flag_ingame,
        flag_cutscene: eldenring_flag_cutscene,
        flag_mainmenu: eldenring_flag_mainmenu,
    };

    return game_funcs;
}

pub unsafe fn eldenring_script_start(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    GAMEPAD_INDEX_ORIG = pointers.gamepad_index.read_i32_rel(None);
    GAMEPAD_FLAGS_ORIG = pointers.gamepad_flags.read_u32_rel(None);

    pointers.xinput_patch.write_u8_rel(None, 1);
}

pub unsafe fn eldenring_script_end(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    pointers.xinput_patch.write_u8_rel(None, 0);

    pointers
        .gamepad_index
        .write_i32_rel(None, GAMEPAD_INDEX_ORIG);
    pointers
        .gamepad_flags
        .write_u32_rel(None, GAMEPAD_FLAGS_ORIG);
}

pub unsafe fn eldenring_frame_next(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn eldenring_frame_start(process: &mut Process) {}

pub unsafe fn eldenring_frame_end(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    // Set correct gamepad flags
    pointers.gamepad_index.write_i32_rel(None, 999);
    pointers.gamepad_flags.write_u32_rel(None, 795);

    // Send gamepad input
    let xinput_state_override_buf = &*(&XINPUT_STATE_OVERRIDE as *const XINPUT_STATE
        as *const [u8; core::mem::size_of::<XINPUT_STATE>()]);
    pointers
        .xinput_state
        .write_memory_rel(None, xinput_state_override_buf);
}

pub unsafe fn eldenring_action_fps(process: &mut Process, fps: f32) {
    let pointers = POINTERS.as_ref().unwrap();
    pointers.fps_limit.write_f32_rel(None, fps);
}

pub unsafe fn eldenring_flag_frame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn eldenring_flag_ingame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u8_rel(None);
    if input_state >> 5 & 1 == 1 && input_state >> 6 & 1 == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn eldenring_flag_cutscene(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    if pointers.cutscene_3d.read_bool_rel(None) {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn eldenring_flag_mainmenu(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) == -1 {
        return true;
    } else {
        return false;
    }
}
