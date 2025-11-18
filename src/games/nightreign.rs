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
}

static mut POINTERS: Option<GamePointers> = None;


pub unsafe fn nightreign_init(process: &mut Process) -> GameFuncs
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
        fps_patch: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "NR_FPS_PATCH_ENABLED").expect("Couldn't find NR_FPS_PATCH_ENABLED").addr, vec![0]),
        fps_limit: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "NR_FPS_CUSTOM_LIMIT").expect("Couldn't find NR_FPS_CUSTOM_LIMIT").addr, vec![0]),
        frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "NR_FRAME_ADVANCE_ENABLED").expect("Couldn't find NR_FRAME_ADVANCE_ENABLED").addr, vec![0]),
        frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "NR_FRAME_RUNNING").expect("Couldn't find NR_FRAME_RUNNING").addr, vec![0]),
        input_state: process.scan_rel("input_state", "48 8B 05 ? ? ? ? 48 85 C0 74 0C 48 39 88", 3, 7, vec![0, 0x174e8, 0x60, 0xf0]).expect("Couldn't find input_state pointer"),
        save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? c6 84 07 02 01 00 00 00 48", 3, 7, vec![0, 0x8, 0x78]).expect("Couldn't find save_active pointer"),
        cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 0d ? ? ? ? 48 8b 49 58 48 85 c9 74 0a", 3, 7, vec![0, 0xf1]).expect("Couldn't find cutscene_3d pointer"),
    });


    // Return all functions
    let game_funcs = GameFuncs {
        script_start: nightreign_script_start,
        script_end: nightreign_script_end,
        frame_next: nightreign_frame_next,
        frame_start: nightreign_frame_start,
        frame_end: nightreign_frame_end,
        action_fps: nightreign_action_fps,
        flag_frame: nightreign_flag_frame,
        flag_control: nightreign_flag_control,
        flag_cutscene: nightreign_flag_cutscene,
        flag_save: nightreign_flag_save
    };

    return game_funcs;
}

pub unsafe fn nightreign_script_start(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);
}

pub unsafe fn nightreign_script_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);
}

pub unsafe fn nightreign_frame_next(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn nightreign_frame_start(process: &mut Process)
{
}

pub unsafe fn nightreign_frame_end(process: &mut Process)
{
}

pub unsafe fn nightreign_action_fps(process: &mut Process, fps: f32)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.fps_limit.write_f32_rel(None, fps);
}

pub unsafe fn nightreign_flag_frame(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn nightreign_flag_control(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();

    let input_state = pointers.input_state.read_u8_rel(None);
    if input_state >> 5 & 1 == 1 && input_state >> 6 & 1 == 1 {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn nightreign_flag_cutscene(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();

    if pointers.cutscene_3d.read_bool_rel(None) {
        return true;
    } else {
        return false;
    }
}

pub unsafe fn nightreign_flag_save(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    if pointers.save_active.read_i32_rel(None) != -1 {
        return true;
    } else {
        return false;
    }
}