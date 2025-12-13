// This file is part of the SoulSplitter distribution (https://github.com/FrankvdStam/SoulSplitter).
// Copyright (c) 2022 Frank van der Stam.
// https://github.com/FrankvdStam/SoulSplitter/blob/main/LICENSE
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::{thread, time::Duration, time::Instant, mem};

use ilhook::x64::{Hooker, HookType, Registers, CallbackOption, HookFlags, HookPoint};
use mem_rs::prelude::*;

use log::info;

use windows::Win32::UI::Input::XboxController::*;
use windows::Win32::System::Performance::*;

use crate::util::GLOBAL_VERSION;


static mut FRAME_ADVANCE_HOOK: Option<HookPoint> = None;
static mut FPS_HOOK: Option<HookPoint> = None;
static mut XINPUT_HOOK: Option<HookPoint> = None;


#[unsafe(no_mangle)]
#[used]
pub static mut DS2SOTFS_FRAME_ADVANCE_ENABLED: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut DS2SOTFS_FRAME_RUNNING: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut DS2SOTFS_FPS_PATCH_ENABLED: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut DS2SOTFS_FPS_CUSTOM_LIMIT: f32 = 0.0f32;

#[unsafe(no_mangle)]
#[used]
pub static mut DS2SOTFS_XINPUT_PATCH_ENABLED: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut DS2SOTFS_XINPUT_STATE: XINPUT_STATE = XINPUT_STATE {
    dwPacketNumber: 0,
    Gamepad: XINPUT_GAMEPAD {
        wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0),
        bLeftTrigger: 0,
        bRightTrigger: 0,
        sThumbLX: 0,
        sThumbLY: 0,
        sThumbRX: 0,
        sThumbRY: 0,
    }
};


static mut LAST_FRAME_TIMESTAMP: Option<Instant> = None;
static mut PERFORMANCE_FREQUENCY: i64 = 10000000;

// Thanks Radai
#[repr(C)]
pub struct DeltatimeData {
    total_deltatime_sum: f32,
    recent_deltatime_sum: f32,
    rolling_deltatime_sum: f32,
    current_deltatime_index: u32,
    consecutive_short_frames: u32,
    unk_0x14: u32,
    last_frame_counter: i64,
    performance_frequency: i64,
    unk_0x28: u64,
    deltatime: f32,
    frametime_adjustment: f32,
    frame_count: u64,
    last_fps_update_time: i64,
    fps_1: f32,
    fps_2: f32,
    enable_dynamic_adjustment: u8,
    dynamic_adjustment_flag: u8,
    unk_0x52: u16,
    frametime_offset: f32,
    target_framerate: u32
}

pub type CalculateDeltatime = unsafe extern "win64" fn(deltatime_data: *mut DeltatimeData, suppress_update_time: bool) -> f32;

pub type XInputGetState = unsafe extern "system" fn(dw_user_index: u32, p_state: *mut XINPUT_STATE) -> u32;


#[allow(unused_assignments)]
pub fn init_darksouls2sotfs()
{
    unsafe
    {
        info!("version: {}", GLOBAL_VERSION);
        
        // Get DS2S process
        let mut process = Process::new_with_memory_type("DarkSoulsII.exe", MemoryType::Direct);
        process.refresh().unwrap();


        let fn_frame_advance_address = process.scan_abs("frame_advance", "80 b9 36 01 00 00 00 48 8b d9 74 16 48 8b 49 08 ba 01 00 00 00 ff ? ? ? ? ? c6 83 36 01 00 00 00 48 8b cb", 0, Vec::new()).unwrap().get_base_address();
        info!("Frame advance at 0x{:x}", fn_frame_advance_address);

        // Enable frame advance patch
        FRAME_ADVANCE_HOOK = Some(Hooker::new(fn_frame_advance_address, HookType::JmpBack(frame_advance), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());


        // Get performance frequency
        let _ = QueryPerformanceFrequency(&mut PERFORMANCE_FREQUENCY);

        // AoB scan for FPS patch
        let fn_fps_address = process.scan_abs("fps", "48 8b c4 56 57 41 56 48 81 ec 90 00 00 00 0f 29 70 c8", 0, Vec::new()).unwrap().get_base_address();
        info!("FPS at 0x{:x}", fn_fps_address);

        // Enable FPS patch
        FPS_HOOK = Some(Hooker::new(fn_fps_address, HookType::Retn(fps), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());


        // Find XInputGetState function in XINPUT1_3.dll
        let xinput_module = process.get_modules().iter().find(|m| m.name == "XINPUT1_3.dll").cloned().expect("Couldn't find XINPUT1_3.dll");
        let xinput_fn_addr = xinput_module.get_exports().iter().find(|e| e.0 == "XInputGetState").expect("Couldn't find XInputGetState").1;
        info!("XInputGetState at 0x{:x}", xinput_fn_addr);

        // Hook XInputGetState
        XINPUT_HOOK = Some(Hooker::new(xinput_fn_addr, HookType::Retn(xinput_fn), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());
    }
}


// Frame advance patch
unsafe extern "win64" fn frame_advance(_registers: *mut Registers, _:usize)
{
    unsafe
    {
        if DS2SOTFS_FRAME_ADVANCE_ENABLED
        {
            DS2SOTFS_FRAME_RUNNING = false;

            while !DS2SOTFS_FRAME_RUNNING && DS2SOTFS_FRAME_ADVANCE_ENABLED {
                thread::sleep(Duration::from_micros(10));
            }
        }
    }
}


// FPS patch
pub unsafe extern "win64" fn fps(registers: *mut Registers, orig_func_ptr: usize, _: usize) -> usize {
    
    unsafe
    {
        if DS2SOTFS_FPS_PATCH_ENABLED
        {
            let deltatime_data = (*registers).rcx as *mut DeltatimeData;
            // let suppress_update_time = (*registers).rdx != 0;


            let deltatime_max_stock: f32 = 0.016666668;
            let deltatime_min_stock: f32 = 0.05;
            let custom_deltatime = 1.0 / DS2SOTFS_FPS_CUSTOM_LIMIT;

            let deltatime_override = if custom_deltatime < deltatime_max_stock || DS2SOTFS_FPS_CUSTOM_LIMIT == 0.0 {
                deltatime_max_stock
            } else if custom_deltatime > deltatime_min_stock {
                deltatime_min_stock
            } else {
                custom_deltatime
            };

            let fps_override = 1.0 / deltatime_override;


            let deltasum_current = deltatime_override / 3.0;
            match (*deltatime_data).current_deltatime_index { // I know this sucks, it's 1 AM and I'm about to fall over into a coma, okay?
                1 => {
                    (*deltatime_data).total_deltatime_sum = 0.0;
                    (*deltatime_data).recent_deltatime_sum = deltasum_current * 2.0;
                    (*deltatime_data).rolling_deltatime_sum = deltasum_current;
                    (*deltatime_data).current_deltatime_index = 0;
                },
                2 => {
                    (*deltatime_data).total_deltatime_sum = deltasum_current;
                    (*deltatime_data).recent_deltatime_sum = 0.0;
                    (*deltatime_data).rolling_deltatime_sum = deltasum_current * 2.0;
                    (*deltatime_data).current_deltatime_index = 1;
                },
                _ => {
                    (*deltatime_data).total_deltatime_sum = deltasum_current * 2.0;
                    (*deltatime_data).recent_deltatime_sum = deltasum_current;
                    (*deltatime_data).rolling_deltatime_sum = 0.0;
                    (*deltatime_data).current_deltatime_index = 2;
                },
            }


            (*deltatime_data).consecutive_short_frames = 999;

            let mut timestamp: i64 = 0;
            let _ = QueryPerformanceCounter(&mut timestamp);

            (*deltatime_data).last_frame_counter = timestamp;
            (*deltatime_data).last_fps_update_time = timestamp;

            (*deltatime_data).deltatime = deltatime_override;
            (*deltatime_data).frametime_adjustment = deltatime_override;

            (*deltatime_data).frame_count = 0; // Unsure what exactly it does, but setting it to 0 seems fine?

            (*deltatime_data).fps_1 = fps_override;
            (*deltatime_data).fps_2 = fps_override;

            (*deltatime_data).enable_dynamic_adjustment = 1;
            (*deltatime_data).frametime_offset = -0.1; // Seems to always be -0.1?


            // Fixed values
            (*deltatime_data).performance_frequency = PERFORMANCE_FREQUENCY;
            (*deltatime_data).dynamic_adjustment_flag = 0;
            (*deltatime_data).target_framerate = 20;

            // Unknown values
            // (*deltatime_data).unk_0x14 = 0;
            // (*deltatime_data).unk_0x28 = 0;
            // (*daltetime_data).unk_0x52 = 0;


            // Limit FPS
            if let Some(last_frame_timestamp) = LAST_FRAME_TIMESTAMP {
                let next_frame_timestamp = last_frame_timestamp + Duration::from_secs_f32(deltatime_override);
                let sleep_duration_option = next_frame_timestamp.checked_duration_since(Instant::now());

                if let Some(sleep_duration) = sleep_duration_option {
                    spin_sleep::sleep(sleep_duration);
                }
            }

            LAST_FRAME_TIMESTAMP = Some(Instant::now());


            // Workaround to put return value into the correct register
            (*registers).xmm0 = f32::to_bits(deltatime_override) as u128;
            return (*registers).rax as usize;
        }
        else
        {
            let deltatime_data = (*registers).rcx as *mut DeltatimeData;
            let suppress_update_time = (*registers).rdx != 0;

            let orig_func: CalculateDeltatime = mem::transmute(orig_func_ptr);
            let frametime = orig_func(deltatime_data, suppress_update_time);

            // Workaround to put return value into the correct register
            (*registers).xmm0 = f32::to_bits(frametime) as u128;
            return (*registers).rax as usize;
        }
    }
}


pub unsafe extern "win64" fn xinput_fn(registers: *mut Registers, orig_func_ptr: usize, _: usize) -> usize {
    
    unsafe
    {
        let dw_user_index = (*registers).rcx as u32;
        let p_state = (*registers).rdx as *mut XINPUT_STATE;

        if !DS2SOTFS_XINPUT_PATCH_ENABLED {
            let orig_func: XInputGetState = mem::transmute(orig_func_ptr);
            return orig_func(dw_user_index, p_state) as usize;
        }

        (*p_state) = DS2SOTFS_XINPUT_STATE;

        // ERROR_SUCCESS = 0x0
        // ERROR_DEVICE_NOT_CONNECTED = 0x48F
        return 0x0;
    }
}