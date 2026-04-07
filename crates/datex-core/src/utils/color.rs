//! # Terminal Color Support
//! 
//! Colors work across all terminal types with automatic fallbacks:
//! - **4-bit**: Old terminals (cmd.exe, classic console)
//! - **8-bit**: SSH, older Linux terminals  
//! - **24-bit (RGB)**: Modern terminals (Kitty, Alacritty, VSCode, macOS)
//! 
//! # Quick Selection Guide
//! | Terminal Type | Use Method |
//! |--------------|------------|
//! | Modern (truecolor) | `as_ansi_rgb()` |
//! | Remote/SSH | `as_ansi_8_bit()` |
//! | Very old | `as_ansi_4_bit()` |

use crate::prelude::*;

/// Raw ANSI escape code constants for terminal styling.
/// 
/// These are the low-level control sequences.
/// 
/// By looking at them I start remembering assembly code, what a cool language.
pub struct AnsiCodes {}
impl AnsiCodes {
    pub const COLOR_DEFAULT: &'static str = "\x1b[39m";

    pub const CLEAR: &'static str = "\x1b[2J"; // clear screen

    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DEFAULT: &'static str = "\x1b[2m";
    pub const ITALIC: &'static str = "\x1b[3m";
    pub const UNDERLINE: &'static str = "\x1b[4m";
    pub const INVERSE: &'static str = "\x1b[7m";
    pub const HIDDEN: &'static str = "\x1b[8m";

    pub const RESET_UNDERLINE: &'static str = "\x1b[24m";
    pub const RESET_INVERSE: &'static str = "\x1b[27m";

    pub const BLACK: &'static str = "\x1b[30m";
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const WHITE: &'static str = "\x1b[37m";
    pub const GREY: &'static str = "\x1b[90m";

    pub const BG_BLACK: &'static str = "\x1b[40m";
    pub const BG_RED: &'static str = "\x1b[41m";
    pub const BG_GREEN: &'static str = "\x1b[42m";
    pub const BG_YELLOW: &'static str = "\x1b[43m";
    pub const BG_BLUE: &'static str = "\x1b[44m";
    pub const BG_MAGENTA: &'static str = "\x1b[45m";
    pub const BG_CYAN: &'static str = "\x1b[46m";
    pub const BG_WHITE: &'static str = "\x1b[47m";
    pub const BG_GREY: &'static str = "\x1b[100m";
    pub const BG_COLOR_DEFAULT: &'static str = "\x1b[49m";
}

/// Used to describe colors for each possible output and input in terminal.
/// 
/// # Example
/// "Some random text" // This will be highlighted with 'MAGENTA' color
#[derive(PartialEq)]
pub enum Color {
    RED,
    GREEN,
    BLUE,
    YELLOW,
    MAGENTA,
    CYAN,
    BLACK,
    WHITE,
    GREY,

    TEXT,
    NUMBER,
    BUFFER,
    PrimitiveConstant,
    TYPE,
    TIME,

    DEFAULT,
    DefaultLight,
    RESERVED,

    ENDPOINT,
    EndpointPerson,
    EndpointInstitution,

    _UNKNOWN, // imply further color resolution
}

impl Color {

    pub fn as_ansi_rgb_bg(&self) -> String {
        self.as_ansi_rgb()
            .as_str()
            .replacen("38", "48", 1)
            .to_string()
    }

    pub fn as_ansi_rgb(&self) -> String {
        match self {
            Color::RED => ansi_rgb(234, 43, 81),
            Color::GREEN => ansi_rgb(30, 218, 109),
            Color::BLUE => ansi_rgb(6, 105, 193),
            Color::YELLOW => ansi_rgb(235, 182, 38),
            Color::MAGENTA => ansi_rgb(196, 112, 222),
            Color::CYAN => ansi_rgb(79, 169, 232),
            Color::BLACK => ansi_rgb(5, 5, 5),
            Color::WHITE => ansi_rgb(250, 250, 250),
            Color::GREY => ansi_rgb(150, 150, 150),

            Color::TEXT => ansi_rgb(183, 129, 227),
            Color::NUMBER => ansi_rgb(253, 139, 25),
            Color::PrimitiveConstant => ansi_rgb(219, 45, 129),
            Color::BUFFER => ansi_rgb(238, 95, 95),
            Color::TYPE => ansi_rgb(50, 153, 220),
            Color::TIME => ansi_rgb(253, 213, 25),

            Color::ENDPOINT => ansi_rgb(24, 219, 164),
            Color::EndpointPerson => ansi_rgb(41, 199, 61),
            Color::EndpointInstitution => ansi_rgb(135, 201, 36),

            Color::RESERVED => ansi_rgb(65, 102, 238),
            Color::DEFAULT => AnsiCodes::COLOR_DEFAULT.to_string(),
            Color::DefaultLight => ansi_rgb(173, 173, 173),

            Color::_UNKNOWN => ansi_rgb(255, 0, 255), // invalid: magenta
        }
    }

    pub fn as_ansi_8_bit(&self) -> String {
        let (r, g, b) = match self {
            Color::RED => (234, 43, 81),
            Color::GREEN => (30, 218, 109),
            Color::BLUE => (6, 105, 193),
            Color::YELLOW => (235, 182, 38),
            Color::MAGENTA => (196, 112, 222),
            Color::CYAN => (79, 169, 232),
            Color::BLACK => (5, 5, 5),
            Color::WHITE => (250, 250, 250),
            Color::GREY => (150, 150, 150),

            Color::TEXT => (183, 129, 227),
            Color::NUMBER => (253, 139, 25),
            Color::PrimitiveConstant => (219, 45, 129),
            Color::BUFFER => (238, 95, 95),
            Color::TYPE => (50, 153, 220),
            Color::TIME => (253, 213, 25),

            Color::ENDPOINT => (24, 219, 164),
            Color::EndpointPerson => (41, 199, 61),
            Color::EndpointInstitution => (135, 201, 36),

            Color::RESERVED => (65, 102, 238),
            Color::DEFAULT => return AnsiCodes::COLOR_DEFAULT.to_string(),
            Color::DefaultLight => (173, 173, 173),

            Color::_UNKNOWN => (255, 0, 255), // invalid: magenta
        };

        ansi_256(rgb_to_ansi256(r, g, b))
    }

    pub fn as_ansi_8_bit_bg(&self) -> String {
        self.as_ansi_8_bit()
            .replacen("38;5", "48;5", 1)
    }

    pub fn as_ansi_4_bit_bg(&self) -> &'static str {
        match self {
            Color::RED => AnsiCodes::BG_RED,
            Color::GREEN => AnsiCodes::BG_GREEN,
            Color::BLUE => AnsiCodes::BG_BLUE,
            Color::YELLOW => AnsiCodes::BG_YELLOW,
            Color::MAGENTA => AnsiCodes::BG_MAGENTA,
            Color::CYAN => AnsiCodes::BG_CYAN,
            Color::BLACK => AnsiCodes::BG_BLACK,
            Color::WHITE => AnsiCodes::BG_WHITE,
            Color::GREY => AnsiCodes::BG_GREY,

            // There is no colors for that, so I chose the most matching
            Color::TEXT => AnsiCodes::BG_MAGENTA,
            Color::NUMBER => AnsiCodes::BG_YELLOW,
            Color::PrimitiveConstant => AnsiCodes::BG_MAGENTA,
            Color::BUFFER => AnsiCodes::BG_RED,
            Color::TYPE => AnsiCodes::BG_BLUE,
            Color::TIME => AnsiCodes::BG_YELLOW,

            Color::ENDPOINT => AnsiCodes::BG_CYAN,
            Color::EndpointPerson => AnsiCodes::BG_GREEN,
            Color::EndpointInstitution => AnsiCodes::BG_GREEN,

            Color::RESERVED => AnsiCodes::BG_BLUE,
            Color::DEFAULT => AnsiCodes::BG_COLOR_DEFAULT,
            Color::DefaultLight => AnsiCodes::BG_GREY,

            Color::_UNKNOWN => AnsiCodes::BG_MAGENTA,
        }
    }

    pub fn as_ansi_4_bit(&self) -> &'static str {
        match self {
            Color::RED => AnsiCodes::RED,
            Color::GREEN => AnsiCodes::GREEN,
            Color::BLUE => AnsiCodes::BLUE,
            Color::YELLOW => AnsiCodes::YELLOW,
            Color::MAGENTA => AnsiCodes::MAGENTA,
            Color::CYAN => AnsiCodes::CYAN,
            Color::BLACK => AnsiCodes::BLACK,
            Color::WHITE => AnsiCodes::WHITE,
            Color::GREY => AnsiCodes::GREY,

            // There is no colors for that, so I chose the most matching
            Color::TEXT => AnsiCodes::MAGENTA,
            Color::NUMBER => AnsiCodes::YELLOW,
            Color::PrimitiveConstant => AnsiCodes::MAGENTA,
            Color::BUFFER => AnsiCodes::RED,
            Color::TYPE => AnsiCodes::BLUE,
            Color::TIME => AnsiCodes::YELLOW,

            Color::ENDPOINT => AnsiCodes::CYAN,
            Color::EndpointPerson => AnsiCodes::GREEN,
            Color::EndpointInstitution => AnsiCodes::GREEN,

            Color::RESERVED => AnsiCodes::BLUE,
            Color::DEFAULT => AnsiCodes::COLOR_DEFAULT,
            Color::DefaultLight => AnsiCodes::GREY,

            Color::_UNKNOWN => AnsiCodes::MAGENTA,
        }
    }


    // Something like that must be implemented, if support true color, then we will use most modern ansi
    // if supports_truecolor() {
    //     color.as_ansi_rgb()
    // } else {
    //     color.as_ansi_4_bit()
    // }
}

/// Converts RGB values to the nearest 256-color ANSI code.
/// 
/// This uses the standard RGB→ANSI256 mapping algorithm that partitions each color channel
/// into 6 levels (0-5), creating a 6×6×6 color cube (216 colors) plus 40 grayscale shades.
/// 
/// # Arguments
/// * `r` - Red component (0-255)
/// * `g` - Green component (0-255)  
/// * `b` - Blue component (0-255)
/// 
/// # Returns
/// ANSI 256-color code (16-255)
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let r = (r as u16 * 5 / 255) as u8;
    let g = (g as u16 * 5 / 255) as u8;
    let b = (b as u16 * 5 / 255) as u8;

    16 + 36 * r + 6 * g + b
}

/// Creates an ANSI 256-color foreground (text) escape sequence.
pub fn ansi_256(color: u8) -> String {
    format!("\x1b[38;5;{color}m")
}

/// Creates an ANSI 256-color background escape sequence.
pub fn ansi_256_bg(color: u8) -> String {
    format!("\x1b[48;5;{color}m")
}

/// Creates an ANSI 24-bit truecolor RGB escape sequence.
/// 
/// This is the highest quality color mode, supported by most modern terminals
/// (Kitty, Alacritty, VSCode terminal, GNOME Terminal, iTerm2, Windows Terminal).
/// 
/// # Arguments
/// * `r` - Red (0-255)
/// * `g` - Green (0-255)
/// * `b` - Blue (0-255)
pub fn ansi_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

#[test]
fn test_ansi_convert() {
    let output = rgb_to_ansi256(0, 0, 255); // Must return pure blue "021" in ansi256 #0000ff
    assert_eq!(output, 21);

    let output = rgb_to_ansi256(0, 0, 0); // Must return pure black "016" in ansi256 #000000
    assert_eq!(output, 16);

    let output = rgb_to_ansi256(255, 135, 255); // Must return cool pink "213" in ansi256 #ff87ff
    assert_eq!(output, 213);
}