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

mod games;
mod utils;

use crate::games::eldenring::*;
use crate::utils::input::*;
use crate::utils::mem::*;

enum TasActionType {
    KeyDown,
    KeyUp,
    MouseDown,
    MouseUp,
    ScrollDown,
    ScrollUp,
    MouseMove,
    Nothing,
    FpsLimit,
    AwaitControl,
    AwaitNoControl,
    AwaitCutscene,
    AwaitNoCutscene,
    AwaitFocus,
    SetFrame,
    PauseMs,
    PauseInput,
}

struct TasAction {
    frame: u32,
    action_type: TasActionType,
    params: Vec<String>,
}

const USAGE_TEXT: &str =
    "Usage: souls-tas-rust.exe (darksouls3/sekiro/eldenring) path/to/tas/script.soulstas";

fn main() {
    // Parse arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Invalid argument count. {}", USAGE_TEXT);
        process::exit(0);
    }

    // Try to find TAS script file
    let tas_script_path = Path::new(&args[2]);
    if !tas_script_path.exists() {
        println!("Can't find tas script. {}", USAGE_TEXT);
        process::exit(0);
    }

    // Parse TAS script file
    let mut previous_frame = 0;
    let mut previous_frame_absolute = 0;
    let mut tas_actions: Vec<TasAction> = Vec::new();
    for (line_num, line) in read_to_string(tas_script_path).unwrap().lines().enumerate() {
        let comment_index = line.find(';');
        let length: usize = if comment_index.is_some() {
            comment_index.unwrap()
        } else {
            line.len()
        };

        let line_uncommented: String = line.chars().into_iter().take(length).collect();
        let mut line_parts = line_uncommented.split_whitespace();

        let frame_opt = line_parts.next();
        if frame_opt.is_none() {
            continue;
        }

        let frame: u32 = if frame_opt.unwrap().starts_with("++") {
            let frame_addition: u32 = frame_opt
                .unwrap()
                .split_at(2)
                .1
                .parse()
                .unwrap_or_else(|_| panic!("TAS script error at line {}: Invalid frame", line_num));
            previous_frame_absolute + frame_addition
        } else if frame_opt.unwrap().starts_with("+") {
            let frame_addition: u32 = frame_opt
                .unwrap()
                .split_at(1)
                .1
                .parse()
                .unwrap_or_else(|_| panic!("TAS script error at line {}: Invalid frame", line_num));
            previous_frame + frame_addition
        } else {
            let frame_absolute: u32 = frame_opt
                .unwrap()
                .parse()
                .unwrap_or_else(|_| panic!("TAS script error at line {}: Invalid frame", line_num));
            previous_frame_absolute = frame_absolute;
            frame_absolute
        };

        previous_frame = frame;

        let action_type_opt = line_parts.next();
        if action_type_opt.is_none() {
            panic!("TAS script error at line {}: Missing action", line_num);
        }
        let action_type: TasActionType = match action_type_opt.unwrap() {
            "key_down" => TasActionType::KeyDown,
            "key_up" => TasActionType::KeyUp,
            "mouse_down" => TasActionType::MouseDown,
            "mouse_up" => TasActionType::MouseUp,
            "scroll_down" => TasActionType::ScrollDown,
            "scroll_up" => TasActionType::ScrollUp,
            "mouse_move" => TasActionType::MouseMove,
            "nothing" => TasActionType::Nothing,
            "fps_limit" => TasActionType::FpsLimit,
            "await_control" => TasActionType::AwaitControl,
            "await_no_control" => TasActionType::AwaitNoControl,
            "await_cutscene" => TasActionType::AwaitCutscene,
            "await_no_cutscene" => TasActionType::AwaitNoCutscene,
            "await_focus" => TasActionType::AwaitFocus,
            "set_frame" => TasActionType::SetFrame,
            "pause_ms" => TasActionType::PauseMs,
            "pause_input" => TasActionType::PauseInput,
            _ => {
                panic!("TAS script error at line {}: Invalid action", line_num)
            }
        };

        let mut params: Vec<String> = Vec::new();
        for line_part in line_parts {
            params.push(line_part.to_string());
        }

        match action_type {
            TasActionType::KeyDown | TasActionType::KeyUp => {
                if params.len() != 1 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                if string_to_keycode(params[0].as_str()).is_none() {
                    panic!("TAS script error at line {}: Invalid key", line_num);
                }
            }
            TasActionType::MouseDown | TasActionType::MouseUp => {
                if params.len() != 1 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                match params[0].as_str() {
                    "left" | "l" | "right" | "r" | "middle" | "m" | "extra1" | "e1" | "extra2"
                    | "e2" => {}
                    _ => panic!("TAS script error at line {}: Invalid button", line_num),
                };
            }
            TasActionType::ScrollDown | TasActionType::ScrollUp => {
                if params.len() != 1 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                params[0].parse::<u32>().unwrap_or_else(|_| {
                    panic!(
                        "TAS script error at line {}: Invalid scroll number",
                        line_num
                    )
                });
            }
            TasActionType::MouseMove => {
                if params.len() != 2 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                params[0].parse::<i32>().unwrap_or_else(|_| {
                    panic!("TAS script error at line {}: Invalid X number", line_num)
                });
                params[1].parse::<i32>().unwrap_or_else(|_| {
                    panic!("TAS script error at line {}: Invalid Y number", line_num)
                });
            }
            TasActionType::Nothing
            | TasActionType::AwaitControl
            | TasActionType::AwaitNoControl
            | TasActionType::AwaitCutscene
            | TasActionType::AwaitNoCutscene
            | TasActionType::AwaitFocus
            | TasActionType::PauseInput => {
                // TODO: Implement
                if params.len() != 0 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
            }
            TasActionType::FpsLimit => {
                if params.len() != 1 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                let fps = params[0].parse::<f32>().unwrap_or_else(|_| {
                    panic!("TAS script error at line {}: Invalid FPS", line_num)
                });
                if (fps < 20.0 || fps > 60.0) && fps != 0.0 {
                    panic!("TAS script error at line {}: Invalid FPS", line_num);
                }
                if fps == 60.0 {
                    println!("TAS script warning at line {}: Detected 60 FPS, which is not exactly the same as the default 60 FPS limit. To use the default 60 FPS limit, set it to 0 instead.", line_num);
                }
            }
            TasActionType::SetFrame => {
                if params.len() != 1 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                params[0].parse::<u32>().unwrap_or_else(|_| {
                    panic!("TAS script error at line {}: Invalid frame", line_num)
                });
            }
            TasActionType::PauseMs => {
                if params.len() != 1 {
                    panic!(
                        "TAS script error at line {}: Invalid action parameter count",
                        line_num
                    );
                }
                params[0].parse::<u64>().unwrap_or_else(|_| {
                    panic!("TAS script error at line {}: Invalid frame", line_num)
                });
            }
            _ => {
                panic!("TAS script error at line {}: Invalid action", line_num)
            }
        }

        tas_actions.push(TasAction {
            frame: frame,
            action_type: action_type,
            params: params,
        });
    }

    let tas_length = tas_actions
        .iter()
        .max_by(|x, y| x.frame.partial_cmp(&y.frame).unwrap())
        .unwrap()
        .frame;

    // Do TAS stuff
    let selected_game: &str = args[1].as_str();
    match selected_game {
        "eldenring" | "er" => {
            let mut process = Process::new("eldenring.exe");
            process
                .refresh()
                .expect("Failed to attach to eldenring.exe");

            // let process_version = Version::from_file_version_info(PathBuf::from(process_path));

            let cutscene_pointer = process.scan_rel("Cutscene_Playing", "48 8B 05 ? ? ? ? 48 85 C0 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 05 ? ? ? ? 80 B8 ? ? ? ? 00 75 4F 48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D", 3, 7, vec![0, 0xE1]).expect("Couldn't find cutscene pointer");

            // Not 100% correct, but "okay enough" for now to check if you are ingame and can control the character.
            // They are set 2 frames too early, which is compensated for in the "await_control" action currently.
            // "Menu_Flag" taken from: https://github.com/FrankvdStam/SoulSplitter/blob/cfb5be9c5d5c4b5b1b39d149ba4df78bfd1dfb90/src/SoulMemory/EldenRing/EldenRing.cs#L336
            // TODO: Improve
            let player_control_pointer = process.scan_rel("Player_Control", "48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 0D ? ? ? ? 0F 28 D6 48 8D 54 24 30 E8", 3, 7, vec![0, 0x298, 0x90, 0x8, 0xFE9]).expect("Couldn't find player control pointer");
            let menu_flag_pointer = process
                .scan_rel(
                    "Menu_Flag",
                    "48 8b 0d ? ? ? ? 48 8b 53 08 48 8b 92 d8 00 00 00 48 83 c4 20 5b",
                    3,
                    7,
                    vec![0, 0x18],
                )
                .expect("Couldn't find menu flag pointer");

            let process_hwnd = unsafe { get_hwnd_by_id(process.get_id()) };
            let mut process_module = unsafe { get_module(&mut process, "soulmods.dll") };

            if process_module.is_none() {
                let exe_path = env::current_exe().unwrap();
                let soulmods_path = PathBuf::from(exe_path)
                    .parent()
                    .unwrap()
                    .join("soulmods.dll");

                process
                    .inject_dll(soulmods_path.into_os_string().to_str().unwrap())
                    .expect("Failed to inject soulmods.dll");
                process_module = unsafe { get_module(&mut process, "soulmods.dll") };

                if process_module.is_none() {
                    panic!("Failed to find soulmods.dll after injection");
                }
            }

            unsafe {
                EXPORTED_FUNCS_ER = get_exported_funcs(&mut process, process_module.unwrap());
                er_frame_advance_get_pointers(&process);

                er_frame_advance_set(&process, true);
                er_fps_patch_set(&process, true);
                er_fps_limit_set(&process, 0.0);

                let mut current_frame = 0;
                while current_frame <= tas_length {
                    println!("{}", current_frame);

                    er_frame_advance_wait(&process);

                    let running_frame = current_frame;
                    for tas_action in tas_actions.iter().filter(|x| x.frame == running_frame) {
                        match tas_action.action_type {
                            TasActionType::KeyDown => {
                                let key_code = string_to_keycode(tas_action.params[0].as_str())
                                    .expect("Invalid key");
                                send_key(key_code, false);
                            }
                            TasActionType::KeyUp => {
                                let key_code = string_to_keycode(tas_action.params[0].as_str())
                                    .expect("Invalid key");
                                send_key(key_code, true);
                            }
                            TasActionType::MouseDown => {
                                send_mouse(tas_action.params[0].as_str(), false);
                            }
                            TasActionType::MouseUp => {
                                send_mouse(tas_action.params[0].as_str(), true);
                            }
                            TasActionType::ScrollDown => {
                                send_scroll(tas_action.params[0].parse::<u32>().unwrap(), false);
                            }
                            TasActionType::ScrollUp => {
                                send_scroll(tas_action.params[0].parse::<u32>().unwrap(), true);
                            }
                            TasActionType::MouseMove => {
                                mouse_move(
                                    tas_action.params[0].parse::<i32>().unwrap(),
                                    tas_action.params[1].parse::<i32>().unwrap(),
                                );
                            }
                            TasActionType::Nothing => { /* Does nothing on purpose */ }
                            TasActionType::FpsLimit => {
                                er_fps_limit_set(
                                    &process,
                                    tas_action.params[0].parse::<f32>().unwrap(),
                                );
                            }
                            TasActionType::AwaitControl => {
                                // This check is not great, but it "works" for now.
                                // TODO: Definitely look into and improve this.

                                loop {
                                    let player_control = player_control_pointer.read_bool_rel(None);
                                    let menu_flag = menu_flag_pointer.read_u32_rel(None);

                                    if player_control && menu_flag == 65793 {
                                        break;
                                    } else {
                                        er_frame_advance_next(&process);
                                        thread::sleep(Duration::from_millis(15));
                                        er_frame_advance_wait(&process);
                                    }
                                }

                                // This simply advances 2 additional frames once the flags are set.
                                // It's a workaround for the current (bad) flags triggering 2 frames too early.
                                er_frame_advance_next(&process);
                                er_frame_advance_wait(&process);

                                er_frame_advance_next(&process);
                                er_frame_advance_wait(&process);
                            }
                            TasActionType::AwaitNoControl => loop {
                                let player_control = player_control_pointer.read_bool_rel(None);
                                let menu_flag = menu_flag_pointer.read_u32_rel(None);

                                if !(player_control && menu_flag == 65793) {
                                    break;
                                } else {
                                    er_frame_advance_next(&process);
                                    thread::sleep(Duration::from_millis(15));
                                    er_frame_advance_wait(&process);
                                }
                            },
                            TasActionType::AwaitCutscene => loop {
                                let cutscene_playing = cutscene_pointer.read_bool_rel(None);

                                if cutscene_playing {
                                    break;
                                } else {
                                    er_frame_advance_next(&process);
                                    thread::sleep(Duration::from_millis(15));
                                    er_frame_advance_wait(&process);
                                }
                            },
                            TasActionType::AwaitNoCutscene => loop {
                                let cutscene_playing = cutscene_pointer.read_bool_rel(None);

                                if !cutscene_playing {
                                    break;
                                } else {
                                    er_frame_advance_next(&process);
                                    thread::sleep(Duration::from_millis(15));
                                    er_frame_advance_wait(&process);
                                }
                            },
                            TasActionType::AwaitFocus => loop {
                                let focus_window_hwnd = GetForegroundWindow();

                                if focus_window_hwnd == process_hwnd {
                                    break;
                                } else {
                                    er_frame_advance_next(&process);
                                    thread::sleep(Duration::from_millis(15));
                                    er_frame_advance_wait(&process);
                                }
                            },
                            TasActionType::SetFrame => {
                                current_frame = cmp::max(
                                    tas_action.params[0].parse::<u32>().unwrap() as i32 - 1,
                                    0,
                                ) as u32;
                            }
                            TasActionType::PauseMs => {
                                thread::sleep(Duration::from_millis(
                                    tas_action.params[0].parse::<u64>().unwrap(),
                                ));
                            }
                            TasActionType::PauseInput => {
                                println!("Pausing. Press enter to continue.");
                                let mut buffer = String::new();
                                let _ = stdin().read_line(&mut buffer);
                            }
                            _ => {}
                        }
                    }

                    er_frame_advance_next(&process);

                    current_frame += 1;
                }

                er_frame_advance_set(&process, false);
                er_fps_patch_set(&process, false);
                er_fps_limit_set(&process, 0.0);
            }
        }
        _ => {
            println!("Bad game specified. {}", USAGE_TEXT);
            process::exit(0);
        }
    }
}
