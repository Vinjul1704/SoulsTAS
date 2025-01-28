use std::mem::*;

use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::utils::actions::*;

#[derive(Debug, Clone, Copy)]
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

pub unsafe fn send_key(key: VIRTUAL_KEY, input_type: InputType) {
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

// Yes, you are allowed to shittalk me for this.
// Yes, I will be sad if you do.
// Yes, I deserve it anyway.
// Yes, I was very tired and lazy when I did this.
pub fn string_to_keycode(key_name: &str) -> Option<VIRTUAL_KEY> {
    match key_name.to_lowercase().as_str() {
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
        "shift" | "shift_l" => Some(VK_LSHIFT),
        "shift_r" => Some(VK_RSHIFT),
        "control" | "ctrl" | "control_l" | "ctrl_l" => Some(VK_LCONTROL),
        "control_r" | "ctrl_r" => Some(VK_RCONTROL),
        "alt" | "alt_l" => Some(VK_LMENU),
        "alt_r" => Some(VK_RMENU),
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
