#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![allow(unreachable_patterns)]

use std::cmp;
use std::fs::read_to_string;
use std::io::stdin;
use std::path::{Path, PathBuf};
use std::{env, process};
use std::{thread, time::Duration};

use windows::Win32::UI::WindowsAndMessaging::*;

use mem_rs::prelude::*;

mod utils;

use crate::utils::actions::*;
use crate::utils::input::*;
use crate::utils::mem::*;
use crate::utils::version::*;

enum GameType {
    DarkSouls3,
    Sekiro,
    EldenRing,
    NightReign,
    Unknown,
}

struct GamePointers {
    fps_patch: Pointer,
    fps_limit: Pointer,
    frame_advance: Pointer,
    frame_running: Pointer,
    input_state: Pointer,
    save_active: Pointer,
    cutscene_3d: Pointer,
    cutscene_movie: Pointer,
}

const USAGE_TEXT: &str =
    "Usage: soulstas.exe (darksouls3/sekiro/eldenring/nightreign) path/to/tas/script.txt";

fn main() {

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid argument count. {}", USAGE_TEXT);
        process::exit(0);
    }


    // Pick game
    let selected_game = match args[1].as_str().to_lowercase().as_str() {
        "darksouls3" | "ds3" => GameType::DarkSouls3,
        "sekiro" => GameType::Sekiro,
        "eldenring" | "er" => GameType::EldenRing,
        "nightreign" | "nr" => GameType::NightReign,
        _ => {
            println!("Unknown game. {}", USAGE_TEXT);
            process::exit(0);
        }
    };


    // Try to find TAS script file
    let tas_script_path = Path::new(&args[2]);
    if !tas_script_path.exists() {
        println!("Can't find tas script. {}", USAGE_TEXT);
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
                panic!("Error in TAS script at line {}: {}", line_num + 1, err);
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
        panic!("No actions found in TAS script");
    }


    // Get process
    let mut process: Process = match selected_game {
        GameType::DarkSouls3 => Process::new("DarkSoulsIII.exe"),
        GameType::Sekiro => Process::new("sekiro.exe"),
        GameType::EldenRing => Process::new("eldenring.exe"),
        GameType::NightReign => {
            println!("WARNING: Nightreign support might be spotty due to active game updates.");
            Process::new("nightreign.exe")
        },
        _ => {
            println!("Game not implemented. {}", USAGE_TEXT);
            process::exit(0);
        }
    };
    process.refresh().expect("Failed to attach to process");

    // Get game version
    let process_version = Version::from_file_version_info(PathBuf::from(process.get_path()));

    // Get HWND and try to find the soulmods DLL
    let process_hwnd = unsafe { get_hwnd_by_id(process.get_id()) };
    let mut process_module = unsafe { get_module(&mut process, "soulmods_x64.dll") };

    // Load soulmods if it isn't loaded yet
    if process_module.is_none() {
        let exe_path = env::current_exe().unwrap();
        let soulmods_path = PathBuf::from(exe_path)
            .parent()
            .unwrap()
            .join("soulmods_x64.dll");

        process
            .inject_dll(soulmods_path.into_os_string().to_str().unwrap())
            .expect("Failed to inject soulmods_x64.dll");
        process_module = unsafe { get_module(&mut process, "soulmods_x64.dll") };

        if process_module.is_none() {
            panic!("Failed to find soulmods_x64.dll after injection");
        }
    }

    // Get exports
    let exports: Vec<ExportedFunction> = unsafe { get_exports(&mut process, process_module.unwrap()) };

    // Wait for soulmods to be initialized
    let ptr_soulmods_initialized = process.create_pointer(exports.iter().find(|f| f.name == "SOULMODS_INITIALIZED").expect("Couldn't find SOULMODS_INITIALIZED").addr, vec![0]);
    while !ptr_soulmods_initialized.read_bool_rel(None) {
        thread::sleep(Duration::from_micros(10));
    }

    // Set up pointers
    let pointers: GamePointers = match selected_game {
        GameType::EldenRing => {
            let playerins_offset: usize = if process_version <= (Version { major: 1, minor: 6, build: 0, revision: 0 }) { // 1.06.0
                0x18468
            } else {
                0x1E508
            };

            GamePointers {
                fps_patch: process.create_pointer(exports.iter().find(|f| f.name == "ER_FPS_PATCH_ENABLED").expect("Couldn't find ER_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(exports.iter().find(|f| f.name == "ER_FPS_CUSTOM_LIMIT").expect("Couldn't find ER_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(exports.iter().find(|f| f.name == "ER_FRAME_ADVANCE_ENABLED").expect("Couldn't find ER_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(exports.iter().find(|f| f.name == "ER_FRAME_RUNNING").expect("Couldn't find ER_FRAME_RUNNING").addr, vec![0]),
                input_state: process.scan_rel("input_state", "48 8B 05 ? ? ? ? 48 85 C0 74 0F 48 39 88", 3, 7, vec![0, playerins_offset, 0x58, 0xE8]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "4c 8b 0d ? ? ? ? 0f b6 d8 49 8b 69 08 48 8d 8d b0 02 00 00", 3, 7, vec![0, 0x8, 0x8]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8B 05 ? ? ? ? 48 85 C0 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 05 ? ? ? ? 80 B8 ? ? ? ? 00 75 4F 48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D", 3, 7, vec![0, 0xE1]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.create_pointer(0xDEADBEEF, vec![0]),
            }
        },
        GameType::Sekiro => {
            GamePointers {
                fps_patch: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FPS_PATCH_ENABLED").expect("Couldn't find SEKIRO_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FPS_CUSTOM_LIMIT").expect("Couldn't find SEKIRO_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FRAME_ADVANCE_ENABLED").expect("Couldn't find SEKIRO_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FRAME_RUNNING").expect("Couldn't find SEKIRO_FRAME_RUNNING").addr, vec![0]),
                input_state: process.scan_rel("input_state", "48 8B 35 ? ? ? ? 44 0F 28 18", 3, 7, vec![0, 0x88, 0x50, 0x190]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "48 8b 15 ? ? ? ? 8b 44 24 28 f3 0f 10 44 24 30", 3, 7, vec![0, 0xBF4]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 05 ? ? ? ? 4c 8b f9 48 8b 49 08", 3, 7, vec![0, 0xD4]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.scan_rel("cutscene_movie", "80 bf b8 0a 00 00 00 75 3f 48 8b 0d ? ? ? ? 48 85 c9 75 2e 48 8d 0d ? ? ? ? e8 ? ? ? ? 4c 8b c8 4c 8d 05 ? ? ? ? ba b1 00 00 00", 12, 16, vec![0, 0x20]).expect("Couldn't find cutscene_movie pointer"),
            }
        },
        GameType::DarkSouls3 => {
            GamePointers {
                fps_patch: process.create_pointer(exports.iter().find(|f| f.name == "DS3_FPS_PATCH_ENABLED").expect("Couldn't find DS3_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(exports.iter().find(|f| f.name == "DS3_FPS_CUSTOM_LIMIT").expect("Couldn't find DS3_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(exports.iter().find(|f| f.name == "DS3_FRAME_ADVANCE_ENABLED").expect("Couldn't find DS3_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(exports.iter().find(|f| f.name == "DS3_FRAME_RUNNING").expect("Couldn't find DS3_FRAME_RUNNING").addr, vec![0]),
                input_state: process.scan_rel("input_state", "48 8B 1D ? ? ? 04 48 8B F9 48 85 DB ? ? 8B 11 85 D2 ? ? 8D", 3, 7, vec![0, 0x80, 0x50, 0x180]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? 48 8b 48 10 48 85 c9 74 08 0f b6 81 f4", 3, 7, vec![0, 0xD70]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 05 ? ? ? ? 48 85 c0 74 37", 3, 7, vec![0, 0x14C]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.scan_rel("cutscene_movie", "48 8b 0d ? ? ? ? e8 ? ? ? ? 84 c0 74 07 c6 83 c8 00 00 00 01", 3, 7, vec![0, 0x15]).expect("Couldn't find cutscene_movie pointer"),
            }
        },
        GameType::NightReign => {
            GamePointers {
                fps_patch: process.create_pointer(exports.iter().find(|f| f.name == "NR_FPS_PATCH_ENABLED").expect("Couldn't find DS3_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(exports.iter().find(|f| f.name == "NR_FPS_CUSTOM_LIMIT").expect("Couldn't find DS3_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(exports.iter().find(|f| f.name == "NR_FRAME_ADVANCE_ENABLED").expect("Couldn't find DS3_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(exports.iter().find(|f| f.name == "NR_FRAME_RUNNING").expect("Couldn't find DS3_FRAME_RUNNING").addr, vec![0]),
                input_state: process.scan_rel("input_state", "48 8B 05 ? ? ? ? 48 85 C0 74 0C 48 39 88", 3, 7, vec![0, 0x174E8, 0x60, 0xF0]).expect("Couldn't find input_state pointer"),
                save_active: process.scan_rel("save_active", "48 8b 05 ? ? ? ? c6 84 07 02 01 00 00 00 48", 3, 7, vec![0, 0x8, 0x78]).expect("Couldn't find save_active pointer"),
                cutscene_3d: process.scan_rel("cutscene_3d", "48 8b 0d ? ? ? ? 48 85 c9 74 08 48 8b d6 e8 ? ? ? ? 48 8b d6", 3, 7, vec![0, 0xF1]).expect("Couldn't find cutscene_3d pointer"),
                cutscene_movie: process.create_pointer(0xDEADBEEF, vec![0]),
            }
        },
        _ => {
            println!("Game not implemented. {}", USAGE_TEXT);
            process::exit(0);
        }
    };
    

    // Enable necessary patches
    pointers.frame_advance.write_u8_rel(None, 1);
    pointers.fps_patch.write_u8_rel(None, 1);
    pointers.fps_limit.write_f32_rel(None, 0.0);

    // Do TAS stuff
    let mut current_frame = 0;
    while current_frame <= frame_max {
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
                TasActionType::Nothing => { /* Does nothing on purpose */ }
                TasActionType::Fps { fps } => {
                    pointers.fps_limit.write_f32_rel(None, fps);
                }
                TasActionType::Await { flag } => {
                    loop {
                        match flag {
                            AwaitFlag::Control => {
                                match selected_game {
                                    GameType::EldenRing | GameType::NightReign => {
                                        let input_state = pointers.input_state.read_u8_rel(None);
                                        if input_state >> 5 & 1 == 1 && input_state >> 6 & 1 == 1 {
                                            break;
                                        }
                                    },
                                    GameType::Sekiro => {
                                        let input_state = pointers.input_state.read_u8_rel(None);
                                        if input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1 {
                                            break;
                                        }
                                    },
                                    GameType::DarkSouls3 => {
                                        let input_state = pointers.input_state.read_u32_rel(None);
                                        if input_state >> 1 & 1 == 1 && input_state >> 16 & 1 == 1 {
                                            break;
                                        }
                                    },
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
                                    },
                                    GameType::Sekiro => {
                                        let input_state = pointers.input_state.read_u8_rel(None);
                                        if !(input_state >> 0 & 1 == 1 && input_state >> 1 & 1 == 1) {
                                            break;
                                        }
                                    },
                                    GameType::DarkSouls3 => {
                                        let input_state = pointers.input_state.read_u32_rel(None);
                                        if !(input_state >> 1 & 1 == 1 && input_state >> 16 & 1 == 1) {
                                            break;
                                        }
                                    },
                                    _ => {}
                                };
                            }
                            AwaitFlag::Cutscene => {
                                match selected_game {
                                    GameType::EldenRing | GameType::NightReign => {
                                        if pointers.cutscene_3d.read_bool_rel(None) {
                                            break;
                                        }
                                    },
                                    GameType::Sekiro | GameType::DarkSouls3 => {
                                        if pointers.cutscene_3d.read_i8_rel(None) == -7 || pointers.cutscene_movie.read_bool_rel(None) {
                                            break;
                                        }
                                    },
                                    _ => {}
                                };
                            }
                            AwaitFlag::NoCutscene => {
                                match selected_game {
                                    GameType::EldenRing | GameType::NightReign => {
                                        if !pointers.cutscene_3d.read_bool_rel(None) {
                                            break;
                                        }
                                    },
                                    GameType::Sekiro | GameType::DarkSouls3 => {
                                        if pointers.cutscene_3d.read_i8_rel(None) == 0 && !pointers.cutscene_movie.read_bool_rel(None) {
                                            break;
                                        }
                                    },
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
                            AwaitFlag::Focus => {
                                unsafe {
                                    if GetForegroundWindow() == process_hwnd {
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        };

                        pointers.frame_running.write_u8_rel(None, 1);
                        while pointers.frame_running.read_bool_rel(None) {
                            thread::sleep(Duration::from_micros(10));
                        }
                    }
                }
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

        pointers.frame_running.write_u8_rel(None, 1);

        current_frame += 1;
    }

    // Disable patches again
    pointers.frame_advance.write_u8_rel(None, 0);
    pointers.fps_patch.write_u8_rel(None, 0);
    pointers.fps_limit.write_f32_rel(None, 0.0);
}
