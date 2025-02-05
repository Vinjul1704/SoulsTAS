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
    Unknown,
}

struct GamePointers {
    fps_patch: Pointer,
    fps_limit: Pointer,
    frame_advance: Pointer,
    frame_running: Pointer,
    input_state: Pointer,
    menu_state: Pointer,
    cutscene_skippable: Pointer,
}

const USAGE_TEXT: &str =
    "Usage: soulstas.exe (darksouls3/sekiro/eldenring) path/to/tas/script.txt";

fn main() {

    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid argument count. {}", USAGE_TEXT);
        process::exit(0);
    }


    // Pick game
    let selected_game = match args[1].as_str() {
        "darksouls3" | "ds3" => GameType::DarkSouls3,
        "sekiro" => GameType::Sekiro,
        "eldenring" | "er" => GameType::EldenRing,
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

    // Set up pointers
    let pointers: GamePointers = match selected_game {
        GameType::EldenRing => {
            let screenstate_offset: usize = if process_version <= (Version { major: 1, minor: 2, build: 3, revision: 0 }) { // 1.02.3
                0x718
            } else if process_version <= (Version { major: 2, minor: 0, build: 1, revision: 0 }) { // 1.10.1
                0x728
            } else {
                0x730
            };

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
                menu_state: process.scan_rel("menu_state", "48 8b 0d ? ? ? ? 48 8b 53 08 48 8b 92 d8 00 00 00 48 83 c4 20 5b", 3, 7, vec![0, screenstate_offset]).expect("Couldn't find menu_state pointer"),
                cutscene_skippable: process.scan_rel("cutscene_skippable", "48 8B 05 ? ? ? ? 48 85 C0 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 05 ? ? ? ? 80 B8 ? ? ? ? 00 75 4F 48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D", 3, 7, vec![0, 0xE1]).expect("Couldn't find cutscene_skippable pointer"),
            }
        },
        GameType::Sekiro => {
            GamePointers {
                fps_patch: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FPS_PATCH_ENABLED").expect("Couldn't find SEKIRO_FPS_PATCH_ENABLED").addr, vec![0]),
                fps_limit: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FPS_CUSTOM_LIMIT").expect("Couldn't find SEKIRO_FPS_CUSTOM_LIMIT").addr, vec![0]),
                frame_advance: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FRAME_ADVANCE_ENABLED").expect("Couldn't find SEKIRO_FRAME_ADVANCE_ENABLED").addr, vec![0]),
                frame_running: process.create_pointer(exports.iter().find(|f| f.name == "SEKIRO_FRAME_RUNNING").expect("Couldn't find SEKIRO_FRAME_RUNNING").addr, vec![0]),
                // TODO: Add missing pointers
                input_state: process.scan_rel("input_state", "48 8B 35 ? ? ? ? 44 0F 28 18", 3, 7, vec![0, 0x88, 0x50, 0x190]).expect("Couldn't find input_state pointer"),
                menu_state: process.create_pointer(0xDEADBEEF, vec![0]),
                cutscene_skippable: process.create_pointer(0xDEADBEEF, vec![0]),
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
                                    GameType::EldenRing => {
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
                                    _ => {}
                                };
                            }
                            AwaitFlag::NoControl => {
                                match selected_game {
                                    GameType::EldenRing => {
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
                                    _ => {}
                                };
                            }
                            AwaitFlag::Cutscene => {
                                if pointers.cutscene_skippable.read_bool_rel(None) {
                                    break;
                                }
                            }
                            AwaitFlag::NoCutscene => {
                                if !pointers.cutscene_skippable.read_bool_rel(None) {
                                    break;
                                }
                            }
                            AwaitFlag::Loading => {
                                if pointers.menu_state.read_u8_rel(None) == 1 {
                                    break;
                                }
                            }
                            AwaitFlag::NoLoading => {
                                if pointers.menu_state.read_u8_rel(None) != 1 {
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
