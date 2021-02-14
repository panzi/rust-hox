use std::fs::File;
use std::fmt::Write;
use std::cmp::{min, max};

#[allow(unused)]
use pancurses_result::{
    initscr, Input, Dimension, Curses, Window,
    Attribute, ColorPair, CursorVisibility,
    COLOR_BLACK, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN,
    COLOR_MAGENTA, COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};

use crate::mmap::MMap;
use crate::result::{Result, Error};
use crate::number_input::NumberInput;
use crate::file_input::FileInput;
use crate::text_box::{TextBox, TextBoxResult};
use crate::search_widget::{SearchWidget, SearchMode};
use crate::consts::*;
use crate::input_widget::{InputWidget, WidgetResult};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

const MASK_SEARCH:          u8 =  1;
const MASK_SEARCH_END:      u8 =  2;
const MASK_HIGHLIGHT:       u8 =  4;
const MASK_HIGHLIGHT_END:   u8 =  8;
const MASK_SELECTED:        u8 = 16;
const MASK_SELECTED_END:    u8 = 32;

const FILE_INPUT_LABEL: &str = "Filename: ";
const SEARCH_LABEL: &str = "Search: ";

fn put_label(window: &mut Window, text: &str) -> Result<()> {
    let mut slice = text;
    while slice.len() > 0 {
        if let Some(index) = slice.find('&') {
            if index + 1 >= slice.len() {
                return Err(Error::message(format!("illegal label: {:?}", text)));
            }
            let tail = &slice[index + 1..];
            if tail.starts_with("&&") {
                window.put_str(&slice[..index])?;
                window.turn_on_attributes(Attribute::Underline)?;
                window.put_char('&')?;
                window.turn_off_attributes(Attribute::Underline)?;
                slice = &slice[index + 3..];
            } else if tail.starts_with('&') {
                window.put_str(&slice[..index + 1])?;
                slice = &slice[index + 1..];
            } else {
                window.put_str(&slice[..index])?;
                window.turn_on_attributes(Attribute::Underline)?;
                window.put_str(&slice[index + 1..index + 2])?;
                window.turn_off_attributes(Attribute::Underline)?;
                slice = &slice[index + 2..];
            }
        } else {
            window.put_str(slice)?;
            break;
        }
    }

    Ok(())
}

fn get_u8(mem: &[u8], cursor: usize) -> Option<u8> {
    if cursor < mem.len() {
        Some(mem[cursor])
    } else {
        None
    }
}

fn get_i8(mem: &[u8], cursor: usize) -> Option<i8> {
    if cursor < mem.len() {
        Some(mem[cursor] as i8)
    } else {
        None
    }
}

fn get_u16(mem: &[u8], cursor: usize, endian: Endian) -> Option<u16> {
    if cursor + 2 <= mem.len() {
        let mem = [mem[cursor], mem[cursor + 1]];
        Some(match endian {
            Endian::Big    => u16::from_be_bytes(mem),
            Endian::Little => u16::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_i16(mem: &[u8], cursor: usize, endian: Endian) -> Option<i16> {
    if cursor + 2 <= mem.len() {
        let mem = [mem[cursor], mem[cursor + 1]];
        Some(match endian {
            Endian::Big    => i16::from_be_bytes(mem),
            Endian::Little => i16::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_u32(mem: &[u8], cursor: usize, endian: Endian) -> Option<u32> {
    if cursor + 4 <= mem.len() {
        let mem = [mem[cursor], mem[cursor + 1], mem[cursor + 2], mem[cursor + 3]];
        Some(match endian {
            Endian::Big    => u32::from_be_bytes(mem),
            Endian::Little => u32::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_i32(mem: &[u8], cursor: usize, endian: Endian) -> Option<i32> {
    if cursor + 4 <= mem.len() {
        let mem = [mem[cursor], mem[cursor + 1], mem[cursor + 2], mem[cursor + 3]];
        Some(match endian {
            Endian::Big    => i32::from_be_bytes(mem),
            Endian::Little => i32::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_u64(mem: &[u8], cursor: usize, endian: Endian) -> Option<u64> {
    if cursor + 8 <= mem.len() {
        let mem = [
            mem[cursor    ], mem[cursor + 1], mem[cursor + 2], mem[cursor + 3],
            mem[cursor + 4], mem[cursor + 5], mem[cursor + 6], mem[cursor + 7],
        ];
        Some(match endian {
            Endian::Big    => u64::from_be_bytes(mem),
            Endian::Little => u64::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_i64(mem: &[u8], cursor: usize, endian: Endian) -> Option<i64> {
    if cursor + 8 <= mem.len() {
        let mem = [
            mem[cursor    ], mem[cursor + 1], mem[cursor + 2], mem[cursor + 3],
            mem[cursor + 4], mem[cursor + 5], mem[cursor + 6], mem[cursor + 7],
        ];
        Some(match endian {
            Endian::Big    => i64::from_be_bytes(mem),
            Endian::Little => i64::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_f32(mem: &[u8], cursor: usize, endian: Endian) -> Option<f32> {
    if cursor + 4 <= mem.len() {
        let mem = [mem[cursor], mem[cursor + 1], mem[cursor + 2], mem[cursor + 3]];
        Some(match endian {
            Endian::Big    => f32::from_be_bytes(mem),
            Endian::Little => f32::from_le_bytes(mem),
        })
    } else {
        None
    }
}

fn get_f64(mem: &[u8], cursor: usize, endian: Endian) -> Option<f64> {
    if cursor + 8 <= mem.len() {
        let mem = [
            mem[cursor    ], mem[cursor + 1], mem[cursor + 2], mem[cursor + 3],
            mem[cursor + 4], mem[cursor + 5], mem[cursor + 6], mem[cursor + 7],
        ];
        Some(match endian {
            Endian::Big    => f64::from_be_bytes(mem),
            Endian::Little => f64::from_le_bytes(mem),
        })
    } else {
        None
    }
}

// TODO: is there a better way?
fn hex_len(mut num: usize) -> usize {
    if num == 0 {
        return 1;
    }

    let mut len = 0;

    while num > 0 {
        num >>= 4;
        len += 1;
    }

    len
}


fn set_search_mask(view_mask: &mut [u8], view_offset: usize, mem: &[u8], needle: &[u8], mask_match: u8, mask_end: u8) {
    let needle_len = needle.len();
    if needle_len > 0 {
        let view_size = view_mask.len();
        let size = mem.len();
        let start_offset = if view_offset > (needle_len - 1) {
            view_offset + 1 - needle_len
        } else {
            0
        };
        let end_offset = view_offset + view_size + needle_len - 1;
        let end_offset = if end_offset <= size {
            end_offset
        } else {
            size + 1 - needle_len
        };

        let view_end_offset = min(view_offset + view_size, size);
        for offset in start_offset..end_offset {
            if &mem[offset..offset + needle_len] == needle {
                let match_offset_start = max(view_offset, offset);
                let match_offset_end   = min(view_end_offset, offset + needle_len);

                let first_view_index = match_offset_start - view_offset;
                let last_view_index  = match_offset_end - view_offset - 1;

                if first_view_index < last_view_index {
                    for item in &mut view_mask[first_view_index..last_view_index] {
                        *item = (*item & !mask_end) | mask_match;
                    }
                }

                if view_mask[last_view_index] & mask_match == 0 {
                    view_mask[last_view_index] |= mask_end | mask_match;
                }
            }
        }
    }
}

pub struct Hox<'a> {
    mmap: MMap<'a>,
    curses:   Curses,
    win_size: Dimension,
    view_offset:     usize,
    view_size:       usize,
    cursor:          usize,
    selection_start: usize,
    selection_end:   usize,
    bytes_per_row:   usize,
    offset_hex_len:  usize,
    const_space:     usize,
    need_redraw:     bool,
    buf: String,
    endian: Endian,
    signed: bool,
    selecting: bool,
    view_mask: Vec<u8>,
    view_mask_valid: bool,
    offset_input: NumberInput<usize>,
    file_input: FileInput,
    help_box: TextBox<'a>,
    help_shown: bool,
    error: Option<String>,
    search_widget: SearchWidget,
    search_data: Vec<u8>,
}

impl<'a> Hox<'a> {
    pub fn new(file: &'a mut File) -> Result<Self> {
        let meta = file.metadata()?;

        let mut curses = initscr()?;
    
        curses.set_echo_input(false)?;
        curses.set_cursor_visibility(CursorVisibility::Invisible)?;
        curses.start_color()?;
    
        let window = curses.window_mut();
    
        window.read_interpolate_function_keys(true)?;
        let size = meta.len();
        if size > std::usize::MAX as u64 {
            return Err(Error::message(format!("file size too big: {} > {}", size, std::usize::MAX)));
        }

        let size = size as usize;
        let mmap = MMap::new(file, 0, size)?;

        let offset_hex_len = hex_len(size);
        let const_space = offset_hex_len + 5;

        let colors = curses.color_mut();

        colors.set_color_pair(PAIR_NORMAL              as i16, COLOR_WHITE, COLOR_BLACK)?;
        colors.set_color_pair(PAIR_INVERTED            as i16, COLOR_BLACK, COLOR_WHITE)?;
        colors.set_color_pair(PAIR_OFFSETS             as i16, 130,         COLOR_BLACK)?;
        colors.set_color_pair(PAIR_NON_ASCII           as i16, 239,         COLOR_BLACK)?;
        colors.set_color_pair(PAIR_CURSOR              as i16, COLOR_WHITE, COLOR_RED)?;
        colors.set_color_pair(PAIR_SELECTION           as i16, COLOR_BLACK, COLOR_BLUE)?;
        colors.set_color_pair(PAIR_SELECTED_CURSOR     as i16, COLOR_WHITE, 128)?;
        colors.set_color_pair(PAIR_INPUT_ERROR         as i16, COLOR_WHITE, COLOR_RED)?;
        colors.set_color_pair(PAIR_SELECTION_MATCH     as i16, COLOR_WHITE, 236)?;
        colors.set_color_pair(PAIR_AUTO_COMPLETE       as i16, 235,         COLOR_BLACK)?;
        colors.set_color_pair(PAIR_ERROR_MESSAGE       as i16, COLOR_RED,   COLOR_BLACK)?;
        colors.set_color_pair(PAIR_SEARCH_MATCH        as i16, COLOR_BLACK, 202)?;
        colors.set_color_pair(PAIR_SEARCH_MATCH_CURSOR as i16, COLOR_BLACK, 197)?;
        
        Ok(Self {
            mmap,
            curses,
            win_size: Dimension::from((0, 0)),
            view_offset: 0,
            view_size: 0,
            cursor: 0,
            selection_start: 0,
            selection_end: 0,
            bytes_per_row: 0,
            offset_hex_len,
            const_space,
            need_redraw: true,
            buf: String::new(),
            endian: Endian::Little,
            signed: false,
            selecting: false,
            view_mask: Vec::new(),
            view_mask_valid: false,
            offset_input: NumberInput::<usize>::new(16),
            file_input: FileInput::new(0),
            help_box: TextBox::new("\
Hotkeys
═══════
h or F1 ... show this help message
q ......... quit
e ......... toggle between big and little endian
i ......... toggle between signed and unsinged
o ......... enter offset to jump to
s ......... toggle select mode
S ......... clear selection
w ......... write selection to file
f or F3 ... open search bar (and search for current selection)
F ......... clear search
n ......... find next
p ......... find previous

Search
──────
Enter or F3 ... find (next)
F5 ............ switch through modes: Text/Binary/Integer
Shift+F5 ...... switch through modes in reverse
F6 ............ switch through integer sizes: 8/16/32/64
F7 ............ toggle signed/unsigned
F8 ............ toggle little endian/big endian
Escape ........ stop search

Non-Text Search
───────────────
Escape or q ... stop search
(and all other global hotkeys)

Navigation
──────────
← ↑ ↓ → ..... move cursor
Home ........ move cursor to start of line
End ......... move cursor to end of line
Ctr+Home .... move cursor to start of file
Ctr+End ..... move cursor to end of file
Page Up ..... move view up one page
Page Down ... move view down one page

Press Enter, Escape or any normal key to clear errors.


© 2021 Mathias Panzenböck", 2, 1,
            ),
            help_shown: false,
            error: None,
            search_widget: SearchWidget::new(0),
            search_data: Vec::new(),
        })
    }

    pub fn set_signed(&mut self, signed: bool) {
        self.signed = signed;
        self.need_redraw = true;
    }

    pub fn set_endian(&mut self, endian: Endian) {
        self.endian = endian;
        self.need_redraw = true;
    }

    pub fn set_cursor(&mut self, mut cursor: usize) {
        let size = self.mmap.size();

        if size > 0 {
            if cursor >= size {
                cursor = size - 1;
            }

            if cursor != self.cursor {
                if self.selecting {
                    if cursor > self.cursor {
                        if self.cursor + 1 == self.selection_end {
                            self.selection_end = cursor + 1;
                        } else if cursor >= self.selection_end {
                            self.selection_start = self.selection_end - 1;
                            self.selection_end   = cursor + 1;
                        } else {
                            self.selection_start = cursor;

                            if self.selection_end <= self.selection_start {
                                self.selection_end = cursor + 1;
                            }
                        }
                    } else {
                        if self.cursor == self.selection_start {
                            self.selection_start = cursor;
                        } else if cursor < self.selection_start {
                            self.selection_end   = self.selection_start + 1;
                            self.selection_start = cursor;
                        } else {
                            self.selection_end = cursor + 1;

                            if self.selection_end <= self.selection_start {
                                self.selection_start = cursor;
                            }
                        }
                    }
                }

                self.cursor = cursor;
                self.need_redraw = true;
                self.adjust_view();
            }
        }
    }

    fn redraw(&mut self) -> Result<()> {
        // 0001:  00 31[32]20 00 00 11 00 10 10  .12                        ......
        //
        // &Offset: [          2 ]  &Selection: 0 - 0
        //
        // int  8:           32    int 32:          8242    float 32:          ...
        // int 16:         8242    int 64:          8242    float 64:          ...
        //
        // [ Little &Endian ]  [ Uns&igned ]  [ &Help ]  [ &Quit ]              0%

        let window = self.curses.window_mut();
        let bytes_per_row = self.bytes_per_row;
        
        if bytes_per_row == 0 || self.win_size.rows < 8 {
            window.move_to((0, 0))?;
            // ignore over long line errors:
            let _ = window.put_str("Window\ntoo\nsmall!");
            return Ok(());
        }

        let mem = self.mmap.mem();
        let size = mem.len();

        if !self.view_mask_valid {
            // TODO: invalidate view_mask in viewer cases
            self.view_mask.resize(self.view_size, 0);
            for item in self.view_mask.iter_mut() {
                *item = 0;
            }

            let mask_selection_start_offset = if self.view_offset < self.selection_start {
                self.selection_start - self.view_offset
            } else {
                0
            };
            if mask_selection_start_offset < self.view_size && self.selection_end > self.view_offset {
                let mask_selection_end_offset = min(self.selection_end - self.view_offset, self.view_size);
                if mask_selection_end_offset > mask_selection_start_offset {
                    for item in &mut self.view_mask[mask_selection_start_offset..mask_selection_end_offset] {
                        *item = MASK_SELECTED;
                    }
                    self.view_mask[mask_selection_end_offset - 1] = MASK_SELECTED | MASK_SELECTED_END;
                }
            }

            set_search_mask(&mut self.view_mask, self.view_offset, &mem, &mem[self.selection_start..self.selection_end], MASK_HIGHLIGHT, MASK_HIGHLIGHT_END);
            set_search_mask(&mut self.view_mask, self.view_offset, &mem, &self.search_data, MASK_SEARCH, MASK_SEARCH_END);

            self.view_mask_valid = true;
        }

        let view_end_offset = min(self.view_offset + self.view_size, size);

        let buf = &mut self.buf;
        let mut line = 0;
        for row_offset in (self.view_offset..view_end_offset).step_by(self.bytes_per_row) {
            buf.clear();
            write!(buf, "{:01$X}:", row_offset, self.offset_hex_len)?;
            
            window.move_to((line, 0))?;
            window.turn_on_attributes(ColorPair(PAIR_OFFSETS))?;
            window.put_str(&buf)?;
            window.turn_off_attributes(ColorPair(PAIR_OFFSETS))?;

            window.put_str("  ")?;

            let overflow_offset = row_offset + self.bytes_per_row;
            let end_byte_offset = min(overflow_offset, size);

            let mut byte_offset = row_offset;
            if byte_offset < end_byte_offset {
                loop {
                    let mask_index = byte_offset - self.view_offset;
                    let mask = self.view_mask[mask_index];

                    let byte = mem[byte_offset];
                    buf.clear();
                    write!(buf, "{:02X}", byte)?;

                    if byte_offset == self.cursor {
                        let attrs = if mask & MASK_SELECTED != 0 {
                            ColorPair(PAIR_SELECTED_CURSOR)
                        } else if mask & MASK_SEARCH != 0 {
                            ColorPair(PAIR_SEARCH_MATCH_CURSOR)
                        } else {
                            ColorPair(PAIR_CURSOR)
                        };

                        window.turn_on_attributes(attrs)?;
                        window.put_str(&buf)?;

                        let attrs = if mask & MASK_SELECTED != 0 {
                            ColorPair(PAIR_SELECTION)
                        } else if mask & MASK_SEARCH != 0 {
                            ColorPair(PAIR_SEARCH_MATCH)
                        } else if mask & MASK_HIGHLIGHT != 0 {
                            ColorPair(PAIR_SELECTION_MATCH)
                        } else {
                            ColorPair(PAIR_NORMAL)
                        };
                        window.turn_on_attributes(attrs)?;
                    } else {
                        let attrs = if mask & MASK_SELECTED != 0 {
                            ColorPair(PAIR_SELECTION)
                        } else if mask & MASK_SEARCH != 0 {
                            ColorPair(PAIR_SEARCH_MATCH)
                        } else if mask & MASK_HIGHLIGHT != 0 {
                            ColorPair(PAIR_SELECTION_MATCH)
                        } else {
                            ColorPair(PAIR_NORMAL)
                        };

                        window.turn_on_attributes(attrs)?;
                        window.put_str(&buf)?;
                    }

                    byte_offset += 1;
                    if byte_offset == end_byte_offset {
                        window.turn_on_attributes(ColorPair(PAIR_NORMAL))?;
                        window.put_char(' ')?;
                        break;
                    }

                    let attrs = if mask & (MASK_SELECTED | MASK_SELECTED_END) == MASK_SELECTED {
                        ColorPair(PAIR_SELECTION)
                    } else if mask & (MASK_SEARCH | MASK_SEARCH_END) == MASK_SEARCH {
                        ColorPair(PAIR_SEARCH_MATCH)
                    } else if mask & (MASK_HIGHLIGHT | MASK_HIGHLIGHT_END) == MASK_HIGHLIGHT {
                        ColorPair(PAIR_SELECTION_MATCH)
                    } else {
                        ColorPair(PAIR_NORMAL)
                    };

                    window.turn_on_attributes(attrs)?;
                    window.put_char(' ')?;
                }
            }

            for _ in end_byte_offset..overflow_offset {
                window.put_str("   ")?;
            }

            window.put_char(' ')?;

            for byte_offset in row_offset..end_byte_offset {
                let mask_index = byte_offset - self.view_offset;
                let mask = self.view_mask[mask_index];

                let byte = mem[byte_offset];
                let ch = if byte >= 0x20 && byte <= 0x7e {
                    byte as char
                } else {
                    '.'
                };

                let attrs = if byte_offset == self.cursor {
                    if mask & MASK_SELECTED != 0 {
                        ColorPair(PAIR_SELECTED_CURSOR)
                    } else if mask & MASK_SEARCH != 0 {
                        ColorPair(PAIR_SEARCH_MATCH_CURSOR)
                    } else {
                        ColorPair(PAIR_CURSOR)
                    }
                } else {
                    if mask & MASK_SELECTED != 0 {
                        ColorPair(PAIR_SELECTION)
                    } else if mask & MASK_SEARCH != 0 {
                        ColorPair(PAIR_SEARCH_MATCH)
                    } else if mask & MASK_HIGHLIGHT != 0 {
                        ColorPair(PAIR_SELECTION_MATCH)
                    } else {
                        ColorPair(PAIR_NORMAL)
                    }
                };

                window.turn_on_attributes(attrs)?;
                window.put_char(ch)?;
            }

            window.turn_on_attributes(ColorPair(PAIR_NORMAL))?;

            for _ in end_byte_offset..overflow_offset {
                window.put_char(' ')?;
            }

            line += 1;
        }

        let rows = self.win_size.rows;
        window.move_to((rows - 6, 0))?;

        buf.clear();
        write!(buf,
            " &Offset: [ {:>14} ]  &Selection: {} - {}",
            self.cursor, self.selection_start, self.selection_end)?;
        if self.selecting {
            buf.push_str(" selecting");
        }
        // 2 & marks
        while buf.len() < self.win_size.columns as usize + 2 {
            buf.push(' ');
        }
        let _ = put_label(window, &buf[..min(self.win_size.columns as usize, buf.len())]);

        if self.offset_input.has_focus() {
            self.offset_input.redraw(window, (rows - 6, 10))?;
        }

        window.move_to((self.win_size.rows - 4, 0))?;

        buf.clear();
        if self.signed {
            if let Some(num) = get_i8(mem, self.cursor) {
                write!(buf, " int  8: {:>6}  ", num)?;
            } else {
                buf.push_str(" int  8:         ");
            }

            if let Some(num) = get_i32(mem, self.cursor, self.endian) {
                write!(buf, "int 32: {:>20}  ", num)?;
            } else {
                buf.push_str("int 32:                       ");
            }
        } else {
            if let Some(num) = get_u8(mem, self.cursor) {
                write!(buf, " int  8: {:>6}  ", num)?;
            } else {
                buf.push_str(" int  8:         ");
            }

            if let Some(num) = get_u32(mem, self.cursor, self.endian) {
                write!(buf, "int 32: {:>20}  ", num)?;
            } else {
                buf.push_str("int 32:                       ");
            }
        }

        if let Some(num) = get_f32(mem, self.cursor, self.endian) {
            write!(buf, "float 32: {:>20.6e}  ", num)?;
        } else {
            buf.push_str("float 32:                              ");
        }

        window.put_str(&buf[..min(self.win_size.columns as usize, buf.len())])?;

        window.move_to((self.win_size.rows - 3, 0))?;

        buf.clear();
        if self.signed {
            if let Some(num) = get_i16(mem, self.cursor, self.endian) {
                write!(buf, " int 16: {:>6}  ", num)?;
            } else {
                buf.push_str(" int 16:         ");
            }

            if let Some(num) = get_i64(mem, self.cursor, self.endian) {
                write!(buf, "int 64: {:>20}  ", num)?;
            } else {
                buf.push_str("int 64:                       ");
            }
        } else {
            if let Some(num) = get_u16(mem, self.cursor, self.endian) {
                write!(buf, " int 16: {:>6}  ", num)?;
            } else {
                buf.push_str(" int 16:         ");
            }

            if let Some(num) = get_u64(mem, self.cursor, self.endian) {
                write!(buf, "int 64: {:>20}  ", num)?;
            } else {
                buf.push_str("int 64:                       ");
            }
        }

        if let Some(num) = get_f64(mem, self.cursor, self.endian) {
            write!(buf, "float 64: {:>20.6e}  ", num)?;
        } else {
            buf.push_str("float 64:                              ");
        }

        window.put_str(&buf[..min(self.win_size.columns as usize, buf.len())])?;

        if self.win_size.columns >= 5 {
            window.move_to((self.win_size.rows - 1, self.win_size.columns - 5))?;
            let pos = if size > 1 {
                100 * self.cursor / (size - 1)
            } else {
                100
            };
            window.put_str(format!("{:>3}%", pos))?;
        }

        window.move_to((self.win_size.rows - 1, 1))?;

        buf.clear();
        buf.push_str(match self.endian {
            Endian::Little => "[ Little &Endian ]",
            Endian::Big    => "[  Big &Endian   ]",
        });

        buf.push_str(
            if self.signed { "  [  S&igned  ]" }
            else           { "  [ Uns&igned ]" }
        );

        buf.push_str("  [ &Help]  [ &Quit ]");

        // ignore over long line errors here
        let _ = put_label(window, buf);

        window.move_to((self.win_size.rows - 7, 0))?;
        if let Some(error) = &self.error {
            let mut error = error.replace('\n', " ");
            error.insert_str(0, "Error: ");
            let count = error.chars().count();
            window.turn_on_attributes(ColorPair(PAIR_ERROR_MESSAGE))?;
            let _ = window.put_str(error);
            window.turn_off_attributes(ColorPair(PAIR_ERROR_MESSAGE))?;
            for _ in count..self.win_size.columns as usize {
                window.put_char(' ')?;
            }
        } else if self.file_input.has_focus() {
            window.put_str(FILE_INPUT_LABEL)?;
            self.file_input.redraw(window, (self.win_size.rows - 7, FILE_INPUT_LABEL.len() as i32))?;
        } else if self.search_widget.has_focus() {
            window.put_str(SEARCH_LABEL)?;
            self.search_widget.redraw(window, (self.win_size.rows - 7, SEARCH_LABEL.len() as i32))?;
        } else {
            for _ in 0..self.win_size.columns {
                window.put_char(' ')?;
            }
        }

        if self.help_shown {
            self.help_box.redraw(window)?;
        }

        Ok(())
    }

    fn resize(&mut self) -> Result<()> {
        let window = self.curses.window_mut();
        let win_size = window.size();

        let label_len = FILE_INPUT_LABEL.len() as i32;
        self.file_input.resize(&Dimension {
            columns: if win_size.columns > label_len { win_size.columns - label_len } else { 0 },
            rows: win_size.rows,
        })?;

        let label_len = SEARCH_LABEL.len() as i32;
        self.search_widget.resize(&Dimension {
            columns: if win_size.columns > label_len { win_size.columns - label_len } else { 0 },
            rows: win_size.rows,
        })?;

        if self.help_shown {
            self.help_box.resize(&win_size)?;
        }

        if win_size.rows != self.win_size.rows || win_size.columns != self.win_size.columns {
            window.clear()?;

            self.win_size = win_size;
            self.need_redraw = true;

            if self.win_size.rows < 7 || self.const_space + 3 > self.win_size.columns as usize {
                self.bytes_per_row = 0;
                self.view_size = 0;
            } else {
                let rest = self.win_size.columns as usize - self.const_space;
                self.bytes_per_row = (rest + 1) / 4;

                let view_rows = (self.win_size.rows - 7) as usize;
                self.view_size = self.bytes_per_row * view_rows;
            }

            self.adjust_view();
        }

        Ok(())
    }

    fn adjust_view(&mut self) {
        if self.bytes_per_row > 0 {
            if self.cursor >= self.view_offset + self.view_size {
                self.view_offset = self.cursor - self.cursor % self.bytes_per_row + self.bytes_per_row - self.view_size;
                self.need_redraw = true;
            } else if self.cursor < self.view_offset {
                self.view_offset = self.cursor - self.cursor % self.bytes_per_row;
                self.need_redraw = true;
            }
        }
        self.view_mask_valid = false;
    }

    fn handle(&mut self, input: Input) -> Result<bool> {
        match input {
            Input::KeyDown => {
                let cursor = self.cursor + self.bytes_per_row;
                if cursor < self.mmap.size() {
                    self.set_cursor(cursor);
                }
                self.error = None;
            }
            Input::KeyUp => {
                if self.cursor >= self.bytes_per_row {
                    self.set_cursor(self.cursor - self.bytes_per_row);
                }
                self.error = None;
            }
            Input::KeyLeft => {
                if self.cursor > 0 {
                    self.set_cursor(self.cursor - 1);
                }
                self.error = None;
            }
            Input::KeyRight => {
                self.set_cursor(self.cursor + 1);
                self.error = None;
            }
            Input::KeyHome => {
                if self.bytes_per_row > 0 {
                    let cursor = self.cursor - self.cursor % self.bytes_per_row;
                    self.set_cursor(cursor);
                }
                self.error = None;
            }
            Input::KeyEnd => {
                let size = self.mmap.size();
                if size > 0 && self.bytes_per_row > 0 {
                    let cursor = min(self.cursor + self.bytes_per_row - self.cursor % self.bytes_per_row , size) - 1;
                    self.set_cursor(cursor);
                }
                self.error = None;
            }
            Input::Character(CANCEL) => { // Ctrl+Home
                if self.cursor != 0 {
                    self.set_cursor(0);
                }
                self.error = None;
            }
            Input::Character(DEVICE_CONTROL3) => { // Ctrl+End
                let size = self.mmap.size();
                if size > 0 {
                    self.set_cursor(size - 1);
                }
                self.error = None;
            }
            Input::KeyPPage => {
                if self.view_offset > 0 {
                    if self.view_offset >= self.view_size {
                        let cursor = self.cursor - self.view_size;
                        self.view_offset -= self.view_size;
                        self.set_cursor(cursor);
                    } else {
                        let cursor = self.cursor - self.view_offset;
                        self.view_offset = 0;
                        self.set_cursor(cursor);
                    }
                    self.need_redraw = true;
                }
                self.error = None;
            }
            Input::KeyNPage => {
                let size = self.mmap.size();
                if self.view_offset < size && self.view_size <= size {
                    if self.view_offset + self.view_size < size {
                        self.view_offset += self.view_size;
                        if size > 0 {
                            let cursor = min(self.cursor + self.view_size, size - 1);
                            self.set_cursor(cursor);
                        }
                    } else if self.bytes_per_row > 0 {
                        self.view_offset = (size + self.bytes_per_row - size % self.bytes_per_row) - self.view_size;
                        self.view_offset += self.view_offset % self.bytes_per_row;
                    }
                    self.need_redraw = true;
                }
                self.error = None;
            }
            Input::KeyResize => {
                self.resize()?;
            }
            Input::Character('e') => {
                self.set_endian(match self.endian {
                    Endian::Big    => Endian::Little,
                    Endian::Little => Endian::Big,
                });
                self.error = None;
            }
            Input::Character('i') => {
                self.set_signed(!self.signed);
                self.error = None;
            }
            Input::Character('s') => {
                if self.selecting {
                    self.selecting = false;
                } else {
                    self.selection_start = self.cursor;
                    self.selection_end   = self.cursor + 1;
                    self.selecting       = true;
                    self.view_mask_valid = false;
                }
                self.need_redraw = true;
                self.error = None;
            }
            Input::Character('S') => {
                self.selecting       = false;
                self.selection_start = 0;
                self.selection_end   = 0;
                self.need_redraw     = true;
                self.view_mask_valid = false;
                self.error = None;
            }
            Input::Character('o') => {
                self.file_input.blur()?;
                self.search_widget.blur()?;
                self.offset_input.set_value(self.cursor)?;
                self.offset_input.focus()?;
                self.need_redraw = true;
                self.selecting = false;
                self.error = None;
            }
            Input::Character('f') | Input::KeyF3 => {
                self.error = None;
                self.selecting = false;
                self.file_input.blur()?;
                self.offset_input.blur()?;
                if self.selection_end > self.selection_start {
                    self.search_data.clear();
                    self.search_data.extend(&self.mmap.mem()[self.selection_start..self.selection_end]);
                    if self.search_data.iter().all(|byte| *byte >= 0x20 && *byte <= 0x7e) {
                        self.search_widget.set_mode_and_value(SearchMode::String, &self.search_data)?;
                    } else {
                        self.search_widget.set_mode_and_value(SearchMode::Binary, &self.search_data)?;
                    }
                }
                self.search_widget.focus()?;
                self.need_redraw = true;
            }
            Input::Character('F') => {
                self.error = None;
                self.search_widget.blur()?;
                self.search_data.clear();
                self.need_redraw = true;
            }
            Input::Character('n') => {
                self.find_next();
            }
            Input::Character('p') => {
                self.find_previous();
            }
            Input::Character('w') => {
                if self.selection_start < self.selection_end {
                    self.error = None;
                    self.selecting = false;
                    self.search_widget.blur()?;
                    self.offset_input.blur()?;
                    self.file_input.set_value("")?;
                    self.file_input.focus()?;
                } else {
                    self.error = Some("Nothing selected".to_owned());
                }
                self.need_redraw = true;
            }
            Input::Character('h') | Input::KeyF1 => {
                self.selecting = false;
                self.help_box.resize(&self.win_size)?;
                self.help_shown  = true;
                self.need_redraw = true;
            }
            Input::Character('q') | Input::Character(END_OF_TRANSMISSION) => {
                return Ok(false)
            }
            _input => {}
        }

        Ok(true)
    }

    pub fn run(&mut self) -> Result<()> {
        self.resize()?;

        loop {
            if self.need_redraw {
                self.redraw()?;
                self.need_redraw = false;
            }

            if let Some(input) = self.curses.window_mut().read_char() {
                if self.help_shown {
                    match input {
                        Input::Character('h') | Input::KeyF1 => {
                            self.help_shown  = false;
                            self.need_redraw = true;
                        }
                        _input => {
                            match self.help_box.handle(input)? {
                                TextBoxResult::Redraw => {
                                    self.need_redraw = true;
                                }
                                TextBoxResult::Ignore => {}
                                TextBoxResult::Quit => {
                                    self.help_shown = false;
                                    self.need_redraw = true;
                                }
                                TextBoxResult::PropagateEvent => {
                                    if !self.handle(input)? {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else if self.error.is_some() {
                    match input {
                        Input::Character(ch) if ch != 'h' => {
                            self.error = None;
                            self.need_redraw = true;
                        }
                        _ => {
                            if !self.handle(input)? {
                                break;
                            }
                        }
                    }
                } else if self.file_input.has_focus() {
                    match self.file_input.handle(input)? {
                        WidgetResult::PropagateEvent => {
                            if !self.handle(input)? {
                                break;
                            }
                        }
                        WidgetResult::Redraw => {
                            self.need_redraw = true;
                        }
                        WidgetResult::Value(path) => {
                            self.need_redraw = true;
                            match File::create(&path) {
                                Ok(mut file) => {
                                    use std::io::Write;

                                    let data = &self.mmap.mem()[self.selection_start..self.selection_end];

                                    if let Err(error) = file.write_all(data) {
                                        self.error = Some(format!("{}: {:?}", error, path));
                                    }
                                }
                                Err(error) => {
                                    self.error = Some(format!("{}: {:?}", error, path));
                                }
                            }
                        }
                        WidgetResult::Ignore => {}
                    }
                } else if self.search_widget.has_focus() {
                    match self.search_widget.handle(input)? {
                        WidgetResult::PropagateEvent => {
                            if !self.handle(input)? {
                                break;
                            }
                        }
                        WidgetResult::Redraw => {
                            self.need_redraw = true;
                        }
                        WidgetResult::Value(bytes) => {
                            self.search_data = bytes;
                            self.view_mask_valid = false;
                            self.need_redraw = true;
                            self.find_next();
                        }
                        WidgetResult::Ignore => {}
                    }
                } else if self.offset_input.has_focus() {
                    match self.offset_input.handle(input)? {
                        WidgetResult::PropagateEvent => {
                            if !self.handle(input)? {
                                break;
                            }
                        }
                        WidgetResult::Redraw => {
                            self.need_redraw = true;
                        }
                        WidgetResult::Value(value) => {
                            self.set_cursor(value);
                        }
                        WidgetResult::Ignore => {}
                    }
                } else {
                    if !self.handle(input)? {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    fn find_next(&mut self) -> bool {
        let search_data = &self.search_data[..];
        let search_size = search_data.len();
        if search_size > 0 {
            let size = self.mmap.size();
            self.need_redraw = true;
            if search_size <= size {
                let mem = self.mmap.mem();
                let start_offset = self.cursor + 1;
                let end_offset = size - search_size + 1;
                for offset in start_offset..end_offset {
                    if &mem[offset..offset + search_size] == search_data {
                        self.error = None;
                        self.set_cursor(offset);
                        return true;
                    }
                }
            }
            self.error = Some("Pattern not found searching forwards".to_owned());
        }

        false
    }

    fn find_previous(&mut self) -> bool {
        let search_data = &self.search_data[..];
        let search_size = search_data.len();
        if search_size > 0 {
            let size = self.mmap.size();
            self.need_redraw = true;
            if self.cursor > 0 {
                let mem = self.mmap.mem();
                let start_offset = min(self.cursor - 1, size - search_size);
                let mut offset = start_offset;
                loop {
                    if &mem[offset..offset + search_size] == search_data {
                        self.error = None;
                        self.set_cursor(offset);
                        return true;
                    }
                    if offset == 0 {
                        break;
                    }
                    offset -= 1;
                }
            }
            self.error = Some("Pattern not found searching backwards".to_owned());
        }

        false
    }
}
