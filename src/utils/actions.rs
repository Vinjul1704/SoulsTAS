use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::utils::input::*;

#[derive(Debug, Clone, Copy)]
pub enum AwaitFlag {
    Control,
    NoControl,
    Cutscene,
    NoCutscene,
    SaveActive,
    NoSaveActive,
    Focus,
}

#[derive(Debug, Clone, Copy)]
pub enum TasActionType {
    Key {
        input_type: InputType,
        key: VIRTUAL_KEY,
    },
    KeyAlternative {
        input_type: InputType,
        key: VIRTUAL_KEY,
    },
    MouseButton {
        input_type: InputType,
        button: MouseButton,
    },
    MouseScroll {
        input_type: InputType,
        amount: u32,
    },
    MouseMove {
        x: i32,
        y: i32,
    },
    GamepadButton {
        input_type: InputType,
        button: GamepadButton,
    },
    GamepadAxis {
        axis: GamepadAxis,
        amount: i32,
    },
    GamepadStick {
        stick: GamepadStick,
        angle: f32,
        amount: f32,
    },
    Nothing,
    Fps {
        fps: f32,
    },
    Await {
        flag: AwaitFlag,
    },
    Frame {
        frame: u32,
    },
    PauseMs {
        ms: u64,
    },
    PauseInput,
}

#[derive(Debug, Clone, Copy)]
pub struct TasAction {
    pub frame: u32,
    pub action: TasActionType,
}

#[derive(Debug, Clone, Copy)]
pub enum FrameType {
    Absolute,
    Relative,
    RelativeAbsolute,
}

#[derive(Debug, Clone, Copy)]
pub struct TasActionInfo {
    pub frame: u32,
    pub frame_type: FrameType,
    pub action: TasActionType,
}

// Tries to parse a TAS script line
// Returns info about the action if successful
// Otherwise returns None if it's empty or a comment
// Returns an error if it can't be parsed
pub fn parse_action(input: &str) -> Result<Option<TasActionInfo>, &str> {
    // Remove comments from input
    let input_uncommented: &str = if let Some(x) = input.find(&[';', '#']) {
        input.split_at(x).0
    } else {
        input
    };

    // Split input into the parts, separated by spaces
    let mut input_parts = input_uncommented.split_whitespace();

    // Get frame number part
    let frame_str: &str = if let Some(x) = input_parts.next() {
        x
    } else {
        return Ok(None);
    };

    // Parse it and check the frame type
    let (frame, frame_type): (u32, FrameType) = if frame_str.starts_with("+++") {
        return Err("Invalid frame");
    } else if frame_str.starts_with("++") {
        (
            if let Ok(x) = frame_str.split_at(2).1.parse::<u32>() {
                x
            } else {
                return Err("Invalid frame");
            },
            FrameType::RelativeAbsolute,
        )
    } else if frame_str.starts_with("+") {
        (
            if let Ok(x) = frame_str.split_at(1).1.parse::<u32>() {
                x
            } else {
                return Err("Invalid frame");
            },
            FrameType::Relative,
        )
    } else {
        (
            if let Ok(x) = frame_str.parse::<u32>() {
                x
            } else {
                return Err("Invalid frame");
            },
            FrameType::Absolute,
        )
    };

    // Get action part
    let action_type_str: &str = if let Some(x) = input_parts.next() {
        x
    } else {
        return Err("Can't parse action");
    };

    // Get remaining parameters
    let params: Vec<&str> = input_parts.collect();

    // Parse actions
    let action: TasActionType = match action_type_str.to_lowercase().as_str() {
        "key" => {
            if params.len() != 2 {
                return Err("Invalid parameter count");
            }

            TasActionType::Key {
                input_type: match params[0].to_lowercase().as_str() {
                    "up" => InputType::Up,
                    "down" => InputType::Down,
                    _ => {
                        return Err("Invalid input type");
                    }
                },
                key: if let Some(x) = string_to_keycode(params[1]) {
                    x
                } else {
                    return Err("Invalid key");
                },
            }
        }
        "key_alternative" => {
            if params.len() != 2 {
                return Err("Invalid parameter count");
            }

            TasActionType::KeyAlternative {
                input_type: match params[0].to_lowercase().as_str() {
                    "up" => InputType::Up,
                    "down" => InputType::Down,
                    _ => {
                        return Err("Invalid input type");
                    }
                },
                key: if let Some(x) = string_to_keycode(params[1]) {
                    x
                } else {
                    return Err("Invalid key");
                },
            }
        }
        "mouse" => {
            if params.len() < 1 {
                return Err("Invalid parameter count");
            }

            match params[0] {
                "button" => {
                    if params.len() != 3 {
                        return Err("Invalid parameter count");
                    }

                    TasActionType::MouseButton {
                        input_type: match params[1].to_lowercase().as_str() {
                            "up" => InputType::Up,
                            "down" => InputType::Down,
                            _ => {
                                return Err("Invalid input type");
                            }
                        },
                        button: match params[2].to_lowercase().as_str() {
                            "left" | "l" => MouseButton::Left,
                            "right" | "r" => MouseButton::Right,
                            "middle" | "m" => MouseButton::Middle,
                            "extra1" | "e1" => MouseButton::Extra1,
                            "extra2" | "e2" => MouseButton::Extra2,
                            _ => {
                                return Err("Invalid button");
                            }
                        },
                    }
                },
                "scroll" => {
                    if params.len() != 3 {
                        return Err("Invalid parameter count");
                    }

                    TasActionType::MouseScroll {
                        input_type: match params[1].to_lowercase().as_str() {
                            "up" => InputType::Up,
                            "down" => InputType::Down,
                            _ => {
                                return Err("Invalid input type");
                            }
                        },
                        amount: if let Ok(x) = params[2].parse::<u32>() {
                            x
                        } else {
                            return Err("Invalid scroll amount");
                        },
                    }
                },
                "move" => {
                    if params.len() != 3 {
                        return Err("Invalid parameter count");
                    }

                    TasActionType::MouseMove {
                        x: if let Ok(x) = params[1].parse::<i32>() {
                            x
                        } else {
                            return Err("Invalid X amount");
                        },
                        y: if let Ok(x) = params[2].parse::<i32>() {
                            x
                        } else {
                            return Err("Invalid Y amount");
                        },
                    }
                },
                _ => {
                    return Err("Invalid mouse action type");
                },
            }
        }
        "gamepad" => {
            if params.len() < 3 {
                return Err("Invalid parameter count");
            }

            match params[0] {
                "button" => {
                    if params.len() != 3 {
                        return Err("Invalid parameter count");
                    }

                    TasActionType::GamepadButton {
                        input_type: match params[1].to_lowercase().as_str() {
                            "up" => InputType::Up,
                            "down" => InputType::Down,
                            _ => {
                                return Err("Invalid input type");
                            }
                        },
                        button: if let Some(x) = string_to_button(params[2]) {
                            x
                        } else {
                            return Err("Invalid button");
                        },
                    }
                },
                "stick" => {
                    if params.len() != 4 {
                        return Err("Invalid parameter count");
                    }

                    if let Some(axis) = string_to_stick(params[1]) {
                        TasActionType::GamepadStick {
                            stick: axis,
                            angle: if let Ok(x) = params[2].parse::<f32>() {
                                x
                            } else {
                                return Err("Invalid angle");
                            },
                            amount: if let Ok(x) = params[3].parse::<f32>() {
                                if x >= 0.0 && x <= 1.0 {
                                    x
                                } else {
                                    return Err("Invalid amount");
                                }
                            } else {
                                return Err("Invalid amount");
                            },
                        }
                    } else {
                        return Err("Invalid stick");
                    }
                },
                "axis" => {
                    if params.len() != 3 {
                        return Err("Invalid parameter count");
                    }

                    if let Some(axis) = string_to_axis(params[1]) {
                        TasActionType::GamepadAxis {
                            axis: axis,
                            amount: if let Ok(x) = params[2].parse::<i32>() {
                                match axis {
                                    GamepadAxis::StickLeftX | GamepadAxis::StickLeftY | GamepadAxis::StickRightX | GamepadAxis::StickRightY => {
                                        if x < -32768 && x > 32767 {
                                            return Err("Invalid amount");
                                        }
                                    },
                                    GamepadAxis::TriggerLeft | GamepadAxis::TriggerRight => {
                                        if x < 0 && x > 255 {
                                            return Err("Invalid amount");
                                        }
                                    },
                                    _ => {
                                        return Err("Invalid axis");
                                    }
                                }
                                x
                            } else {
                                return Err("Invalid amount");
                            },
                        }
                    } else {
                        return Err("Invalid axis");
                    }
                },
                _ => {
                    return Err("Invalid gamepad action type");
                },
            }
        }
        "nothing" => {
            if params.len() != 0 {
                return Err("Invalid parameter count");
            }

            TasActionType::Nothing
        }
        "fps" => {
            if params.len() != 1 {
                return Err("Invalid parameter count");
            }

            TasActionType::Fps {
                fps: if let Ok(x) = params[0].parse::<f32>() {
                    x
                } else {
                    return Err("Invalid FPS");
                },
            }
        }
        "await" => {
            if params.len() != 1 {
                return Err("Invalid parameter count");
            }

            TasActionType::Await {
                flag: match params[0].to_lowercase().as_str() {
                    "control" => AwaitFlag::Control,
                    "no_control" => AwaitFlag::NoControl,
                    "cutscene" => AwaitFlag::Cutscene,
                    "no_cutscene" => AwaitFlag::NoCutscene,
                    "save_active" => AwaitFlag::SaveActive,
                    "no_save_active" => AwaitFlag::NoSaveActive,
                    "focus" => AwaitFlag::Focus,
                    _ => {
                        return Err("Invalid await flag");
                    }
                },
            }
        }
        "frame" => {
            if params.len() != 1 {
                return Err("Invalid parameter count");
            }

            TasActionType::Frame {
                frame: if let Ok(x) = params[0].parse::<u32>() {
                    x
                } else {
                    return Err("Invalid frame");
                },
            }
        }
        "pause" => {
            if params.len() < 1 {
                return Err("Invalid parameter count");
            }

            match params[0] {
                "ms" => {
                    if params.len() != 2 {
                        return Err("Invalid parameter count");
                    }

                    TasActionType::PauseMs {
                        ms: if let Ok(x) = params[1].parse::<u64>() {
                            x
                        } else {
                            return Err("Invalid ms");
                        },
                    }
                },
                "input" => {
                    if params.len() != 1 {
                        return Err("Invalid parameter count");
                    }

                    TasActionType::PauseInput
                },
                _ => {
                    return Err("Invalid pause action type");
                },
            }
        }
        _ => {
            return Err("Invalid action");
        }
    };

    // Finally return action info
    return Ok(Some(TasActionInfo {
        frame: frame,
        frame_type: frame_type,
        action: action,
    }));
}
