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
use crate::utils::actions::*;
use crate::utils::input::*;
use crate::utils::mem::*;
use crate::utils::version::*;

const USAGE_TEXT: &str =
    "Usage: soulstas.exe (darksouls3/sekiro/eldenring) path/to/tas/script.soulstas";

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

    // Do TAS stuff
    unsafe {
        let selected_game: &str = args[1].as_str();
        match selected_game {
            // Elden Ring
            "eldenring" | "er" => {
                // Find the process
                let mut process = Process::new("eldenring.exe");
                process
                    .refresh()
                    .expect("Failed to attach to eldenring.exe");

                // Get game version
                let process_version = Version::from_file_version_info(PathBuf::from(process.get_path()));

                // Get HWND and try to find the soulmods DLL
                let process_hwnd = get_hwnd_by_id(process.get_id());
                let mut process_module = get_module(&mut process, "soulmods_x64.dll");

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
                    process_module = get_module(&mut process, "soulmods_x64.dll");

                    if process_module.is_none() {
                        panic!("Failed to find soulmods_x64.dll after injection");
                    }
                }

                // Get soulmods exports
                EXPORTS_ER = get_exports(&mut process, process_module.unwrap());

                // Get screenstate offset based on version
                let screenstate_offset: usize = if process_version <= (Version { major: 1, minor: 2, build: 3, revision: 0 }) { // 1.02.3
                    0x718
                } else if process_version <= (Version { major: 2, minor: 0, build: 1, revision: 0 }) { // 1.10.1
                    0x728
                } else {
                    0x730
                };

                // Not 100% correct, but "okay enough" for now, in particular to check if you are ingame and can control the character.
                // They are set 2 frames too early, which is compensated for in the "await control" action currently.
                // "Menu_Flag" and "Menu_State" taken from: https://github.com/FrankvdStam/SoulSplitter/blob/cfb5be9c5d5c4b5b1b39d149ba4df78bfd1dfb90/src/SoulMemory/EldenRing/EldenRing.cs#L336
                // TODO: Improve

                // Set up memory pointers
                let menu_state_pointer = process.scan_rel("Menu_State", "48 8b 0d ? ? ? ? 48 8b 53 08 48 8b 92 d8 00 00 00 48 83 c4 20 5b", 3, 7, vec![0, screenstate_offset]).expect("Couldn't find menu state pointer");
                let cutscene_pointer = process.scan_rel("Cutscene_Playing", "48 8B 05 ? ? ? ? 48 85 C0 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 05 ? ? ? ? 80 B8 ? ? ? ? 00 75 4F 48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D", 3, 7, vec![0, 0xE1]).expect("Couldn't find cutscene pointer");
                let player_control_pointer = process.scan_rel("Player_Control", "48 8B 0D ? ? ? ? 48 85 C9 75 2E 48 8D 0D ? ? ? ? E8 ? ? ? ? 4C 8B C8 4C 8D 05 ? ? ? ? BA ? ? ? ? 48 8D 0D ? ? ? ? E8 ? ? ? ? 48 8B 0D ? ? ? ? 0F 28 D6 48 8D 54 24 30 E8", 3, 7, vec![0, 0x290, 0x50, 0x20, 0xF59]).expect("Couldn't find player control pointer");
                let menu_flag_pointer = process
                    .scan_rel(
                        "Menu_Flag",
                        "48 8b 0d ? ? ? ? 48 8b 53 08 48 8b 92 d8 00 00 00 48 83 c4 20 5b",
                        3,
                        7,
                        vec![0, 0x18],
                    )
                    .expect("Couldn't find menu flag pointer");

                // Enable patches
                er_frame_advance_set(&process, true);
                er_fps_patch_set(&process, true);
                er_fps_limit_set(&process, 0.0);

                // Iterate over each frame and do actions if needed
                let mut current_frame = 0;
                while current_frame <= frame_max {
                    println!("{}", current_frame);

                    er_frame_advance_wait(&process);

                    let running_frame = current_frame;
                    for tas_action in tas_actions.iter().filter(|x| x.frame == running_frame) {
                        match *&tas_action.action {
                            TasActionType::Key { input_type, key } => {
                                send_key_raw(key, input_type);
                            }
                            TasActionType::KeyAlternative { input_type, key } => {
                                send_key(key, input_type);
                            }
                            TasActionType::MouseButton { input_type, button } => {
                                send_mouse_button(button, input_type);
                            }
                            TasActionType::MouseScroll { input_type, amount } => {
                                send_mouse_scroll(amount, input_type);
                            }
                            TasActionType::MouseMove { x, y } => {
                                send_mouse_move(x, y);
                            }
                            TasActionType::Nothing => { /* Does nothing on purpose */ }
                            TasActionType::Fps { fps } => {
                                er_fps_limit_set(&process, fps);
                            }
                            TasActionType::Await { flag } => {
                                loop {
                                    match flag {
                                        AwaitFlag::Control => {
                                            let player_control =
                                                player_control_pointer.read_bool_rel(None);
                                            let menu_flag = menu_flag_pointer.read_u32_rel(None);

                                            if player_control && menu_flag == 65793 {
                                                // This simply advances 2 additional frames once the flags are set.
                                                // It's a workaround for the current (bad) flags triggering 2 frames too early.
                                                er_frame_advance_next(&process);
                                                er_frame_advance_wait(&process);

                                                er_frame_advance_next(&process);
                                                er_frame_advance_wait(&process);

                                                break;
                                            }
                                        }
                                        AwaitFlag::NoControl => {
                                            let player_control =
                                                player_control_pointer.read_bool_rel(None);
                                            let menu_flag = menu_flag_pointer.read_u32_rel(None);

                                            if !(player_control && menu_flag == 65793) {
                                                break;
                                            }
                                        }
                                        AwaitFlag::Cutscene => {
                                            if cutscene_pointer.read_bool_rel(None) {
                                                break;
                                            }
                                        }
                                        AwaitFlag::NoCutscene => {
                                            if !cutscene_pointer.read_bool_rel(None) {
                                                break;
                                            }
                                        }
                                        AwaitFlag::Loading => {
                                            if menu_state_pointer.read_u8_rel(None) == 1 {
                                                break;
                                            }
                                        }
                                        AwaitFlag::NoLoading => {
                                            if menu_state_pointer.read_u8_rel(None) != 1 {
                                                break;
                                            }
                                        }
                                        AwaitFlag::Focus => {
                                            if GetForegroundWindow() == process_hwnd {
                                                break;
                                            }
                                        }
                                        _ => {}
                                    };

                                    er_frame_advance_next(&process);
                                    thread::sleep(Duration::from_millis(10));
                                    er_frame_advance_wait(&process);
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

                    er_frame_advance_next(&process);

                    current_frame += 1;
                }

                // Disable the patches again at the end
                er_frame_advance_set(&process, false);
                er_fps_patch_set(&process, false);
                er_fps_limit_set(&process, 0.0);
            }
            _ => {
                println!("Bad game specified. {}", USAGE_TEXT);
                process::exit(0);
            }
        }
    }
}
