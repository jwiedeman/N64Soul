// display.rs
// N64 display handling

use alloc::string::String;
use alloc::format;
use core::fmt::Write;
use crate::n64_sys;

// N64 display buffer address (adjust for real hardware)
const DISPLAY_BUFFER: *mut u16 = 0xA0400000 as *mut u16;
const DISPLAY_WIDTH: usize = 320;
const DISPLAY_HEIGHT: usize = 240;
const CHAR_WIDTH: usize = 8;
const CHAR_HEIGHT: usize = 8;

// Current cursor position
static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

// Full font data for 96 printable ASCII characters (32–127)
// Each entry is an 8-byte array representing an 8×8 bitmap.
// (Using a typical 8×8 VGA font here; replace with your own data if desired.)
static FONT_DATA: [[u8; 8]; 96] = [
    // ASCII 32: ' '
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // ASCII 33: '!'
    [0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x00],
    // ASCII 34: '"'
    [0x36, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00],
    // ASCII 35: '#'
    [0x36, 0x36, 0x7F, 0x36, 0x7F, 0x36, 0x36, 0x00],
    // ASCII 36: '$'
    [0x0C, 0x3E, 0x03, 0x1E, 0x30, 0x1F, 0x0C, 0x00],
    // ASCII 37: '%'
    [0x00, 0x63, 0x33, 0x18, 0x0C, 0x66, 0x63, 0x00],
    // ASCII 38: '&'
    [0x1C, 0x36, 0x1C, 0x6E, 0x3B, 0x33, 0x6E, 0x00],
    // ASCII 39: '\''
    [0x06, 0x06, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00],
    // ASCII 40: '('
    [0x18, 0x0C, 0x06, 0x06, 0x06, 0x0C, 0x18, 0x00],
    // ASCII 41: ')'
    [0x06, 0x0C, 0x18, 0x18, 0x18, 0x0C, 0x06, 0x00],
    // ASCII 42: '*'
    [0x00, 0x36, 0x1C, 0x7F, 0x1C, 0x36, 0x00, 0x00],
    // ASCII 43: '+'
    [0x00, 0x0C, 0x0C, 0x3F, 0x0C, 0x0C, 0x00, 0x00],
    // ASCII 44: ','
    [0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x06, 0x00],
    // ASCII 45: '-'
    [0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00],
    // ASCII 46: '.'
    [0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C, 0x00, 0x00],
    // ASCII 47: '/'
    [0x60, 0x30, 0x18, 0x0C, 0x06, 0x03, 0x01, 0x00],
    // ASCII 48: '0'
    [0x3E, 0x63, 0x73, 0x7B, 0x6F, 0x67, 0x3E, 0x00],
    // ASCII 49: '1'
    [0x0C, 0x0E, 0x0F, 0x0C, 0x0C, 0x0C, 0x3F, 0x00],
    // ASCII 50: '2'
    [0x1E, 0x33, 0x30, 0x1C, 0x06, 0x33, 0x3F, 0x00],
    // ASCII 51: '3'
    [0x1E, 0x33, 0x30, 0x1C, 0x30, 0x33, 0x1E, 0x00],
    // ASCII 52: '4'
    [0x38, 0x3C, 0x36, 0x33, 0x7F, 0x30, 0x78, 0x00],
    // ASCII 53: '5'
    [0x3F, 0x03, 0x1F, 0x30, 0x30, 0x33, 0x1E, 0x00],
    // ASCII 54: '6'
    [0x1C, 0x06, 0x03, 0x1F, 0x33, 0x33, 0x1E, 0x00],
    // ASCII 55: '7'
    [0x3F, 0x33, 0x30, 0x18, 0x0C, 0x0C, 0x0C, 0x00],
    // ASCII 56: '8'
    [0x1E, 0x33, 0x33, 0x1E, 0x33, 0x33, 0x1E, 0x00],
    // ASCII 57: '9'
    [0x1E, 0x33, 0x33, 0x3E, 0x30, 0x18, 0x0E, 0x00],
    // ASCII 58: ':'
    [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00, 0x00],
    // ASCII 59: ';'
    [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x06, 0x00],
    // ASCII 60: '<'
    [0x18, 0x0C, 0x06, 0x03, 0x06, 0x0C, 0x18, 0x00],
    // ASCII 61: '='
    [0x00, 0x00, 0x3F, 0x00, 0x3F, 0x00, 0x00, 0x00],
    // ASCII 62: '>'
    [0x06, 0x0C, 0x18, 0x30, 0x18, 0x0C, 0x06, 0x00],
    // ASCII 63: '?'
    [0x1E, 0x33, 0x30, 0x18, 0x0C, 0x00, 0x0C, 0x00],
    // ASCII 64: '@'
    [0x3E, 0x63, 0x7B, 0x7B, 0x7B, 0x03, 0x1E, 0x00],
    // ASCII 65: 'A'
    [0x0C, 0x1E, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x00],
    // ASCII 66: 'B'
    [0x3F, 0x66, 0x66, 0x3E, 0x66, 0x66, 0x3F, 0x00],
    // ASCII 67: 'C'
    [0x3C, 0x66, 0x03, 0x03, 0x03, 0x66, 0x3C, 0x00],
    // ASCII 68: 'D'
    [0x1F, 0x36, 0x66, 0x66, 0x66, 0x36, 0x1F, 0x00],
    // ASCII 69: 'E'
    [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x46, 0x7F, 0x00],
    // ASCII 70: 'F'
    [0x7F, 0x46, 0x16, 0x1E, 0x16, 0x06, 0x0F, 0x00],
    // ASCII 71: 'G'
    [0x3C, 0x66, 0x03, 0x03, 0x73, 0x66, 0x7C, 0x00],
    // ASCII 72: 'H'
    [0x33, 0x33, 0x33, 0x3F, 0x33, 0x33, 0x33, 0x00],
    // ASCII 73: 'I'
    [0x1E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // ASCII 74: 'J'
    [0x78, 0x30, 0x30, 0x30, 0x33, 0x33, 0x1E, 0x00],
    // ASCII 75: 'K'
    [0x67, 0x66, 0x36, 0x1E, 0x1E, 0x36, 0x67, 0x00],
    // ASCII 76: 'L'
    [0x0F, 0x06, 0x06, 0x06, 0x46, 0x66, 0x7F, 0x00],
    // ASCII 77: 'M'
    [0x63, 0x77, 0x7F, 0x7F, 0x6B, 0x63, 0x63, 0x00],
    // ASCII 78: 'N'
    [0x63, 0x67, 0x6F, 0x7B, 0x73, 0x63, 0x63, 0x00],
    // ASCII 79: 'O'
    [0x1C, 0x36, 0x63, 0x63, 0x63, 0x36, 0x1C, 0x00],
    // ASCII 80: 'P'
    [0x3F, 0x66, 0x66, 0x3F, 0x06, 0x06, 0x0F, 0x00],
    // ASCII 81: 'Q'
    [0x1E, 0x33, 0x33, 0x33, 0x3B, 0x1E, 0x38, 0x00],
    // ASCII 82: 'R'
    [0x3F, 0x66, 0x66, 0x3F, 0x1E, 0x36, 0x67, 0x00],
    // ASCII 83: 'S'
    [0x1E, 0x33, 0x07, 0x0E, 0x38, 0x33, 0x1E, 0x00],
    // ASCII 84: 'T'
    [0x3F, 0x2D, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // ASCII 85: 'U'
    [0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x3F, 0x00],
    // ASCII 86: 'V'
    [0x33, 0x33, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00],
    // ASCII 87: 'W'
    [0x63, 0x63, 0x63, 0x6B, 0x7F, 0x77, 0x63, 0x00],
    // ASCII 88: 'X'
    [0x63, 0x63, 0x36, 0x1C, 0x1C, 0x36, 0x63, 0x00],
    // ASCII 89: 'Y'
    [0x33, 0x33, 0x33, 0x1E, 0x0C, 0x0C, 0x1E, 0x00],
    // ASCII 90: 'Z'
    [0x7F, 0x63, 0x31, 0x18, 0x4C, 0x66, 0x7F, 0x00],
    // ASCII 91: '['
    [0x1E, 0x06, 0x06, 0x06, 0x06, 0x06, 0x1E, 0x00],
    // ASCII 92: '\\'
    [0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xC0, 0x00],
    // ASCII 93: ']'
    [0x1E, 0x18, 0x18, 0x18, 0x18, 0x18, 0x1E, 0x00],
    // ASCII 94: '^'
    [0x08, 0x1C, 0x36, 0x63, 0x00, 0x00, 0x00, 0x00],
    // ASCII 95: '_'
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF],
    // ASCII 96: '`'
    [0x0C, 0x0C, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00],
    // ASCII 97: 'a'
    [0x00, 0x00, 0x1E, 0x30, 0x3E, 0x33, 0x6E, 0x00],
    // ASCII 98: 'b'
    [0x07, 0x06, 0x06, 0x3E, 0x66, 0x66, 0x3B, 0x00],
    // ASCII 99: 'c'
    [0x00, 0x00, 0x1E, 0x33, 0x03, 0x33, 0x1E, 0x00],
    // ASCII 100: 'd'
    [0x38, 0x30, 0x30, 0x3E, 0x33, 0x33, 0x6E, 0x00],
    // ASCII 101: 'e'
    [0x00, 0x00, 0x1E, 0x33, 0x3F, 0x03, 0x1E, 0x00],
    // ASCII 102: 'f'
    [0x1C, 0x36, 0x06, 0x0F, 0x06, 0x06, 0x0F, 0x00],
    // ASCII 103: 'g'
    [0x00, 0x00, 0x6E, 0x33, 0x33, 0x3E, 0x30, 0x1F],
    // ASCII 104: 'h'
    [0x07, 0x06, 0x36, 0x6E, 0x66, 0x66, 0x67, 0x00],
    // ASCII 105: 'i'
    [0x0C, 0x00, 0x0E, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // ASCII 106: 'j'
    [0x18, 0x00, 0x1C, 0x18, 0x18, 0x18, 0x1B, 0x00],
    // ASCII 107: 'k'
    [0x07, 0x06, 0x66, 0x36, 0x1E, 0x36, 0x67, 0x00],
    // ASCII 108: 'l'
    [0x0E, 0x0C, 0x0C, 0x0C, 0x0C, 0x0C, 0x1E, 0x00],
    // ASCII 109: 'm'
    [0x00, 0x00, 0x33, 0x7F, 0x7F, 0x6B, 0x63, 0x00],
    // ASCII 110: 'n'
    [0x00, 0x00, 0x1F, 0x33, 0x33, 0x33, 0x33, 0x00],
    // ASCII 111: 'o'
    [0x00, 0x00, 0x1E, 0x33, 0x33, 0x33, 0x1E, 0x00],
    // ASCII 112: 'p'
    [0x00, 0x00, 0x3B, 0x66, 0x66, 0x3E, 0x06, 0x0F],
    // ASCII 113: 'q'
    [0x00, 0x00, 0x6E, 0x33, 0x33, 0x3E, 0x30, 0x78],
    // ASCII 114: 'r'
    [0x00, 0x00, 0x3B, 0x6E, 0x66, 0x06, 0x0F, 0x00],
    // ASCII 115: 's'
    [0x00, 0x00, 0x3E, 0x03, 0x1E, 0x33, 0x1E, 0x00],
    // ASCII 116: 't'
    [0x08, 0x0C, 0x3E, 0x0C, 0x0C, 0x2C, 0x18, 0x00],
    // ASCII 117: 'u'
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x33, 0x6E, 0x00],
    // ASCII 118: 'v'
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x1E, 0x0C, 0x00],
    // ASCII 119: 'w'
    [0x00, 0x00, 0x63, 0x6B, 0x7F, 0x7F, 0x36, 0x00],
    // ASCII 120: 'x'
    [0x00, 0x00, 0x63, 0x36, 0x1C, 0x36, 0x63, 0x00],
    // ASCII 121: 'y'
    [0x00, 0x00, 0x33, 0x33, 0x33, 0x3E, 0x30, 0x1F],
    // ASCII 122: 'z'
    [0x00, 0x00, 0x3F, 0x19, 0x0C, 0x26, 0x3F, 0x00],
    // ASCII 123: '{'
    [0x38, 0x0C, 0x0C, 0x07, 0x0C, 0x0C, 0x38, 0x00],
    // ASCII 124: '|'
    [0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x18, 0x00],
    // ASCII 125: '}'
    [0x07, 0x0C, 0x0C, 0x38, 0x0C, 0x0C, 0x07, 0x00],
    // ASCII 126: '~'
    [0x6E, 0x3B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    // ASCII 127: DEL (often unused; here blank)
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
];

// Initialize display system
pub fn init() {
    unsafe {
        *(n64_sys::VI_STATUS_REG as *mut u32) = 0x0000320E; // 16-bit color, antialiasing on
        *(n64_sys::VI_WIDTH_REG as *mut u32) = DISPLAY_WIDTH as u32;
        *(n64_sys::VI_ORIGIN_REG as *mut u32) = DISPLAY_BUFFER as u32;
        *(n64_sys::VI_H_SYNC_REG as *mut u32) = 0x000C15A0;
        *(n64_sys::VI_V_SYNC_REG as *mut u32) = 0x0000020D;
        *(n64_sys::VI_X_SCALE_REG as *mut u32) = 0x100 | ((DISPLAY_WIDTH as u32) << 16);
        *(n64_sys::VI_Y_SCALE_REG as *mut u32) = 0x100 | ((DISPLAY_HEIGHT as u32) << 16);
        CURSOR_X = 0;
        CURSOR_Y = 0;
        clear();
    }
}

// Clear the display
pub fn clear() {
    unsafe {
        for i in 0..(DISPLAY_WIDTH * DISPLAY_HEIGHT) {
            *DISPLAY_BUFFER.add(i) = 0;
        }
        CURSOR_X = 0;
        CURSOR_Y = 0;
    }
}

// Convert RGB to N64 16-bit color format
fn rgb_to_n64_color(r: u8, g: u8, b: u8) -> u16 {
    let r5 = ((r as u16) >> 3) & 0x1F;
    let g5 = ((g as u16) >> 3) & 0x1F;
    let b5 = ((b as u16) >> 3) & 0x1F;
    (r5 << 11) | (g5 << 6) | (b5 << 1) | 1
}

// Print a single character at the current cursor position
fn print_char(c: char) {
    unsafe {
        if c == '\n' {
            CURSOR_X = 0;
            CURSOR_Y += CHAR_HEIGHT;
            if CURSOR_Y >= DISPLAY_HEIGHT {
                scroll_display();
                CURSOR_Y = DISPLAY_HEIGHT - CHAR_HEIGHT;
            }
            return;
        }
        // Only print printable characters between ' ' and '~'
        if c < ' ' || c > '~' {
            return;
        }
        let char_index = (c as usize) - (' ' as usize);
        if char_index >= FONT_DATA.len() {
            return;
        }
        let char_data = FONT_DATA[char_index]; // [u8; 8]
        for y in 0..CHAR_HEIGHT {
            let row_data = char_data[y];
            for x in 0..CHAR_WIDTH {
                let pixel_x = CURSOR_X + x;
                let pixel_y = CURSOR_Y + y;
                if pixel_x < DISPLAY_WIDTH && pixel_y < DISPLAY_HEIGHT {
                    let is_pixel_set = (row_data & (0x80 >> x)) != 0;
                    let color = if is_pixel_set { rgb_to_n64_color(255, 255, 255) } else { 0 };
                    *DISPLAY_BUFFER.add(pixel_y * DISPLAY_WIDTH + pixel_x) = color;
                }
            }
        }
        CURSOR_X += CHAR_WIDTH;
        if CURSOR_X >= DISPLAY_WIDTH {
            CURSOR_X = 0;
            CURSOR_Y += CHAR_HEIGHT;
            if CURSOR_Y >= DISPLAY_HEIGHT {
                scroll_display();
                CURSOR_Y = DISPLAY_HEIGHT - CHAR_HEIGHT;
            }
        }
    }
}

// Print a string at the current cursor position
pub fn print(text: &str) {
    for c in text.chars() {
        print_char(c);
    }
}

// Print a string followed by a newline
pub fn print_line(text: &str) {
    print(text);
    print_char('\n');
}

// New: Read input from the N64 controller.
// Returns Some(String) if any button is pressed, otherwise None.
pub fn read_input() -> Option<String> {
    unsafe {
        let controller = n64_sys::read_controller(n64_sys::CONTROLLER_1);
        if controller.buttons != 0 {
            print_line(&format!("[input] buttons: {:#06x}", controller.buttons));
            let mut input = String::new();
            if (controller.buttons & n64_sys::A_BUTTON) != 0 {
                input.push('A');
            }
            if (controller.buttons & n64_sys::B_BUTTON) != 0 {
                input.push('B');
            }
            if (controller.buttons & n64_sys::START_BUTTON) != 0 {
                input.push('\n');
            }
            if (controller.buttons & n64_sys::UP_BUTTON) != 0 {
                input.push('U');
            }
            if (controller.buttons & n64_sys::DOWN_BUTTON) != 0 {
                input.push('D');
            }
            if (controller.buttons & n64_sys::LEFT_BUTTON) != 0 {
                input.push('L');
            }
            if (controller.buttons & n64_sys::RIGHT_BUTTON) != 0 {
                input.push('R');
            }

            if !input.is_empty() {
                return Some(input);
            }
        }
        None
    }
}

// Simple on-screen keyboard driven by the controller D-Pad and buttons.
// Returns `true` when the user presses START to submit the current buffer.
pub fn keyboard_input(buffer: &mut String) -> bool {
    const KEYBOARD: [&str; 4] = [
        "ABCDEFGHIJ",
        "KLMNOPQRST",
        "UVWXYZ0123",
        "456789.,? ",
    ];

    unsafe {
        static mut KB_ROW: usize = 0;
        static mut KB_COL: usize = 0;

        let controller = n64_sys::read_controller(n64_sys::CONTROLLER_1);
        let mut updated = false;

        if (controller.buttons & n64_sys::UP_BUTTON) != 0 && KB_ROW > 0 {
            KB_ROW -= 1;
            updated = true;
        }
        if (controller.buttons & n64_sys::DOWN_BUTTON) != 0 && KB_ROW + 1 < KEYBOARD.len() {
            KB_ROW += 1;
            updated = true;
        }
        if (controller.buttons & n64_sys::LEFT_BUTTON) != 0 && KB_COL > 0 {
            KB_COL -= 1;
            updated = true;
        }
        if (controller.buttons & n64_sys::RIGHT_BUTTON) != 0 && KB_COL + 1 < KEYBOARD[KB_ROW].len() {
            KB_COL += 1;
            updated = true;
        }

        if (controller.buttons & n64_sys::A_BUTTON) != 0 {
            let ch = KEYBOARD[KB_ROW].as_bytes()[KB_COL] as char;
            buffer.push(ch);
            updated = true;
        }
        if (controller.buttons & n64_sys::B_BUTTON) != 0 {
            buffer.pop();
            updated = true;
        }

        if updated {
            // redraw prompt and keyboard
            print_line(&format!("Input: {}", buffer));
            for (r, row) in KEYBOARD.iter().enumerate() {
                let mut line = String::new();
                for (c, ch) in row.chars().enumerate() {
                    if r == KB_ROW && c == KB_COL {
                        line.push('[');
                        line.push(ch);
                        line.push(']');
                    } else {
                        line.push(' ');
                        line.push(ch);
                        line.push(' ');
                    }
                }
                print_line(&line);
            }
        }

        (controller.buttons & n64_sys::START_BUTTON) != 0
    }
}

// Display a simple progress indicator while running inference.
pub fn show_progress(current: usize, total: usize) {
    const THROTTLE: usize = 4;
    static mut COUNT: usize = 0;
    unsafe {
        COUNT = COUNT.wrapping_add(1);
        if COUNT % THROTTLE != 0 && current < total {
            return;
        }
    }
    let bar_width = 20;
    let filled = bar_width * current / total;
    let mut bar = String::new();
    for i in 0..bar_width {
        if i < filled {
            bar.push('#');
        } else {
            bar.push('-');
        }
    }
    print_line(&format!("Working... [{}] {}/{}", bar, current, total));
}

pub fn print_probe_result(off: u64, ok: bool, bytes: &[u8]) {
    // Example: "0x18000000  OK  12 34 56 78 9A BC DE F0"
    use core::fmt::Write;
    let mut buf = heapless::String::<96>::new();
    let _ = write!(
        &mut buf, "0x{off:08X}  {}  ",
        if ok { "OK " } else { "ERR" }
    );
    for b in bytes.iter().take(8) {
        let _ = write!(&mut buf, "{:02X} ", b);
    }
    print_line(buf.as_str());
}

// Scroll the display up by one character row
fn scroll_display() {
    unsafe {
        for y in CHAR_HEIGHT..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let src_idx = y * DISPLAY_WIDTH + x;
                let dst_idx = (y - CHAR_HEIGHT) * DISPLAY_WIDTH + x;
                *DISPLAY_BUFFER.add(dst_idx) = *DISPLAY_BUFFER.add(src_idx);
            }
        }
        for y in (DISPLAY_HEIGHT - CHAR_HEIGHT)..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                *DISPLAY_BUFFER.add(y * DISPLAY_WIDTH + x) = 0;
            }
        }
    }
}
