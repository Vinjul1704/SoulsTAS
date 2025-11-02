#![allow(static_mut_refs)]
#![allow(unreachable_patterns)]

use std::fs::read_to_string;
use std::io::stdin;
use std::path::{Path, PathBuf};
use std::{cmp, env, process, thread, time::Duration};

use mem_rs::prelude::*;

use windows::Win32::UI::Input::XboxController::*;
use windows::Win32::UI::WindowsAndMessaging::*;

mod utils;

use crate::utils::actions::*;
use crate::utils::input::*;
use crate::utils::mem::*;
use crate::utils::version::*;

#[derive(PartialEq)]
enum GameType {
    DarkSouls1,
    DarkSouls1Remastered,
    DarkSouls3,
    Sekiro,
    EldenRing,
    ArmoredCore6,
    NightReign,
}

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

#[cfg(target_arch = "x86")]
const USAGE_TEXT: &str = "Usage: soulstas_x86.exe ds1 path/to/tas/script.txt";

#[cfg(target_arch = "x86_64")]
const USAGE_TEXT: &str = "Usage: soulstas_x64.exe (dsr/ds3/sekiro/er/ac6/nr) path/to/tas/script.txt";


fn main() {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid argument count. {}", USAGE_TEXT);
        process::exit(0);
    }


    // Pick game
    #[cfg(target_arch = "x86_64")]
    let selected_game = match args[1].as_str().to_lowercase().as_str() {
        "darksouls1remastered" | "ds1r" | "dsr" => GameType::DarkSouls1Remastered,
        "darksouls3" | "ds3" => GameType::DarkSouls3,
        "sekiro" => GameType::Sekiro,
        "eldenring" | "er" => GameType::EldenRing,
        "armoredcore6" | "ac6" => GameType::ArmoredCore6,
        "nightreign" | "nr" => GameType::NightReign,
        _ => {
            println!("Unknown game for current architecture. {}", USAGE_TEXT);
            process::exit(0);
        }
    };

    #[cfg(target_arch = "x86")]
    let selected_game = match args[1].as_str().to_lowercase().as_str() {
        "darksouls1" | "ds1" | "ptde" => GameType::DarkSouls1,
        _ => {
            println!("Unknown game for current architecture. {}", USAGE_TEXT);
            process::exit(0);
        }
    };


    // Try to find TAS script file
    let tas_script_path = Path::new(&args[2]);
    if !tas_script_path.exists() {
        println!("Can't find TAS script. {}", USAGE_TEXT);
        process::exit(0);
    }

    // Set up frame info vars for actions
    let mut frame_max: u32 = 0;
    let mut frame_previous: u32 = 0;
    let mut frame_previous_absolute: u32 = 0;

    // Create action vector
    let mut tas_actions: Vec<TasAction> = Vec::new();

    // Read TAS script and parse actions
    for (line_num, line) in read_to_string(tas_script_path).unwrap().lines().enumerate() {
        // Parse the action
        let action_info: TasActionInfo = match parse_action(line) {
            Ok(res_opt) => {
                if let Some(res) = res_opt {
                    res
                } else {
                    continue;
                }
            }
            Err(err) => {
                println!("Error in TAS script at line {}: {}", line_num + 1, err);
                process::exit(0);
            }
        };

        // Calculate the frame
        let frame: u32 = match action_info.frame_type {
            FrameType::Absolute => {
                frame_previous_absolute = action_info.frame;
                action_info.frame
            }
            FrameType::Relative => action_info.frame + frame_previous,
            FrameType::RelativeAbsolute => action_info.frame + frame_previous_absolute,
        };
        frame_previous = frame;
        frame_max = cmp::max(frame_max, frame);

        // Add the action to the action vector
        tas_actions.push(TasAction {
            frame: frame,
            action: action_info.action,
        });
    }

    // Make sure there are actually any actions
    if tas_actions.len() <= 0 {
        println!("No actions found in TAS script");
        process::exit(0);
    }


    // Get process
    let mut process: Process = match selected_game {
        GameType::DarkSouls1 => {
            Process::new("DARKSOULS.exe") // TODO: Handle DATA.exe
        },
        GameType::DarkSouls1Remastered => {
            println!("WARNING: DSR support might be spotty. Gamepad input is only supported if you have one plugged in.");
            Process::new("DarkSoulsRemastered.exe")
        },
        GameType::DarkSouls3 => Process::new("DarkSoulsIII.exe"),
        GameType::Sekiro => Process::new("sekiro.exe"),
        GameType::EldenRing => Process::new("eldenring.exe"),
        GameType::ArmoredCore6 => {
            println!("WARNING: AC6 support might be spotty. Gamepad input is not supported currently and cutscene actions are not 100% reliable.");
            Process::new("armoredcore6.exe")
        },
        GameType::NightReign => {
            println!("WARNING: Nightreign support might be spotty due to active game updates. Gamepad input is not supported currently.");
            Process::new("nightreign.exe")
        },
        _ => {
            println!("Game not implemented. {}", USAGE_TEXT);
            process::exit(0);
        }
    };
    process.refresh().expect("Failed to attach to process");

    // Get game version and HWND
    let process_version = Version::from_file_version_info(PathBuf::from(process.get_path()));
    let process_hwnd = unsafe { get_hwnd_by_id(process.get_id()) };


    // Get/Inject soulmods
    #[cfg(target_arch = "x86_64")]
    let soulmods_module = unsafe { get_or_inject_module(&mut process, "soulmods_x64.dll") };

    // Get exports
    #[cfg(target_arch = "x86_64")]
    let soulmods_exports: Vec<ModuleExport> = unsafe { get_exports(soulmods_module.unwrap()) };
    #[cfg(target_arch = "x86")]
    let soulmods_exports: Vec<ModuleExport> = Vec::new();

    // Get value to check if DLLs are initialized..
    #[cfg(target_arch = "x86_64")]
    let ptr_soulmods_initialized = process.create_pointer(
        soulmods_exports
            .iter()
            .find(|f| f.name == "SOULMODS_INITIALIZED")
            .expect("Couldn't find SOULMODS_INITIALIZED")
            .addr,
        vec![0],
    );

    // ..and wait until they are
    #[cfg(target_arch = "x86_64")]
    while !ptr_soulmods_initialized.read_bool_rel(None) {
        thread::sleep(Duration::from_micros(10));
    }


    // Get/Inject soulstas patches
    #[cfg(target_arch = "x86_64")]
    let soulstas_patches_module = unsafe { get_or_inject_module(&mut process, "soulstas_patches_x64.dll") };
    #[cfg(target_arch = "x86")]
    let soulstas_patches_module = unsafe { get_or_inject_module(&mut process, "soulstas_patches_x86.dll") };

    // Get exports
    let soulstas_patches_exports: Vec<ModuleExport> = unsafe { get_exports(soulstas_patches_module.unwrap()) };

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


    // Set up pointers
    let pointers: GamePointers = match selected_game {
        GameType::DarkSouls1 => {
            GamePointers {
                fps_patch: process.create_pointer(0xDEADBEEF, vec![0]),
                fps_limit: process.create_pointer(0xDEADBEEF, vec![0]),
                frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1_FRAME_ADVANCE_ENABLED").expect("Couldn't find DS1_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1_FRAME_RUNNING").expect("Couldn't find DS1_FRAME_RUNNING").addr, vec![0]),
                xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1_XINPUT_PATCH_ENABLED").expect("Couldn't find DS1_XINPUT_PATCH_ENABLED").addr, vec![0]),
                xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1_XINPUT_STATE").expect("Couldn't find DS1_XINPUT_STATE").addr, vec![0]),
                input_state: process.scan_abs("input_state", "a1 ? ? ? ? 83 ec 28 53 c7 47 08 00 00 00 00 8b 58 3c", 1, vec![0, 0, 0x3c, 0x28, 0xc0]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_abs("save_active", "8b 15 ? ? ? ? 8a 4a 04 80 f9 ff 74 0f 80 f9 01 75 04 8a c1 59 c3", 2, vec![0, 0, 0x928]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_abs("cutscene_3d", "8b 0d ? ? ? ? 0f 57 c0 0f 2f 41 30 72 12 8b 15 ? ? ? ? 89 9a dc 02 00 00", 2, vec![0, 0, 0x154]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.scan_abs("cutscene_movie", "a3 ? ? ? ? e8 ? ? ? ? 5f 89 86 f4 00 00 00 5e c3 cc 6a", 1, vec![0, 0, 0xf4, 0x93d]).expect("Couldn't find cutscene_movie pointer"),
                gamepad_index: process.scan_abs("gamepad_index", "8b 15 ? ? ? ? f2 0f 5e c8 f2 0f 5a c9 f3 0f 11 4a 34", 2, vec![0, 0, 0x8, 0x8, 0x164]).expect("Couldn't find gamepad_index pointer"),
                gamepad_flags: process.scan_abs("gamepad_flags", "8b 15 ? ? ? ? f2 0f 5e c8 f2 0f 5a c9 f3 0f 11 4a 34", 2, vec![0, 0, 0x8, 0x8, 0x194]).expect("Couldn't find gamepad_flags pointer"),
            }
        },
        GameType::DarkSouls1Remastered => {
            let playerctrl_offset: usize = if process_version <= (Version { major: 1, minor: 3, build: 0, revision: 0 }) { // Pre-1.03.0
                0x48
            } else {
                0x68
            };

            GamePointers {
                fps_patch: process.create_pointer(0xDEADBEEF, vec![0]),
                fps_limit: process.create_pointer(0xDEADBEEF, vec![0]),
                frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1R_FRAME_ADVANCE_ENABLED").expect("Couldn't find DS1R_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1R_FRAME_RUNNING").expect("Couldn't find DS1R_FRAME_RUNNING").addr, vec![0]),
                xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1R_XINPUT_PATCH_ENABLED").expect("Couldn't find DS1R_XINPUT_PATCH_ENABLED").addr, vec![0]),
                xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "DS1R_XINPUT_STATE").expect("Couldn't find DS1R_XINPUT_STATE").addr, vec![0]),
                input_state: process.scan_rel("input_state", "48 8b 05 ? ? ? ? 33 ff 83 cd ff 45 0f b6 f0 44 8b fa", 3, 7, vec![0, 0x68, playerctrl_offset, 0x100]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? 48 8b 58 10 48 8b 05 ? ? ? ? 48 8b 78 68", 3, 7, vec![0, 0xd20]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 05 ? ? ? ? 0f 28 80 60 01 00 00 48 8b c1 66 0f 7f 01", 3, 7, vec![0, 0x154]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.scan_rel("cutscene_movie", "48 89 05 ? ? ? ? 48 8b cf e8 ? ? ? ? 48 89 87 08 02 00 00", 3, 7, vec![0, 0x60, 0x350]).expect("Couldn't find cutscene_movie pointer"),
                gamepad_index: process.scan_rel("gamepad_index", "48 8b 05 ? ? ? ? 48 8b 48 10 80 79 28 00 75 0e 0f b6 59 28", 3, 7, vec![0, 0x10, 0x10, 0x264]).expect("Couldn't find gamepad_index pointer"),
                gamepad_flags: process.scan_rel("gamepad_flags", "48 8b 05 ? ? ? ? 48 8b 48 10 80 79 28 00 75 0e 0f b6 59 28", 3, 7, vec![0, 0x10, 0x10, 0x2dc]).expect("Couldn't find gamepad_flags pointer"),
            }
        },
        GameType::DarkSouls3 => {
            GamePointers {
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
            }
        },
        GameType::Sekiro => {
            GamePointers {
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
            }
        },
        GameType::EldenRing => {
            let playerins_offset: usize = if process_version <= (Version { major: 1, minor: 6, build: 0, revision: 0 }) { // Pre-1.06.0
                0x18468
            } else {
                0x1E508
            };

            GamePointers {
                fps_patch: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "ER_FPS_PATCH_ENABLED").expect("Couldn't find ER_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "ER_FPS_CUSTOM_LIMIT").expect("Couldn't find ER_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_FRAME_ADVANCE_ENABLED").expect("Couldn't find ER_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_FRAME_RUNNING").expect("Couldn't find ER_FRAME_RUNNING").addr, vec![0]),
                xinput_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_XINPUT_PATCH_ENABLED").expect("Couldn't find ER_XINPUT_PATCH_ENABLED").addr, vec![0]),
                xinput_state: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "ER_XINPUT_STATE").expect("Couldn't find ER_XINPUT_STATE").addr, vec![0]),
                input_state: process.scan_rel("input_state", "48 8B 05 ? ? ? ? 48 85 C0 74 0F 48 39 88", 3, 7, vec![0, playerins_offset, 0x58, 0xe8]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "4c 8b 0d ? ? ? ? 0f b6 d8 49 8b 69 08 48 8d 8d b0 02 00 00", 3, 7, vec![0, 0x8, 0x8]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8B 05 ? ? ? ? 48 85 C0 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 05 ? ? ? ? 80 B8 ? ? ? ? 00 75 4F 48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D", 3, 7, vec![0, 0xE1]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.create_pointer(0xDEADBEEF, vec![0]),
                gamepad_index: process.scan_rel("gamepad_index", "48 8b 1d ? ? ? ? 8b f2 48 8b f9 48 85 db 75 2e", 3, 7, vec![0, 0x18, 0x10, 0x894]).expect("Couldn't find gamepad_index pointer"),
                gamepad_flags: process.scan_rel("gamepad_flags", "48 8b 1d ? ? ? ? 8b f2 48 8b f9 48 85 db 75 2e", 3, 7, vec![0, 0x18, 0x10, 0x90c]).expect("Couldn't find gamepad_flags pointer"),
            }
        },
        GameType::ArmoredCore6 => {
            GamePointers {
                fps_patch: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "AC6_FPS_PATCH_ENABLED").expect("Couldn't find AC6_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "AC6_FPS_CUSTOM_LIMIT").expect("Couldn't find AC6_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "AC6_FRAME_ADVANCE_ENABLED").expect("Couldn't find AC6_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "AC6_FRAME_RUNNING").expect("Couldn't find AC6_FRAME_RUNNING").addr, vec![0]),
                xinput_patch: process.create_pointer(0xDEADBEEF, vec![0]),
                xinput_state: process.create_pointer(0xDEADBEEF, vec![0]),
                input_state: process.scan_rel("input_state", "48 8b 1d ? ? ? ? 0f 28 00 66 0f 7f 45 f7 48 85 db", 3, 7, vec![0, 0xA5A0, 0x80, 0x118]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? 48 8b 10 48 83 c2 19 41 b8 10 00 00 00 48 8d 4d 97", 3, 7, vec![0, 0x8, 0x8]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 39 1d ? ? ? ? 48 8b 4b 18 75 11 45 33 c0", 3, 7, vec![0, 0x114]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.create_pointer(0xDEADBEEF, vec![0]),
                gamepad_index: process.create_pointer(0xDEADBEEF, vec![0]),
                gamepad_flags: process.create_pointer(0xDEADBEEF, vec![0]),
            }
        },
        GameType::NightReign => {
            GamePointers {
                fps_patch: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "NR_FPS_PATCH_ENABLED").expect("Couldn't find NR_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(soulmods_exports.iter().find(|f| f.name == "NR_FPS_CUSTOM_LIMIT").expect("Couldn't find NR_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "NR_FRAME_ADVANCE_ENABLED").expect("Couldn't find NR_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(soulstas_patches_exports.iter().find(|f| f.name == "NR_FRAME_RUNNING").expect("Couldn't find NR_FRAME_RUNNING").addr, vec![0]),
                xinput_patch: process.create_pointer(0xDEADBEEF, vec![0]),
                xinput_state: process.create_pointer(0xDEADBEEF, vec![0]),
                input_state: process.scan_rel("input_state", "48 8B 05 ? ? ? ? 48 85 C0 74 0C 48 39 88", 3, 7, vec![0, 0x174e8, 0x60, 0xf0]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? c6 84 07 02 01 00 00 00 48", 3, 7, vec![0, 0x8, 0x78]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 0d ? ? ? ? 48 8b 49 58 48 85 c9 74 0a", 3, 7, vec![0, 0xf1]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.create_pointer(0xDEADBEEF, vec![0]),
                gamepad_index: process.create_pointer(0xDEADBEEF, vec![0]),
                gamepad_flags: process.create_pointer(0xDEADBEEF, vec![0]),
            }
        },
        _ => {
            println!("Game not implemented. {}", USAGE_TEXT);
            process::exit(0);
        }
    };


    // Enable necessary patches
    pointers.frame_advance.write_u8_rel(None, 1);

    match selected_game {
        GameType::DarkSouls1 | GameType::DarkSouls1Remastered => { /* No FPS patch for DS1/DSR */ },
        _ => {
            pointers.fps_patch.write_u8_rel(None, 1);
            pointers.fps_limit.write_f32_rel(None, 0.0);
        }
    }


    // Store gamepad index and flags
    let mut gamepad_index_orig: i32 = 0;
    let mut gamepad_flags_orig: u32 = 0;

    // Enable and set gamepad stuff
    match selected_game {
        GameType::NightReign | GameType::ArmoredCore6 => { /* No gamepad support for NR and AC6 yet */ },
        _ => {
            gamepad_index_orig = pointers.gamepad_index.read_i32_rel(None);
            gamepad_flags_orig = pointers.gamepad_flags.read_u32_rel(None);

            pointers.xinput_patch.write_u8_rel(None, 1);
        }
    }


    // Do TAS stuff
    let mut current_frame = 0;
    while current_frame <= frame_max {
        // Refresh every frame, to ensure the game is still up
        process.refresh().expect("Failed to refresh process");

        println!("{}", current_frame);

        while pointers.frame_running.read_bool_rel(None) {
            thread::sleep(Duration::from_micros(10));
        }

        let running_frame = current_frame;
        for tas_action in tas_actions.iter().filter(|x| x.frame == running_frame) {
            match *&tas_action.action {
                TasActionType::Key { input_type, key } => {
                    unsafe { send_key_raw(key, input_type) };
                }
                TasActionType::KeyAlternative { input_type, key } => {
                    unsafe { send_key(key, input_type) };
                }
                TasActionType::MouseButton { input_type, button } => {
                    unsafe { send_mouse_button(button, input_type) };
                }
                TasActionType::MouseScroll { input_type, amount } => {
                    unsafe { send_mouse_scroll(amount, input_type) };
                }
                TasActionType::MouseMove { x, y } => {
                    unsafe { send_mouse_move(x, y) };
                }
                TasActionType::GamepadButton { input_type, button } => {
                    unsafe { send_gamepad_button(button, input_type) };
                }
                TasActionType::GamepadStick {
                    stick,
                    angle,
                    amount,
                } => {
                    let mut x = angle.to_radians().sin() * amount;
                    x = if x >= 0.0 { x * 32767.0 } else { x * 32768.0 };

                    let mut y = angle.to_radians().cos() * amount;
                    y = if y >= 0.0 { y * 32767.0 } else { y * 32768.0 };

                    match stick {
                        GamepadStick::StickLeft => unsafe {
                            send_gamepad_axis(GamepadAxis::StickLeftX, x.round() as i32);
                            send_gamepad_axis(GamepadAxis::StickLeftY, y.round() as i32);
                        },
                        GamepadStick::StickRight => unsafe {
                            send_gamepad_axis(GamepadAxis::StickRightX, x.round() as i32);
                            send_gamepad_axis(GamepadAxis::StickLeftY, y.round() as i32);
                        },
                        _ => {}
                    }
                }
                TasActionType::GamepadAxis { axis, amount } => {
                    unsafe { send_gamepad_axis(axis, amount) };
                }
                TasActionType::Nothing => { /* Does nothing on purpose */ }
                TasActionType::Fps { fps } => {
                    match selected_game {
                        GameType::DarkSouls1 | GameType::DarkSouls1Remastered => {
                            println!("Setting FPS is not supported in DS1 and DSR. Ignoring...");
                        },
                        _ => {
                            pointers.fps_limit.write_f32_rel(None, fps);
                        }
                    }
                }
                TasActionType::Await { flag } => loop {
                    match flag {
                        AwaitFlag::Control => {
                            match selected_game {
                                GameType::EldenRing | GameType::NightReign => {
                                    let input_state = pointers.input_state.read_u8_rel(None);
                                    if input_state >> 5 & 1 == 1 && input_state >> 6 & 1 == 1 {
                                        break;
                                    }
                                }
                                GameType::ArmoredCore6 => {
                                    let input_state = pointers.input_state.read_u8_rel(None);
                                    if input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 && input_state >> 2 & 1 == 1 && input_state >> 3 & 1 == 1 {
                                        break;
                                    }
                                }
                                GameType::Sekiro | GameType::DarkSouls1 | GameType::DarkSouls1Remastered => {
                                    let input_state = pointers.input_state.read_u8_rel(None);
                                    if input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 {
                                        break;
                                    }
                                }
                                GameType::DarkSouls3 => {
                                    let input_state = pointers.input_state.read_u32_rel(None);
                                    if input_state >> 1 & 1 == 1 && input_state >> 16 & 1 == 1 {
                                        break;
                                    }
                                }
                                _ => {}
                            };
                        }
                        AwaitFlag::NoControl => {
                            match selected_game {
                                GameType::EldenRing | GameType::NightReign => {
                                    let input_state = pointers.input_state.read_u8_rel(None);
                                    if !(input_state >> 5 & 1 == 1 && input_state >> 6 & 1 == 1) {
                                        break;
                                    }
                                }
                                GameType::ArmoredCore6 => {
                                    let input_state = pointers.input_state.read_u8_rel(None);
                                    if !(input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 && input_state >> 2 & 1 == 1 && input_state >> 3 & 1 == 1) {
                                        break;
                                    }
                                }
                                GameType::Sekiro | GameType::DarkSouls1 | GameType::DarkSouls1Remastered => {
                                    let input_state = pointers.input_state.read_u8_rel(None);
                                    if !(input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1) {
                                        break;
                                    }
                                }
                                GameType::DarkSouls3 => {
                                    let input_state = pointers.input_state.read_u32_rel(None);
                                    if !(input_state >> 1 & 1 == 1 && input_state >> 16 & 1 == 1) {
                                        break;
                                    }
                                }
                                _ => {}
                            };
                        }
                        AwaitFlag::Cutscene => {
                            match selected_game {
                                GameType::EldenRing | GameType::NightReign => {
                                    if pointers.cutscene_3d.read_bool_rel(None) {
                                        break;
                                    }
                                }
                                GameType::ArmoredCore6 => { // TODO: Handle some 2d/ui cutscenes like briefings
                                    let cutscene_3d = pointers.cutscene_3d.read_u8_rel(None);
                                    if cutscene_3d >> 3 & 1 == 1 {
                                        break;
                                    }
                                }
                                GameType::Sekiro | GameType::DarkSouls3 => {
                                    if pointers.cutscene_3d.read_i8_rel(None) == -7
                                        || pointers.cutscene_movie.read_bool_rel(None)
                                    {
                                        break;
                                    }
                                }
                                GameType::DarkSouls1 | GameType::DarkSouls1Remastered => {
                                    if pointers.cutscene_3d.read_bool_rel(None)
                                        || pointers.cutscene_movie.read_bool_rel(None)
                                    {
                                        break;
                                    }
                                }
                                _ => {}
                            };
                        }
                        AwaitFlag::NoCutscene => {
                            match selected_game {
                                GameType::EldenRing | GameType::NightReign => {
                                    if !pointers.cutscene_3d.read_bool_rel(None) {
                                        break;
                                    }
                                }
                                GameType::ArmoredCore6 => { // TODO: Handle some 2d/ui cutscenes like briefings
                                    let cutscene_3d = pointers.cutscene_3d.read_u8_rel(None);
                                    if !(cutscene_3d >> 3 & 1 == 1) {
                                        break;
                                    }
                                }
                                GameType::Sekiro | GameType::DarkSouls3 => {
                                    if pointers.cutscene_3d.read_i8_rel(None) == 0
                                        && !pointers.cutscene_movie.read_bool_rel(None)
                                    {
                                        break;
                                    }
                                }
                                GameType::DarkSouls1 | GameType::DarkSouls1Remastered => {
                                    if !pointers.cutscene_3d.read_bool_rel(None)
                                        && !pointers.cutscene_movie.read_bool_rel(None)
                                    {
                                        break;
                                    }
                                }
                                _ => {}
                            };
                        }
                        AwaitFlag::SaveActive => {
                            if pointers.save_active.read_i32_rel(None) != -1 {
                                break;
                            }
                        }
                        AwaitFlag::NoSaveActive => {
                            if pointers.save_active.read_i32_rel(None) == -1 {
                                break;
                            }
                        }
                        AwaitFlag::Focus => unsafe {
                            if GetForegroundWindow() == process_hwnd {
                                break;
                            }
                        },
                        _ => {}
                    };

                    pointers.frame_running.write_u8_rel(None, 1);
                    while pointers.frame_running.read_bool_rel(None) {
                        thread::sleep(Duration::from_micros(10));
                    }
                },
                TasActionType::Frame { frame } => {
                    current_frame = cmp::max(frame - 1, 0);
                }
                TasActionType::PauseMs { ms } => {
                    thread::sleep(Duration::from_millis(ms));
                }
                TasActionType::PauseInput => {
                    println!("Pausing. Press enter to continue.");
                    let mut buffer = String::new();
                    let _ = stdin().read_line(&mut buffer);
                }
                _ => {}
            }
        }

        // Handle gamepad input
        match selected_game {
            GameType::NightReign | GameType::ArmoredCore6 => { /* No gamepad support for NR and AC6 yet */ },
            _ => {
                pointers.gamepad_index.write_i32_rel(None, 999);
                pointers.gamepad_flags.write_u32_rel(None, 795);

                unsafe {
                    let xinput_state_override_buf = &*(&XINPUT_STATE_OVERRIDE as *const XINPUT_STATE
                        as *const [u8; core::mem::size_of::<XINPUT_STATE>()]);
                    pointers.xinput_state.write_memory_rel(None, xinput_state_override_buf);
                }
            }
        }

        pointers.frame_running.write_u8_rel(None, 1);

        current_frame += 1;
    }


    // Disable patches again
    pointers.frame_advance.write_u8_rel(None, 0);

    match selected_game {
        GameType::DarkSouls1 | GameType::DarkSouls1Remastered => { /* No FPS patch for DS1/DSR */ },
        _ => {
            pointers.fps_patch.write_u8_rel(None, 0);
            pointers.fps_limit.write_f32_rel(None, 0.0);
        }
    }


    // Restore gamepad index and flags
    match selected_game {
        GameType::NightReign | GameType::ArmoredCore6 => { /* No gamepad support for NR and AC6 yet */ },
        _ => {
            pointers.xinput_patch.write_u8_rel(None, 0);

            pointers.gamepad_index.write_i32_rel(None, gamepad_index_orig);
            pointers.gamepad_flags.write_u32_rel(None, gamepad_flags_orig);
        }
    }
}
