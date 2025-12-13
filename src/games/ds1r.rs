use mem_rs::prelude::*;
use std::path::PathBuf;
use windows::Win32::UI::Input::XboxController::*;

use crate::games::shared::*;

use crate::utils::input::*;
use crate::utils::mem::*;
use crate::utils::version::*;

struct GamePointers {
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

pub unsafe fn ds1r_init(process: &mut Process) -> GameFuncs {
    // Refresh process
    process.refresh().expect("Failed to refresh process");

    // Inject DLLs
    let soulstas_patches_module = inject_soulstas_patches(process);

    // Get exports
    let soulstas_patches_exports: Vec<ModuleExport> = get_exports(soulstas_patches_module.unwrap());

    // Determine playerctrl offset based depending on version
    let process_version = Version::from_file_version_info(PathBuf::from(process.get_path()));
    let playerctrl_offset: usize = if process_version
        <= (Version {
            major: 1,
            minor: 3,
            build: 0,
            revision: 0,
        }) {
        // Pre-1.03.0
        0x48
    } else {
        0x68
    };

    // Get all necessary memory pointers
    POINTERS = Some(GamePointers {
        frame_advance: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1R_FRAME_ADVANCE_ENABLED")
                .expect("Couldn't find DS1R_FRAME_ADVANCE_ENABLED")
                .addr,
            vec![0],
        ),
        frame_running: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1R_FRAME_RUNNING")
                .expect("Couldn't find DS1R_FRAME_RUNNING")
                .addr,
            vec![0],
        ),
        xinput_patch: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1R_XINPUT_PATCH_ENABLED")
                .expect("Couldn't find DS1R_XINPUT_PATCH_ENABLED")
                .addr,
            vec![0],
        ),
        xinput_state: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1R_XINPUT_STATE")
                .expect("Couldn't find DS1R_XINPUT_STATE")
                .addr,
            vec![0],
        ),
        input_state: process
            .scan_rel(
                "input_state",
                "48 8b 05 ? ? ? ? 33 ff 83 cd ff 45 0f b6 f0 44 8b fa",
                3,
                7,
                vec![0, 0x68, playerctrl_offset, 0x100],
            )
            .expect("Couldn't find input_state pointer"),
        save_active: process
            .scan_rel(
                "save_active",
                "48 8b 05 ? ? ? ? 48 8b 58 10 48 8b 05 ? ? ? ? 48 8b 78 68",
                3,
                7,
                vec![0, 0xd20],
            )
            .expect("Couldn't find save_active pointer"),
        cutscene_3d: process
            .scan_rel(
                "cutscene_3d",
                "48 8b 05 ? ? ? ? 0f 28 80 60 01 00 00 48 8b c1 66 0f 7f 01",
                3,
                7,
                vec![0, 0x154],
            )
            .expect("Couldn't find cutscene_3d pointer"),
        cutscene_movie: process
            .scan_rel(
                "cutscene_movie",
                "48 89 05 ? ? ? ? 48 8b cf e8 ? ? ? ? 48 89 87 08 02 00 00",
                3,
                7,
                vec![0, 0x60, 0x350],
            )
            .expect("Couldn't find cutscene_movie pointer"),
        gamepad_index: process
            .scan_rel(
                "gamepad_index",
                "48 8b 05 ? ? ? ? 48 8b 48 10 80 79 28 00 75 0e 0f b6 59 28",
                3,
                7,
                vec![0, 0x10, 0x10, 0x264],
            )
            .expect("Couldn't find gamepad_index pointer"),
        gamepad_flags: process
            .scan_rel(
                "gamepad_flags",
                "48 8b 05 ? ? ? ? 48 8b 48 10 80 79 28 00 75 0e 0f b6 59 28",
                3,
                7,
                vec![0, 0x10, 0x10, 0x2dc],
            )
            .expect("Couldn't find gamepad_flags pointer"),
    });

    // Return all functions
    let game_funcs = GameFuncs {
        script_start: ds1r_script_start,
        script_end: ds1r_script_end,
        frame_next: ds1r_frame_next,
        frame_start: ds1r_frame_start,
        frame_end: ds1r_frame_end,
        action_fps: ds1r_action_fps,
        flag_frame: ds1r_flag_frame,
        flag_ingame: ds1r_flag_ingame,
        flag_cutscene: ds1r_flag_cutscene,
        flag_mainmenu: ds1r_flag_mainmenu,
    };

    return game_funcs;
}

pub unsafe fn ds1r_script_start(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    GAMEPAD_INDEX_ORIG = pointers.gamepad_index.read_i32_rel(None);
    GAMEPAD_FLAGS_ORIG = pointers.gamepad_flags.read_u32_rel(None);

    pointers.xinput_patch.write_u8_rel(None, 1);
}

pub unsafe fn ds1r_script_end(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.xinput_patch.write_u8_rel(None, 0);

    pointers
        .gamepad_index
        .write_i32_rel(None, GAMEPAD_INDEX_ORIG);
    pointers
        .gamepad_flags
        .write_u32_rel(None, GAMEPAD_FLAGS_ORIG);
}

pub unsafe fn ds1r_frame_next(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn ds1r_frame_start(process: &mut Process) {}

pub unsafe fn ds1r_frame_end(process: &mut Process) {
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

pub unsafe fn ds1r_action_fps(process: &mut Process, fps: f32) {}

pub unsafe fn ds1r_flag_frame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn ds1r_flag_ingame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u8_rel(None);
    if input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn ds1r_flag_cutscene(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    if pointers.cutscene_3d.read_bool_rel(None) || pointers.cutscene_movie.read_bool_rel(None) {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn ds1r_flag_mainmenu(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) != -1 {
        return true;
    } else {
        return false;
    }
}
