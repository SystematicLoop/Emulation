use tcod::input::check_for_event;
use tcod::input::KEY     as KEY_EVENT;
use tcod::input::MOUSE   as MOUSE_EVENT;
use tcod::input::KEY_PRESS;
use tcod::input::KEY_RELEASE;
use tcod::input::MOUSE_PRESS;
use tcod::input::MOUSE_RELEASE;
use tcod::input::MOUSE_MOVE;
use tcod::input::Event   as TcodEvent;
use tcod::input::Key     as TcodKey;
use tcod::input::KeyCode as TcodKeyCode;
use tcod::input::Mouse   as TcodMouse;

use crate::{Position, Dimension};

pub struct Input {
    keys_last_frame: [bool; KeyCode::count()],
    keys_this_frame: [bool; KeyCode::count()],
    any_key_down:    Option<KeyCode>,

    mouse_last_frame: [bool; MouseButton::count()],
    mouse_this_frame: [bool; MouseButton::count()],

    mouse: Mouse
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys_last_frame:  [false; KeyCode::count()],
            keys_this_frame:  [false; KeyCode::count()],
            any_key_down:     None,
            mouse_last_frame: [false; MouseButton::count()],
            mouse_this_frame: [false; MouseButton::count()],
            mouse: Mouse::default()
        }
    }

    pub fn update(&mut self, world_size: Dimension, world_offset: Position) {
        for i in 0..KeyCode::count() {
            self.keys_last_frame[i] = self.keys_this_frame[i];
        }

        for i in 0..MouseButton::count() {
            self.mouse_last_frame[i] = self.mouse_this_frame[i];
        }

        self.any_key_down = None;

        loop {
            let event = check_for_event(KEY_EVENT | MOUSE_EVENT);
            match event {
                Some((KEY_PRESS, TcodEvent::Key(tcod_key))) => {
                    let code = KeyCode::from(tcod_key);
                    self.keys_this_frame[code as usize] = true;
                    self.any_key_down = Some(code);
                },

                Some((KEY_RELEASE, TcodEvent::Key(tcod_key))) => {
                    let code = KeyCode::from(tcod_key);
                    self.keys_this_frame[code as usize] = false;
                },

                Some((MOUSE_PRESS, TcodEvent::Mouse(tcod_mouse))) => {
                    let mouse_button = MouseButton::from(tcod_mouse);
                    self.mouse_this_frame[mouse_button as usize] = true;
                },

                Some((MOUSE_RELEASE, TcodEvent::Mouse(tcod_mouse))) => {
                    let mouse_button = MouseButton::from(tcod_mouse);
                    self.mouse_this_frame[mouse_button as usize] = false;
                },

                Some((_, TcodEvent::Mouse(tcod_mouse))) => {
                    self.mouse.pixel_pos = Position::new(tcod_mouse.x  as i32, tcod_mouse.y  as i32);
                    self.mouse.cell_pos  = Position::new(tcod_mouse.cx as i32, tcod_mouse.cy as i32);
                    self.mouse.world_pos = self.mouse.cell_pos.into_world_pos(world_size, world_offset);
                },
                
                None => {
                    // No more events!
                    break;
                },

                _ => {
                    
                },
            }
        }

        if let Some(code) = self.any_key_down {
            // If the key was also pressed last frame, cancel
            // the "any key down" state for that key.
            if self.keys_last_frame[code as usize] {
                self.any_key_down = None;
            }
        }
    }

    pub fn key(&self, code: KeyCode) -> ButtonState {
        let last_frame = self.keys_last_frame[code as usize];
        let this_frame = self.keys_this_frame[code as usize];
        
        ButtonState {
            down: !last_frame &&  this_frame,
            up:    last_frame && !this_frame,
            held:  last_frame &&  this_frame
        }
    }

    pub fn any_key_down(&self) -> Option<KeyCode> {
        self.any_key_down
    }

    pub fn button(&self, button: MouseButton) -> ButtonState {
        let last_frame = self.mouse_last_frame[button as usize];
        let this_frame = self.mouse_this_frame[button as usize];
        
        ButtonState {
            down: !last_frame &&  this_frame,
            up:    last_frame && !this_frame,
            held:  last_frame &&  this_frame
        }
    }

    pub fn mouse(&self) -> Mouse {
        self.mouse
    }
}

#[derive(Debug, Copy, Clone)]
pub enum KeyCode {
    A1, A2, A3, A4, A5, A6, A7, A8, A9, A0,

    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    Right, Up, Left, Down, Space, Escape, Delete,

    // Ensure this is the last item in the list.
    // It is used for determining the number of
    // elements in the enum.
    Unknown
}

impl KeyCode {
    const fn count() -> usize {
        KeyCode::Unknown as usize + 1
    }
}

impl Default for KeyCode {
    fn default() -> Self {
        KeyCode::Unknown
    }
}

impl From<TcodKey> for KeyCode {
    fn from(tcod_key: TcodKey) -> Self {
        match tcod_key.printable.to_ascii_uppercase() {
            '1' => KeyCode::A1, '2' => KeyCode::A2, '3' => KeyCode::A3,
            '4' => KeyCode::A4, '5' => KeyCode::A5, '6' => KeyCode::A6,
            '7' => KeyCode::A7, '8' => KeyCode::A8, '9' => KeyCode::A9,
            '0' => KeyCode::A0,

            'A' => KeyCode::A, 'B' => KeyCode::B, 'C' => KeyCode::C,
            'D' => KeyCode::D, 'E' => KeyCode::E, 'F' => KeyCode::F,
            'G' => KeyCode::G, 'H' => KeyCode::H, 'I' => KeyCode::I,
            'J' => KeyCode::J, 'K' => KeyCode::K, 'L' => KeyCode::L,
            'M' => KeyCode::M, 'N' => KeyCode::N, 'O' => KeyCode::O,
            'P' => KeyCode::P, 'Q' => KeyCode::Q, 'R' => KeyCode::R,
            'S' => KeyCode::S, 'T' => KeyCode::T, 'U' => KeyCode::U,
            'V' => KeyCode::V, 'W' => KeyCode::W, 'X' => KeyCode::X,
            'Y' => KeyCode::Y, 'Z' => KeyCode::Z,
            
            _   => match tcod_key.code {
                TcodKeyCode::Right => KeyCode::Right,
                TcodKeyCode::Up    => KeyCode::Up,
                TcodKeyCode::Left  => KeyCode::Left,
                TcodKeyCode::Down  => KeyCode::Down,

                TcodKeyCode::Spacebar => KeyCode::Space,
                TcodKeyCode::Escape   => KeyCode::Escape,
                TcodKeyCode::Delete   => KeyCode::Delete,
                
                _ => {
                    KeyCode::Unknown
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseButton {
    Left,
    Middle,
    Right,

    // Ensure this is the last item in the list.
    // It is used for determining the number of
    // elements in the enum.
    Unknown
}

impl MouseButton {
    const fn count() -> usize {
        MouseButton::Unknown as usize + 1
    }
}

impl Default for MouseButton {
    fn default() -> Self {
        MouseButton::Unknown
    }
}

impl From<TcodMouse> for MouseButton {
    fn from(tcod_mouse: TcodMouse) -> Self {
        if tcod_mouse.lbutton || tcod_mouse.lbutton_pressed {
            MouseButton::Left
        } else if tcod_mouse.rbutton || tcod_mouse.rbutton_pressed {
            MouseButton::Right
        } else if tcod_mouse.mbutton || tcod_mouse.mbutton_pressed {
            MouseButton::Middle
        } else {
            MouseButton::Unknown
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Mouse {
    pub pixel_pos: Position,
    pub cell_pos:  Position,
    pub world_pos: Position
}

#[derive(Default)]
pub struct ButtonState {
    /// Whether the button was first pressed this frame.
    pub down:    bool,

    /// Whether the button was first released this frame.
    pub up:      bool,

    /// Whether the button is held.
    pub held:    bool
}