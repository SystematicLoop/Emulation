use tcod::console::*;
use crate::{Graphics, Input, KeyCode};

pub enum MenuResult<T: Copy> {
    Selected(T),
    NoResponse,
    Cancel
}

pub struct MenuOption<T: Copy> {
    text: String,
    item: T
}

pub struct Menu<T: Copy> {
    pub prompt:  String,
    pub options: Vec<MenuOption<T>>
}

pub struct MenuBuilder<T: Copy> {
    prompt:  String,
    options: Vec<MenuOption<T>>
}

impl<T: Copy> MenuBuilder<T> {
    pub fn new() -> Self {
        MenuBuilder {
            prompt:  String::from("Select an Option"),
            options: Vec::new()
        }
    }

    pub fn with_prompt(mut self, prompt: String) -> Self {
        self.prompt = prompt;
        self
    }

    pub fn with_option(mut self, text: String, item: T) -> Self {
        self.options.push(MenuOption {
            text,
            item
        });

        self
    }
    
    pub fn build(self) -> Menu<T> {
        Menu {
            prompt:  self.prompt,
            options: self.options
        }
    }
}

impl<T: Copy> Menu<T> {
    pub fn show(&self, graphics: &mut Graphics, input: &Input) -> MenuResult<T> {
        graphics.root.clear();
        graphics.root.print(1, 1, format!("{}", self.prompt));

        for (i, option) in self.options.iter().enumerate() {
            graphics.root.print(1, 2 + i as i32, format!("{} {}", i + 1, option.text));
        }

        graphics.root.flush();

        match input.any_key_down() {
            Some(code) => {
                let index = match code {
                    KeyCode::A1 => 0,
                    KeyCode::A2 => 1,
                    KeyCode::A3 => 2,
                    KeyCode::A4 => 3,
                    KeyCode::A5 => 4,
                    KeyCode::A6 => 5,
                    KeyCode::A7 => 6,
                    KeyCode::A8 => 7,
                    KeyCode::A9 => 8,
                    KeyCode::A0 => 9,

                    KeyCode::Escape => {
                        return MenuResult::Cancel;
                    }

                    _ => {
                        return MenuResult::NoResponse;
                    }
                };

                let option = self.options.get(index);
                match option {
                    Some(option) => {
                        return MenuResult::Selected(option.item);
                    },

                    None => {
                        return MenuResult::NoResponse;
                    }
                }
            },

            None => {
                return MenuResult::NoResponse;
            }
        }
    }
}