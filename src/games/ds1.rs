use mem_rs::prelude::*;
use windows::Win32::UI::Input::XboxController::*;

use crate::games::shared::*;

use crate::utils::input::*;
use crate::utils::mem::*;

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

pub unsafe fn ds1_init(process: &mut Process) -> GameFuncs {
    // Refresh process
    process.refresh().expect("Failed to refresh process");

    // Inject DLLs
    let soulstas_patches_module = inject_soulstas_patches(process);

    // Get exports
    let soulstas_patches_exports: Vec<ModuleExport> = get_exports(soulstas_patches_module.unwrap());

    // Get all necessary memory pointers
    POINTERS = Some(GamePointers {
        frame_advance: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1_FRAME_ADVANCE_ENABLED")
                .expect("Couldn't find DS1_FRAME_ADVANCE_ENABLED")
                .addr,
            vec![0],
        ),
        frame_running: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1_FRAME_RUNNING")
                .expect("Couldn't find DS1_FRAME_RUNNING")
                .addr,
            vec![0],
        ),
        xinput_patch: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1_XINPUT_PATCH_ENABLED")
                .expect("Couldn't find DS1_XINPUT_PATCH_ENABLED")
                .addr,
            vec![0],
        ),
        xinput_state: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "DS1_XINPUT_STATE")
                .expect("Couldn't find DS1_XINPUT_STATE")
                .addr,
            vec![0],
        ),
        input_state: process
            .scan_abs(
                "input_state",
                "a1 ? ? ? ? 83 ec 28 53 c7 47 08 00 00 00 00 8b 58 3c",
                1,
                vec![0, 0, 0x3c, 0x28, 0xc0],
            )
            .expect("Couldn't find input_state pointer"),
        save_active: process
            .scan_abs(
                "save_active",
                "8b 15 ? ? ? ? 8a 4a 04 80 f9 ff 74 0f 80 f9 01 75 04 8a c1 59 c3",
                2,
                vec![0, 0, 0x928],
            )
            .expect("Couldn't find save_active pointer"),
        cutscene_3d: process
            .scan_abs(
                "cutscene_3d",
                "8b 0d ? ? ? ? 0f 57 c0 0f 2f 41 30 72 12 8b 15 ? ? ? ? 89 9a dc 02 00 00",
                2,
                vec![0, 0, 0x154],
            )
            .expect("Couldn't find cutscene_3d pointer"),
        cutscene_movie: process
            .scan_abs(
                "cutscene_movie",
                "a3 ? ? ? ? e8 ? ? ? ? 5f 89 86 f4 00 00 00 5e c3 cc 6a",
                1,
                vec![0, 0, 0xf4, 0x93d],
            )
            .expect("Couldn't find cutscene_movie pointer"),
        gamepad_index: process
            .scan_abs(
                "gamepad_index",
                "8b 15 ? ? ? ? f2 0f 5e c8 f2 0f 5a c9 f3 0f 11 4a 34",
                2,
                vec![0, 0, 0x8, 0x8, 0x164],
            )
            .expect("Couldn't find gamepad_index pointer"),
        gamepad_flags: process
            .scan_abs(
                "gamepad_flags",
                "8b 15 ? ? ? ? f2 0f 5e c8 f2 0f 5a c9 f3 0f 11 4a 34",
                2,
                vec![0, 0, 0x8, 0x8, 0x194],
            )
            .expect("Couldn't find gamepad_flags pointer"),
    });

    // Return all functions
    let game_funcs = GameFuncs {
        script_start: ds1_script_start,
        script_end: ds1_script_end,
        frame_next: ds1_frame_next,
        frame_start: ds1_frame_start,
        frame_end: ds1_frame_end,
        action_fps: ds1_action_fps,
        flag_frame: ds1_flag_frame,
        flag_ingame: ds1_flag_ingame,
        flag_cutscene: ds1_flag_cutscene,
        flag_mainmenu: ds1_flag_mainmenu,
    };

    return game_funcs;
}

// TODO: Disable FPS check/kick
pub unsafe fn ds1_script_start(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    GAMEPAD_INDEX_ORIG = pointers.gamepad_index.read_i32_rel(None);
    GAMEPAD_FLAGS_ORIG = pointers.gamepad_flags.read_u32_rel(None);

    pointers.xinput_patch.write_u8_rel(None, 1);
}

pub unsafe fn ds1_script_end(process: &mut Process) {
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

pub unsafe fn ds1_frame_next(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn ds1_frame_start(process: &mut Process) {}

pub unsafe fn ds1_frame_end(process: &mut Process) {
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

pub unsafe fn ds1_action_fps(process: &mut Process, fps: f32) {}

pub unsafe fn ds1_flag_frame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn ds1_flag_ingame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u8_rel(None);
    if input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn ds1_flag_cutscene(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    if pointers.cutscene_3d.read_bool_rel(None) || pointers.cutscene_movie.read_bool_rel(None) {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn ds1_flag_mainmenu(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) != -1 {
        return true;
    } else {
        return false;
    }
}
