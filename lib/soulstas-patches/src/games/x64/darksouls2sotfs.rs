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


        let fn_frame_advance_address = process.scan_abs("frame_advance", "48 8d 4d a8 45 33 c9 45 33 c0 33 d2 89 5c 24 20", 0, Vec::new()).unwrap().get_base_address();
        info!("Frame advance at 0x{:x}", fn_frame_advance_address);

        // Enable frame advance patch
        FRAME_ADVANCE_HOOK = Some(Hooker::new(fn_frame_advance_address, HookType::JmpBack(frame_advance), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());


        // AoB scan for FPS patch
        let fn_fps_address = process.scan_abs("fps", "74 2e 8b 8b 54 01 00 00 ff c9 74 1c ff c9 74 0e ff c9 75 1c", 0, Vec::new()).unwrap().get_base_address();
        info!("FPS at 0x{:x}", fn_fps_address);

        // Enable FPS patch
        FPS_HOOK = Some(Hooker::new(fn_fps_address, HookType::JmpBack(fps), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());


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
unsafe extern "win64" fn fps(registers: *mut Registers, _:usize)
{
    unsafe
    {
        if DS2SOTFS_FPS_PATCH_ENABLED
        {
            let ptr_mainapp = (*registers).rbx as *const u8; // Main app struct
            let ptr_deltatime_data = ptr_mainapp.offset(0xd0) as *const u8; // Struct with FPS-related data

            let ptr_deltasum_index = ptr_deltatime_data.offset(0xc) as *mut u32; // Index of current deltasum value
            let ptr_deltasum_0 = ptr_deltatime_data.offset(0x0) as *mut f32;
            let ptr_deltasum_1 = ptr_deltatime_data.offset(0x4) as *mut f32;
            let ptr_deltasum_2 = ptr_deltatime_data.offset(0x8) as *mut f32;

            let ptr_shortframes = ptr_deltatime_data.offset(0x10) as *mut u32; // Amount of "short frames" in a row

            let ptr_timestamp = ptr_deltatime_data.offset(0x18) as *mut u64;
            let ptr_timestamp_lastupdate = ptr_deltatime_data.offset(0x40) as *mut u64;

            let ptr_deltatime = ptr_deltatime_data.offset(0x30) as *mut f32; // Current deltatime
            let ptr_deltatime_adjusted = ptr_deltatime_data.offset(0x34) as *mut f32; // Current deltatime, but seemingly adjusted to within limits

            let ptr_framecount = ptr_deltatime_data.offset(0x38) as *mut u64; // Unclear, but kinda just switching between 0/1-ish and almost max value randomly

            let ptr_fps = ptr_deltatime_data.offset(0x48) as *mut f32;
            let ptr_fps_alt = ptr_deltatime_data.offset(0x4c) as *mut f32;

            let ptr_adjustment = ptr_deltatime_data.offset(0x50) as *mut u16; // Value to indicate if "dynamic adjustment" is enabled, just leave at 1
            let ptr_adjustment_offset = ptr_deltatime_data.offset(0x54) as *mut f32; // Some kind of adjustment offset value?


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


            let deltasum_index = std::ptr::read_volatile(ptr_deltasum_index);
            let deltasum_current = deltatime_override / 3.0;
            match deltasum_index { // I know this sucks, it's 1 AM and I'm about to fall over into a coma, okay?
                1 => {
                    std::ptr::write_volatile(ptr_deltasum_0, 0.0);
                    std::ptr::write_volatile(ptr_deltasum_1, deltasum_current * 2.0);
                    std::ptr::write_volatile(ptr_deltasum_2, deltasum_current);
                },
                2 => {
                    std::ptr::write_volatile(ptr_deltasum_0, deltasum_current);
                    std::ptr::write_volatile(ptr_deltasum_1, 0.0);
                    std::ptr::write_volatile(ptr_deltasum_2, deltasum_current * 2.0);
                },
                _ => {
                    std::ptr::write_volatile(ptr_deltasum_0, deltasum_current * 2.0);
                    std::ptr::write_volatile(ptr_deltasum_1, deltasum_current);
                    std::ptr::write_volatile(ptr_deltasum_2, 0.0);
                },
            }

            std::ptr::write_volatile(ptr_shortframes, 999); // Set to anything > 5 to ensure it still "runs" at the limit

            let timestamp = std::ptr::read_volatile(ptr_timestamp);
            std::ptr::write_volatile(ptr_timestamp_lastupdate, timestamp); // Just always set them the same, to be safe?

            std::ptr::write_volatile(ptr_deltatime, deltatime_override);
            std::ptr::write_volatile(ptr_deltatime_adjusted, deltatime_override);

            std::ptr::write_volatile(ptr_framecount, 0); // Unsure what exactly it does, but setting it to 0 seems fine?

            std::ptr::write_volatile(ptr_fps, fps_override);
            std::ptr::write_volatile(ptr_fps_alt, fps_override);

            std::ptr::write_volatile(ptr_adjustment, 1);
            std::ptr::write_volatile(ptr_adjustment_offset, -0.1); // Seems to always be -0.1?

            (*registers).xmm0 = f32::to_bits(deltatime_override) as u128;


            // Work around DS2 being dumb
            if let Some(last_frame_timestamp) = LAST_FRAME_TIMESTAMP {
                let next_frame_timestamp = last_frame_timestamp + Duration::from_secs_f32(deltatime_override);
                let sleep_duration_option = next_frame_timestamp.checked_duration_since(Instant::now());

                if let Some(sleep_duration) = sleep_duration_option {
                    thread::sleep(sleep_duration);
                }
            }

            LAST_FRAME_TIMESTAMP = Some(Instant::now());

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