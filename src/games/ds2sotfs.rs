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
    /*
    input_state: Pointer,
    save_active: Pointer,
    cutscene_3d: Pointer,
    cutscene_movie: Pointer,
    */
}

static mut POINTERS: Option<GamePointers> = None;


pub unsafe fn ds2sotfs_init(process: &mut Process) -> GameFuncs
{
    // Refresh process
    process.refresh().expect("Failed to refresh process");


    // Inject DLLs
    let soulstas_patches_module = inject_soulstas_patches(process);

    // Get exports
    let soulstas_patches_exports: Vec<ModuleExport> = get_exports(soulstas_patches_module.unwrap());


    // Get all necessary memory pointers
    POINTERS = Some(GamePointers {
        fps_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS2SOTFS_FPS_PATCH_ENABLED").expect("Couldn't find DS2SOTFS_FPS_PATCH_ENABLED").addr, vec![0]),
        fps_limit: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS2SOTFS_FPS_CUSTOM_LIMIT").expect("Couldn't find DS2SOTFS_FPS_CUSTOM_LIMIT").addr, vec![0]),
        frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS2SOTFS_FRAME_ADVANCE_ENABLED").expect("Couldn't find DS2SOTFS_FRAME_ADVANCE_ENABLED").addr, vec![0]),
        frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS2SOTFS_FRAME_RUNNING").expect("Couldn't find DS2SOTFS_FRAME_RUNNING").addr, vec![0]),
        xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS2SOTFS_XINPUT_PATCH_ENABLED").expect("Couldn't find DS2SOTFS_XINPUT_PATCH_ENABLED").addr, vec![0]),
        xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS2SOTFS_XINPUT_STATE").expect("Couldn't find DS2SOTFS_XINPUT_STATE").addr, vec![0]),
        /*
        input_state: process.scan_rel("input_state", "", 3, 7, vec![0]).expect("Couldn't find input_state pointer"),
        save_active: process.scan_rel("save_active", "", 3, 7, vec![0]).expect("Couldn't find save_active pointer"),
        cutscene_3d: process.scan_rel("cutscene_3d", "", 3, 7, vec![0]).expect("Couldn't find cutscene_3d pointer"),
        cutscene_movie: process.scan_rel("cutscene_movie", "", 3, 7, vec![0]).expect("Couldn't find cutscene_movie pointer"),
        */
    });


    // Return all functions
    let game_funcs = GameFuncs {
        script_start: ds2sotfs_script_start,
        script_end: ds2sotfs_script_end,
        frame_next: ds2sotfs_frame_next,
        frame_start: ds2sotfs_frame_start,
        frame_end: ds2sotfs_frame_end,
        action_fps: ds2sotfs_action_fps,
        flag_frame: ds2sotfs_flag_frame,
        flag_control: ds2sotfs_flag_control,
        flag_cutscene: ds2sotfs_flag_cutscene,
        flag_save: ds2sotfs_flag_save
    };

    return game_funcs;
}

pub unsafe fn ds2sotfs_script_start(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 1);

    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    pointers.xinput_patch.write_u8_rel(None, 1);
}

pub unsafe fn ds2sotfs_script_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    pointers.frame_advance.write_u8_rel(None, 0);

    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    pointers.xinput_patch.write_u8_rel(None, 0);
}

pub unsafe fn ds2sotfs_frame_next(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.frame_running.write_u8_rel(None, 1);
}

pub unsafe fn ds2sotfs_frame_start(process: &mut Process)
{
}

pub unsafe fn ds2sotfs_frame_end(process: &mut Process)
{
    let pointers = POINTERS.as_ref().unwrap();

    // Send gamepad input
    let xinput_state_override_buf = &*(&XINPUT_STATE_OVERRIDE as *const XINPUT_STATE as *const [u8; core::mem::size_of::<XINPUT_STATE>()]);
    pointers.xinput_state.write_memory_rel(None, xinput_state_override_buf);
}

pub unsafe fn ds2sotfs_action_fps(process: &mut Process, fps: f32)
{
    let pointers = POINTERS.as_ref().unwrap();
    pointers.fps_limit.write_f32_rel(None, fps);
}

pub unsafe fn ds2sotfs_flag_frame(process: &mut Process) -> bool
{
    let pointers = POINTERS.as_ref().unwrap();
    return pointers.frame_running.read_bool_rel(None);
}

pub unsafe fn ds2sotfs_flag_control(process: &mut Process) -> bool
{
    return true; // TEMP
}

pub unsafe fn ds2sotfs_flag_cutscene(process: &mut Process) -> bool
{
    return true; // TEMP
}

pub unsafe fn ds2sotfs_flag_save(process: &mut Process) -> bool
{
    return true; // TEMP
}