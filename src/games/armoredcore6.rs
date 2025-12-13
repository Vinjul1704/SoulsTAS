use mem_rs::prelude::*;

use crate::games::shared::*;

use crate::utils::mem::*;

struct GamePointers {
    fps_patch: Pointer,
    fps_limit: Pointer,
    frame_advance: Pointer,
    frame_running: Pointer,
    input_state: Pointer,
    save_active: Pointer,
    cutscene_3d: Pointer,
    cutscene_briefing: Pointer,
}

static mut POINTERS: Option<GamePointers> = None;

pub unsafe fn armoredcore6_init(process: &mut Process) -> GameFuncs {
    // Refresh process
    process.refresh().expect("Failed to refresh process");

    // Inject DLLs
    let soulstas_patches_module = inject_soulstas_patches(process);

    // Get exports
    let soulstas_patches_exports: Vec<ModuleExport> = get_exports(soulstas_patches_module.unwrap());

    // Get all necessary memory pointers
    POINTERS = Some(GamePointers {
        fps_patch: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "AC6_FPS_PATCH_ENABLED")
                .expect("Couldn't find AC6_FPS_PATCH_ENABLED")
                .addr,
            vec![0],
        ),
        fps_limit: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "AC6_FPS_CUSTOM_LIMIT")
                .expect("Couldn't find AC6_FPS_CUSTOM_LIMIT")
                .addr,
            vec![0],
        ),
        frame_advance: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "AC6_FRAME_ADVANCE_ENABLED")
                .expect("Couldn't find AC6_FRAME_ADVANCE_ENABLED")
                .addr,
            vec![0],
        ),
        frame_running: process.create_pointer(
            soulstas_patches_exports
                .iter()
                .find(|f| f.name == "AC6_FRAME_RUNNING")
                .expect("Couldn't find AC6_FRAME_RUNNING")
                .addr,
            vec![0],
        ),
        input_state: process
            .scan_rel(
                "input_state",
                "48 8b 1d ? ? ? ? 0f 28 00 66 0f 7f 45 f7 48 85 db",
                3,
                7,
                vec![0, 0xA5A0, 0x80, 0x118],
            )
            .expect("Couldn't find input_state pointer"),
        save_active: process
            .scan_rel(
                "save_active",
                "48 8b 05 ? ? ? ? 48 8b 10 48 83 c2 19 41 b8 10 00 00 00 48 8d 4d 97",
                3,
                7,
                vec![0, 0x8, 0x8],
            )
            .expect("Couldn't find save_active pointer"),
        cutscene_3d: process
            .scan_rel(
                "cutscene_3d",
                "48 39 1d ? ? ? ? 48 8b 4b 18 75 11 45 33 c0",
                3,
                7,
                vec![0, 0x114],
            )
            .expect("Couldn't find cutscene_3d pointer"),
        cutscene_briefing: process
            .scan_rel(
                "cutscene_briefing",
                "48 8b 15 ? ? ? ? 44 8d 4e 03 48 8b 82 90 06 00 00",
                3,
                7,
                vec![0, 0x140, 0x78, 0x98, 0xa8],
            )
            .expect("Couldn't find cutscene_briefing pointer"),
    });

    // Return all functions
    let game_funcs = GameFuncs {
        script_start: armoredcore6_script_start,
        script_end: armoredcore6_script_end,
        frame_next: armoredcore6_frame_next,
        frame_start: armoredcore6_frame_start,
        frame_end: armoredcore6_frame_end,
        action_fps: armoredcore6_action_fps,
        flag_frame: armoredcore6_flag_frame,
        flag_ingame: armoredcore6_flag_ingame,
        flag_cutscene: armoredcore6_flag_cutscene,
        flag_mainmenu: armoredcore6_flag_mainmenu,
    };

    return game_funcs;
}

pub unsafe fn armoredcore6_script_start(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);
}

pub unsafe fn armoredcore6_script_end(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);
}

pub unsafe fn armoredcore6_frame_next(process: &mut Process) {
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn armoredcore6_frame_start(process: &mut Process) {}

pub unsafe fn armoredcore6_frame_end(process: &mut Process) {}

pub unsafe fn armoredcore6_action_fps(process: &mut Process, fps: f32) {
    let pointers = POINTERS.as_ref().unwrap();
    pointers.fps_limit.write_f32_rel(None, fps);
}

pub unsafe fn armoredcore6_flag_frame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn armoredcore6_flag_ingame(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u8_rel(None);
    if input_state >> 0 & 1 == 1
        && input_state >> 1 & 1 == 1
        && input_state >> 2 & 1 == 1
        && input_state >> 3 & 1 == 1
    {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn armoredcore6_flag_cutscene(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();

    // TODO: Handle some 2d/ui cutscenes like briefings
    let cutscene_3d = pointers.cutscene_3d.read_u8_rel(None);
    let cutscene_briefing = pointers.cutscene_briefing.read_u8_rel(None);
    if cutscene_3d >> 3 & 1 == 1 || cutscene_briefing == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn armoredcore6_flag_mainmenu(process: &mut Process) -> bool {
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) != -1 {
        return true;
    } else {
        return false;
    }
}
