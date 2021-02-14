#![allow(unused)]

pub const END_OF_TRANSMISSION: char = '\u{4}';  // Ctrl+D
pub const DEVICE_CONTROL3:     char = '\u{13}'; // Ctrl+End
pub const CANCEL:              char = '\u{18}'; // Ctrl+Home
pub const END_OF_MEDIUM:       char = '\u{19}'; // Shift+F5
pub const SUBSTITUDE:          char = '\u{1a}'; // Shift+F6
pub const FILE_SEPARATOR:      char = '\u{1c}'; // Shift+F8
pub const ESCAPE:              char = '\u{1b}'; // Shift+F7

pub const PAIR_NORMAL:              u8 =  1;
pub const PAIR_INVERTED:            u8 =  2;
pub const PAIR_OFFSETS:             u8 =  3;
pub const PAIR_NON_ASCII:           u8 =  4;
pub const PAIR_CURSOR:              u8 =  5;
pub const PAIR_SELECTION:           u8 =  6;
pub const PAIR_SELECTED_CURSOR:     u8 =  7;
pub const PAIR_INPUT_ERROR:         u8 =  8;
pub const PAIR_SELECTION_MATCH:     u8 =  9;
pub const PAIR_AUTO_COMPLETE:       u8 = 10;
pub const PAIR_ERROR_MESSAGE:       u8 = 11;
pub const PAIR_SEARCH_MATCH:        u8 = 12;
pub const PAIR_SEARCH_MATCH_CURSOR: u8 = 13;
