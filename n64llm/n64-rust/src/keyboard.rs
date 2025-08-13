#![no_std]

use alloc::string::String;
use crate::display;
use crate::n64_sys;

const KEY_ROWS: usize = 3;
const KEY_COLS: usize = 10;
const LAYOUT: [[char; KEY_COLS]; KEY_ROWS] = [
    ['a','b','c','d','e','f','g','h','i','j'],
    ['k','l','m','n','o','p','q','r','s','t'],
    ['u','v','w','x','y','z',' ','-','.','?'],
];

pub struct OnScreenKeyboard {
    cursor_x: usize,
    cursor_y: usize,
}

impl OnScreenKeyboard {
    pub fn new() -> Self {
        OnScreenKeyboard { cursor_x: 0, cursor_y: 0 }
    }

    pub fn draw(&self) {
        let start = display::screen_lines().saturating_sub(KEY_ROWS + 1);
        for (row_idx, row) in LAYOUT.iter().enumerate() {
            display::set_cursor(0, start + row_idx);
            let mut line = String::new();
            for (col_idx, ch) in row.iter().enumerate() {
                if self.cursor_x == col_idx && self.cursor_y == row_idx {
                    line.push('[');
                    line.push(*ch);
                    line.push(']');
                } else {
                    line.push(' ');
                    line.push(*ch);
                    line.push(' ');
                }
            }
            display::print_line(&line);
        }
    }

    pub fn handle_input(&mut self, buttons: u16, buffer: &mut String) -> bool {
        if (buttons & n64_sys::START_BUTTON) != 0 {
            return true;
        }
        if (buttons & n64_sys::A_BUTTON) != 0 {
            buffer.push(LAYOUT[self.cursor_y][self.cursor_x]);
        }
        if (buttons & n64_sys::B_BUTTON) != 0 {
            buffer.pop();
        }
        if (buttons & n64_sys::LEFT_BUTTON) != 0 && self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
        if (buttons & n64_sys::RIGHT_BUTTON) != 0 && self.cursor_x + 1 < KEY_COLS {
            self.cursor_x += 1;
        }
        if (buttons & n64_sys::UP_BUTTON) != 0 && self.cursor_y > 0 {
            self.cursor_y -= 1;
        }
        if (buttons & n64_sys::DOWN_BUTTON) != 0 && self.cursor_y + 1 < KEY_ROWS {
            self.cursor_y += 1;
        }
        false
    }
}
