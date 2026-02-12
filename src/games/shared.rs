use mem_rs::prelude::*;
use std::{thread, time::Duration};

use crate::utils::mem::*;

pub struct GameFuncs {
    pub script_start: unsafe fn(&mut Process), // Before script
    pub script_end: unsafe fn(&mut Process),   // After script
    pub frame_next: unsafe fn(&mut Process),   // Run the next frame
    pub frame_start: unsafe fn(&mut Process),  // Start of frame, before actions
    pub frame_end: unsafe fn(&mut Process),    // End of frame, after actions
    pub action_fps: unsafe fn(&mut Process, f32), // Action to set FPS
    pub flag_frame: unsafe fn(&mut Process) -> bool, // Flag to determine if a frame is running
    pub flag_ingame: unsafe fn(&mut Process) -> bool, // Flag to determine if you are ingame and have control
    pub flag_cutscene: unsafe fn(&mut Process) -> bool, // Flag to determine if a skippable cutscene of any kind is playing
    pub flag_mainmenu: unsafe fn(&mut Process) -> bool, // Flag to determine if you are in the main menu
    pub flag_position: unsafe fn(&mut Process, f32, f32, f32, f32) -> bool, // Flag to determine if you are near a position within range (X, Y, Z, Range)
    pub flag_position_alternative: unsafe fn(&mut Process, f32, f32, f32, f32) -> bool, // Flag to determine if you are near a position within range (X, Y, Z, Range), alternative coords (different per-game, if implemented)
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn inject_soulmods(process: &mut Process) -> Option<ProcessModule> {
    // Refresh process
    process.refresh().expect("Failed to refresh process");

    // Get/Inject soulmods
    let soulmods_module = get_or_inject_module(process, "soulmods_x64.dll");

    // Get exports
    let soulmods_exports: Vec<ModuleExport> = get_exports(soulmods_module.clone().unwrap());

    // Get value to check if DLLs are initialized..
    let ptr_soulmods_initialized = process.create_pointer(
        soulmods_exports
            .iter()
            .find(|f| f.name == "SOULMODS_INITIALIZED")
            .expect("Couldn't find SOULMODS_INITIALIZED")
            .addr,
        vec![0],
    );

    // ..and wait until they are
    while !ptr_soulmods_initialized.read_bool_rel(None) {
        thread::sleep(Duration::from_micros(10));
    }

    return soulmods_module;
}

pub unsafe fn inject_soulstas_patches(process: &mut Process) -> Option<ProcessModule> {
    // Refresh process
    process.refresh().expect("Failed to refresh process");

    // Get/Inject soulstas patches
    #[cfg(target_arch = "x86_64")]
    let soulstas_patches_module = get_or_inject_module(process, "soulstas_patches_x64.dll");
    #[cfg(target_arch = "x86")]
    let soulstas_patches_module = get_or_inject_module(process, "soulstas_patches_x86.dll");

    // Get exports
    let soulstas_patches_exports: Vec<ModuleExport> =
        get_exports(soulstas_patches_module.clone().unwrap());

    // Get value to check if DLLs are initialized..
    let ptr_soulstas_patches_initialized = process.create_pointer(
        soulstas_patches_exports
            .iter()
            .find(|f| f.name == "SOULSTAS_PATCHES_INITIALIZED")
            .expect("Couldn't find SOULSTAS_PATCHES_INITIALIZED")
            .addr,
        vec![0],
    );

    // ..and wait until they are
    while !ptr_soulstas_patches_initialized.read_bool_rel(None) {
        thread::sleep(Duration::from_micros(10));
    }

    return soulstas_patches_module;
}
