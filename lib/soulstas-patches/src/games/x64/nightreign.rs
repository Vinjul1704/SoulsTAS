use std::thread;
use std::time::Duration;
use ilhook::x64::{Hooker, HookType, Registers, CallbackOption, HookFlags, HookPoint};
use mem_rs::prelude::*;
use log::info;

use crate::util::GLOBAL_VERSION;


static mut FRAME_ADVANCE_HOOK: Option<HookPoint> = None;


#[unsafe(no_mangle)]
#[used]
pub static mut NR_FRAME_RUNNING: bool = false;

#[unsafe(no_mangle)]
#[used]
pub static mut NR_FRAME_ADVANCE_ENABLED: bool = false;


pub fn init_nightreign()
{
    unsafe
    {
        info!("version: {}", GLOBAL_VERSION);
        let mut process = Process::new_with_memory_type("nightreign.exe", MemoryType::Direct);
        process.refresh().unwrap();


        // AoB scan for frame advance patch
        let fn_frame_advance_address = process.scan_abs("frame_advance", "e8 ? ? ? ? e8 ? ? ? ? 84 c0 74 4f", 21, Vec::new()).unwrap().get_base_address();
        info!("Frame advance at 0x{:x}", fn_frame_advance_address);

        // Enable frame advance patch
        FRAME_ADVANCE_HOOK = Some(Hooker::new(fn_frame_advance_address, HookType::JmpBack(frame_advance), CallbackOption::None, 0, HookFlags::empty()).hook().unwrap());
    }
}


// Frame advance patch
unsafe extern "win64" fn frame_advance(_registers: *mut Registers, _:usize)
{
    unsafe
    {
        if NR_FRAME_ADVANCE_ENABLED
        {
            NR_FRAME_RUNNING = false;

            while !NR_FRAME_RUNNING && NR_FRAME_ADVANCE_ENABLED {
                thread::sleep(Duration::from_micros(10));
            }
        }
    }
}