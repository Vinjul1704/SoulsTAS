#![allow(dead_code)]
#![allow(static_mut_refs)]
#![allow(unreachable_patterns)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_variables)]


use std::fs::read_to_string;
use std::io::stdin;
use std::{cmp, env, process, thread, time::Duration, path::Path};

use mem_rs::prelude::*;

use windows::Win32::UI::WindowsAndMessaging::*;

mod utils;
mod games;

use crate::utils::actions::*;
use crate::utils::input::*;
use crate::utils::mem::*;

use crate::games::*;


#[derive(PartialEq)]
enum GameType {
    DarkSouls1,
    DarkSouls1Remastered,
    DarkSouls2,
    DarkSouls2Sotfs,
    DarkSouls3,
    Sekiro,
    EldenRing,
    ArmoredCore6,
    NightReign,
}


#[cfg(target_arch = "x86_64")]
const USAGE_TEXT: &str = "Usage: soulstas_x64.exe (dsr/sotfs/ds3/sekiro/er/ac6/nr) path/to/tas/script.txt";

#[cfg(target_arch = "x86")]
const USAGE_TEXT: &str = "Usage: soulstas_x86.exe ds1 path/to/tas/script.txt";
// const USAGE_TEXT: &str = "Usage: soulstas_x86.exe (ds1/ds2) path/to/tas/script.txt";


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
        "darksouls2sotfs" | "ds2s" | "sotfs" => GameType::DarkSouls2Sotfs,
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
        "darksouls2" | "ds2" => GameType::DarkSouls2,
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


    // Attach to game
    let mut process: Process = match selected_game {
        GameType::DarkSouls1 => Process::new("DARKSOULS.exe"), // TODO: Handle DATA.exe
        GameType::DarkSouls1Remastered => {
            println!("WARNING: DSR support might be spotty. Gamepad input is only supported if you have one plugged in.");
            Process::new("DarkSoulsRemastered.exe")
        },
        GameType::DarkSouls2 | GameType::DarkSouls2Sotfs => Process::new("DarkSoulsII.exe"),
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

    
    // Get all funcs for given game
    #[cfg(target_arch = "x86_64")]
    let game_funcs: GameFuncs = match selected_game {
        GameType::DarkSouls1Remastered => unsafe { ds1r_init(&mut process) },
        GameType::DarkSouls2Sotfs => unsafe { ds2sotfs_init(&mut process) },
        GameType::DarkSouls3 => unsafe { ds3_init(&mut process) },
        GameType::Sekiro => unsafe { sekiro_init(&mut process) },
        GameType::EldenRing => unsafe { eldenring_init(&mut process) },
        GameType::ArmoredCore6 => unsafe { armoredcore6_init(&mut process) },
        GameType::NightReign => unsafe { nightreign_init(&mut process) },
        _ => {
            println!("Game not implemented. {}", USAGE_TEXT);
            process::exit(0);
        }
    };

    #[cfg(target_arch = "x86")]
    let game_funcs: GameFuncs = match selected_game {
        GameType::DarkSouls1 => unsafe { ds1_init(&mut process) },
        GameType::DarkSouls2 => unsafe { ds2_init(&mut process) },
        _ => {
            println!("Game not implemented. {}", USAGE_TEXT);
            process::exit(0);
        }
    };


    // Get game version and HWND
    let process_hwnd = unsafe { get_hwnd_by_id(process.get_id()) };

    unsafe {
        // Run stuff before the script starts
        (game_funcs.script_start)(&mut process);
    }


    // Do TAS stuff
    let mut current_frame = 0;
    while current_frame <= frame_max {
        // Refresh every frame, to ensure the game is still up
        process.refresh().expect("Failed to refresh process");

        println!("{}", current_frame);

        unsafe {
            // Wait for the game to finish its frame
            while (game_funcs.flag_frame)(&mut process) {
                thread::sleep(Duration::from_micros(10));
            }

            // Do stuff at the very beginning of a frame, before the actions
            (game_funcs.frame_start)(&mut process);
        }

        let running_frame = current_frame;
        for tas_action in tas_actions.iter().filter(|x| x.frame == running_frame) {
            match *&tas_action.action {
                TasActionType::Key { input_type, key } => unsafe {
                    send_key_raw(key, input_type);
                }
                TasActionType::KeyAlternative { input_type, key } => unsafe {
                    send_key(key, input_type);
                }
                TasActionType::MouseButton { input_type, button } => unsafe {
                    send_mouse_button(button, input_type);
                }
                TasActionType::MouseScroll { input_type, amount } => unsafe {
                    send_mouse_scroll(amount, input_type);
                }
                TasActionType::MouseMove { x, y } => unsafe {
                    send_mouse_move(x, y);
                }
                TasActionType::GamepadButton { input_type, button } => unsafe {
                    send_gamepad_button(button, input_type);
                }
                TasActionType::GamepadStick { stick, angle, amount, } => {
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
                TasActionType::GamepadAxis { axis, amount } => unsafe {
                    send_gamepad_axis(axis, amount);
                }
                TasActionType::Nothing => { /* Does nothing on purpose */ }
                TasActionType::Fps { fps } => unsafe {
                    (game_funcs.action_fps)(&mut process, fps);
                }
                TasActionType::Await { flag } => loop {
                    match flag {
                        AwaitFlag::Ingame => unsafe {
                            if (game_funcs.flag_ingame)(&mut process) {
                                break;
                            }
                        }
                        AwaitFlag::NoIngame => unsafe {
                            if !(game_funcs.flag_ingame)(&mut process) {
                                break;
                            }
                        }
                        AwaitFlag::Cutscene => unsafe {
                            if (game_funcs.flag_cutscene)(&mut process) {
                                break;
                            }
                        }
                        AwaitFlag::NoCutscene => unsafe {
                            if !(game_funcs.flag_cutscene)(&mut process) {
                                break;
                            }
                        }
                        AwaitFlag::Mainmenu => unsafe {
                            if (game_funcs.flag_mainmenu)(&mut process) {
                                break;
                            }
                        }
                        AwaitFlag::NoMainmenu => unsafe {
                            if !(game_funcs.flag_mainmenu)(&mut process) {
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

                    unsafe {
                        (game_funcs.frame_next)(&mut process);

                        while (game_funcs.flag_frame)(&mut process) {
                            thread::sleep(Duration::from_micros(10));
                        }
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

        unsafe {
            // Do stuff after the actions, like sending inputs
            (game_funcs.frame_end)(&mut process);

            // Run the next frame
            (game_funcs.frame_next)(&mut process);
        }

        current_frame += 1;
    }

    unsafe {
        // Run stuff after the script, cleanup etc.
        (game_funcs.script_end)(&mut process);
    }
}
