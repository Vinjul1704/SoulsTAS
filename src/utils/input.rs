use std::mem::*;

use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Input::XboxController::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use mem_rs::prelude::*;

use crate::utils::actions::*;
use crate::utils::mem::*;

pub static mut XINPUT_STATE_OVERRIDE: XINPUT_STATE = XINPUT_STATE {
    dwPacketNumber: 0,
    Gamepad: XINPUT_GAMEPAD {
        wButtons: XINPUT_GAMEPAD_BUTTON_FLAGS(0),
        bLeftTrigger: 0,
        bRightTrigger: 0,
        sThumbLX: 0,
        sThumbLY: 0,
        sThumbRX: 0,
        sThumbRY: 0,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Extra1,
    Extra2,
}

#[derive(Debug, Clone, Copy)]
pub enum GamepadButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    Start,
    Select,
    StickLeft,
    StickRight,
    ShoulderLeft,
    ShoulderRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamepadStick {
    StickLeft,
    StickRight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GamepadAxis {
    StickLeftX,
    StickLeftY,
    StickRightX,
    StickRightY,
    TriggerLeft,
    TriggerRight,
}

pub unsafe fn send_key(key: VIRTUAL_KEY, input_type: InputType) {
    let flags: KEYBD_EVENT_FLAGS = match input_type {
        InputType::Up => match key {
            VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT => KEYEVENTF_KEYUP | KEYEVENTF_EXTENDEDKEY,
            _ => KEYEVENTF_KEYUP,
        },
        InputType::Down => match key {
            VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT => KEYEVENTF_EXTENDEDKEY,
            _ => KEYBD_EVENT_FLAGS(0),
        },
    };

    let key_input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    SendInput(&[key_input], size_of::<INPUT>() as i32);
}

pub unsafe fn send_key_raw(key: VIRTUAL_KEY, input_type: InputType) {
    let flags: KEYBD_EVENT_FLAGS = match input_type {
        InputType::Up => match key {
            VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT => {
                KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP | KEYEVENTF_EXTENDEDKEY
            }
            _ => KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP,
        },
        InputType::Down => match key {
            VK_UP | VK_DOWN | VK_LEFT | VK_RIGHT => KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY,
            _ => KEYEVENTF_SCANCODE,
        },
    };

    let key_input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: MapVirtualKeyW(key.0 as u32, MAPVK_VK_TO_VSC) as u16,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    SendInput(&[key_input], size_of::<INPUT>() as i32);
}

pub unsafe fn send_mouse_button(button: MouseButton, input_type: InputType) {
    let mouse_event: MOUSE_EVENT_FLAGS = match button {
        MouseButton::Left => match input_type {
            InputType::Up => MOUSEEVENTF_LEFTUP,
            InputType::Down => MOUSEEVENTF_LEFTDOWN,
        },
        MouseButton::Right => match input_type {
            InputType::Up => MOUSEEVENTF_RIGHTUP,
            InputType::Down => MOUSEEVENTF_RIGHTDOWN,
        },
        MouseButton::Middle => match input_type {
            InputType::Up => MOUSEEVENTF_MIDDLEUP,
            InputType::Down => MOUSEEVENTF_MIDDLEDOWN,
        },
        MouseButton::Extra1 | MouseButton::Extra2 => match input_type {
            InputType::Up => MOUSEEVENTF_XUP,
            InputType::Down => MOUSEEVENTF_XDOWN,
        },
    };

    let mouse_data: u32 = match button {
        MouseButton::Extra1 => XBUTTON1 as u32,
        MouseButton::Extra2 => XBUTTON2 as u32,
        _ => 0,
    };

    let mouse_input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: mouse_data,
                dwFlags: mouse_event,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    SendInput(&[mouse_input], size_of::<INPUT>() as i32);
}

pub unsafe fn send_mouse_scroll(amount: u32, input_type: InputType) {
    let scroll_amount: i32 = match input_type {
        InputType::Up => (WHEEL_DELTA * amount) as i32,
        InputType::Down => (WHEEL_DELTA * amount) as i32 * -1,
    };

    let scroll_input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: 0,
                dy: 0,
                mouseData: transmute::<i32, u32>(scroll_amount),
                dwFlags: MOUSEEVENTF_WHEEL,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    SendInput(&[scroll_input], size_of::<INPUT>() as i32);
}

pub unsafe fn send_mouse_move(x: i32, y: i32) {
    let move_input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: x,
                dy: y,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    SendInput(&[move_input], size_of::<INPUT>() as i32);
}

pub unsafe fn send_gamepad_button(button: GamepadButton, input_type: InputType) {
    XINPUT_STATE_OVERRIDE.dwPacketNumber += 1;

    let mut buttons = XINPUT_STATE_OVERRIDE.Gamepad.wButtons.0;

    if input_type == InputType::Down {
        match button {
            GamepadButton::Up => {
                buttons = buttons | (1 << 0);
            }
            GamepadButton::Down => {
                buttons = buttons | (1 << 1);
            }
            GamepadButton::Left => {
                buttons = buttons | (1 << 2);
            }
            GamepadButton::Right => {
                buttons = buttons | (1 << 3);
            }
            GamepadButton::Start => {
                buttons = buttons | (1 << 4);
            }
            GamepadButton::Select => {
                buttons = buttons | (1 << 5);
            }
            GamepadButton::StickLeft => {
                buttons = buttons | (1 << 6);
            }
            GamepadButton::StickRight => {
                buttons = buttons | (1 << 7);
            }
            GamepadButton::ShoulderLeft => {
                buttons = buttons | (1 << 8);
            }
            GamepadButton::ShoulderRight => {
                buttons = buttons | (1 << 9);
            }
            GamepadButton::A => {
                buttons = buttons | (1 << 12);
            }
            GamepadButton::B => {
                buttons = buttons | (1 << 13);
            }
            GamepadButton::X => {
                buttons = buttons | (1 << 14);
            }
            GamepadButton::Y => {
                buttons = buttons | (1 << 15);
            }
            _ => {}
        }
    } else {
        match button {
            GamepadButton::Up => {
                buttons = buttons & !(1 << 0);
            }
            GamepadButton::Down => {
                buttons = buttons & !(1 << 1);
            }
            GamepadButton::Left => {
                buttons = buttons & !(1 << 2);
            }
            GamepadButton::Right => {
                buttons = buttons & !(1 << 3);
            }
            GamepadButton::Start => {
                buttons = buttons & !(1 << 4);
            }
            GamepadButton::Select => {
                buttons = buttons & !(1 << 5);
            }
            GamepadButton::StickLeft => {
                buttons = buttons & !(1 << 6);
            }
            GamepadButton::StickRight => {
                buttons = buttons & !(1 << 7);
            }
            GamepadButton::ShoulderLeft => {
                buttons = buttons & !(1 << 8);
            }
            GamepadButton::ShoulderRight => {
                buttons = buttons & !(1 << 9);
            }
            GamepadButton::A => {
                buttons = buttons & !(1 << 12);
            }
            GamepadButton::B => {
                buttons = buttons & !(1 << 13);
            }
            GamepadButton::X => {
                buttons = buttons & !(1 << 14);
            }
            GamepadButton::Y => {
                buttons = buttons & !(1 << 15);
            }
            _ => {}
        }
    }

    XINPUT_STATE_OVERRIDE.Gamepad.wButtons = XINPUT_GAMEPAD_BUTTON_FLAGS(buttons);
}

pub unsafe fn send_gamepad_axis(axis: GamepadAxis, amount: i32) {
    XINPUT_STATE_OVERRIDE.dwPacketNumber += 1;

    match axis {
        GamepadAxis::StickLeftX => {
            XINPUT_STATE_OVERRIDE.Gamepad.sThumbLX = amount as i16;
        }
        GamepadAxis::StickLeftY => {
            XINPUT_STATE_OVERRIDE.Gamepad.sThumbLY = amount as i16;
        }
        GamepadAxis::StickRightX => {
            XINPUT_STATE_OVERRIDE.Gamepad.sThumbRX = amount as i16;
        }
        GamepadAxis::StickRightY => {
            XINPUT_STATE_OVERRIDE.Gamepad.sThumbRY = amount as i16;
        }
        GamepadAxis::TriggerLeft => {
            XINPUT_STATE_OVERRIDE.Gamepad.bLeftTrigger = amount as u8;
        }
        GamepadAxis::TriggerRight => {
            XINPUT_STATE_OVERRIDE.Gamepad.bRightTrigger = amount as u8;
        }
        _ => {}
    }
}

pub fn string_to_keycode(name: &str) -> Option<VIRTUAL_KEY> {
    match name.to_lowercase().as_str() {
        "0" => Some(VK_0),
        "1" => Some(VK_1),
        "2" => Some(VK_2),
        "3" => Some(VK_3),
        "4" => Some(VK_4),
        "5" => Some(VK_5),
        "6" => Some(VK_6),
        "7" => Some(VK_7),
        "8" => Some(VK_8),
        "9" => Some(VK_9),
        "a" => Some(VK_A),
        "b" => Some(VK_B),
        "c" => Some(VK_C),
        "d" => Some(VK_D),
        "e" => Some(VK_E),
        "f" => Some(VK_F),
        "g" => Some(VK_G),
        "h" => Some(VK_H),
        "i" => Some(VK_I),
        "j" => Some(VK_J),
        "k" => Some(VK_K),
        "l" => Some(VK_L),
        "m" => Some(VK_M),
        "n" => Some(VK_N),
        "o" => Some(VK_O),
        "p" => Some(VK_P),
        "q" => Some(VK_Q),
        "r" => Some(VK_R),
        "s" => Some(VK_S),
        "t" => Some(VK_T),
        "u" => Some(VK_U),
        "v" => Some(VK_V),
        "w" => Some(VK_W),
        "x" => Some(VK_X),
        "y" => Some(VK_Y),
        "z" => Some(VK_Z),
        "f1" => Some(VK_F1),
        "f2" => Some(VK_F2),
        "f3" => Some(VK_F3),
        "f4" => Some(VK_F4),
        "f5" => Some(VK_F5),
        "f6" => Some(VK_F6),
        "f7" => Some(VK_F7),
        "f8" => Some(VK_F8),
        "f9" => Some(VK_F9),
        "f10" => Some(VK_F10),
        "f11" => Some(VK_F11),
        "f12" => Some(VK_F12),
        "shift" | "shift_l" | "shift_left" => Some(VK_LSHIFT),
        "shift_r" | "shift_right" => Some(VK_RSHIFT),
        "control" | "ctrl" | "control_l" | "ctrl_l" | "control_left" | "ctrl_left" => {
            Some(VK_LCONTROL)
        }
        "control_r" | "ctrl_r" | "control_right" | "ctrl_right" => Some(VK_RCONTROL),
        "alt" | "alt_l" | "alt_left" => Some(VK_LMENU),
        "alt_r" | "alt_right" => Some(VK_RMENU),
        "tab" => Some(VK_TAB),
        "back" | "backspace" => Some(VK_BACK),
        "enter" | "return" => Some(VK_RETURN),
        "caps" | "capslock" => Some(VK_CAPITAL),
        "space" => Some(VK_SPACE),
        "escape" | "esc" => Some(VK_ESCAPE),
        "up" | "arrow_up" => Some(VK_UP),
        "down" | "arrow_down" => Some(VK_DOWN),
        "left" | "arrow_left" => Some(VK_LEFT),
        "right" | "arrow_right" => Some(VK_RIGHT),
        _ => None,
    }
}

pub fn string_to_mousebutton(name: &str) -> Option<MouseButton> {
    match name.to_lowercase().as_str() {
        "left" | "l" => Some(MouseButton::Left),
        "right" | "r" => Some(MouseButton::Right),
        "middle" | "m" => Some(MouseButton::Middle),
        "extra1" | "e1" => Some(MouseButton::Extra1),
        "extra2" | "e2" => Some(MouseButton::Extra2),
        _ => None,
    }
}

pub fn string_to_button(name: &str) -> Option<GamepadButton> {
    match name.to_lowercase().as_str() {
        "up" | "dpad_up" => Some(GamepadButton::Up),
        "down" | "dpad_down" => Some(GamepadButton::Down),
        "left" | "dpad_left" => Some(GamepadButton::Left),
        "right" | "dpad_right" => Some(GamepadButton::Right),
        "a" | "cross" => Some(GamepadButton::A),
        "b" | "circle" => Some(GamepadButton::B),
        "x" | "square" => Some(GamepadButton::X),
        "y" | "triangle" => Some(GamepadButton::Y),
        "start" | "options" => Some(GamepadButton::Start),
        "select" | "share" => Some(GamepadButton::Select),
        "l3" | "stick_l" | "stick_left" => Some(GamepadButton::StickLeft),
        "r3" | "stick_r" | "stick_right" => Some(GamepadButton::StickRight),
        "l1" | "shoulder_l" | "shoulder_left" => Some(GamepadButton::ShoulderLeft),
        "r1" | "shoulder_r" | "shoulder_right" => Some(GamepadButton::ShoulderRight),
        _ => None,
    }
}

pub fn string_to_stick(name: &str) -> Option<GamepadStick> {
    match name.to_lowercase().as_str() {
        "left" | "l" => Some(GamepadStick::StickLeft),
        "right" | "r" => Some(GamepadStick::StickRight),
        _ => None,
    }
}

pub fn string_to_axis(name: &str) -> Option<GamepadAxis> {
    match name.to_lowercase().as_str() {
        "stick_left_x" | "stick_l_x" | "left_x" | "l_x" => Some(GamepadAxis::StickLeftX),
        "stick_left_y" | "stick_l_y" | "left_y" | "l_y" => Some(GamepadAxis::StickLeftY),
        "stick_right_x" | "stick_r_x" | "right_x" | "r_x" => Some(GamepadAxis::StickRightX),
        "stick_right_y" | "stick_r_y" | "right_y" | "r_y" => Some(GamepadAxis::StickRightY),
        "trigger_left" | "trigger_l" | "l2" => Some(GamepadAxis::TriggerLeft),
        "trigger_right" | "trigger_r" | "r2" => Some(GamepadAxis::TriggerRight),
        _ => None,
    }
}
