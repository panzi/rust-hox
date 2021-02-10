use std::fs::File;
use std::fmt::Write;
use std::cmp::{min, max};

use pancurses_result::{
    initscr, Input, Dimension, Curses, Window,
    Attribute, ColorPair, CursorVisibility,
    COLOR_BLACK, COLOR_BLUE, COLOR_CYAN, COLOR_GREEN,
    COLOR_MAGENTA, COLOR_RED, COLOR_WHITE, COLOR_YELLOW,
};

use crate::mmap::MMap;
use crate::result::{Result, Error};
use crate::number_input::{NumberInput, NumberResult};
use crate::consts::*;
use crate::input_widget::InputWidget;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Endian {
    Big,
    Little,
}

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
    matchmap: Vec<bool>,
    offset_input: NumberInput<usize>,
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

        colors.set_color_pair(PAIR_NORMAL             as i16, COLOR_WHITE, COLOR_BLACK)?;
        colors.set_color_pair(PAIR_OFFSETS            as i16, 130,         COLOR_BLACK)?;
        colors.set_color_pair(PAIR_NON_ASCII          as i16, 239,         COLOR_BLACK)?;
        colors.set_color_pair(PAIR_CURSOR             as i16, COLOR_WHITE, COLOR_RED)?;
        colors.set_color_pair(PAIR_SELECTION          as i16, COLOR_BLACK, COLOR_BLUE)?;
        colors.set_color_pair(PAIR_SELECTED_CURSOR    as i16, COLOR_WHITE, 128)?;
        colors.set_color_pair(PAIR_INPUT              as i16, COLOR_BLACK, COLOR_WHITE)?;
        colors.set_color_pair(PAIR_INPUT_ERROR        as i16, COLOR_WHITE, COLOR_RED)?;
        colors.set_color_pair(PAIR_MATCH              as i16, COLOR_WHITE, 236)?;
        
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
            matchmap: Vec::new(),
            offset_input: NumberInput::<usize>::new(14),
        })
    }

    pub fn signed(&self) -> bool {
        self.signed
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }

    pub fn cursor(&self) -> usize {
        self.cursor
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
        // 0001:  00 31[32]20 00 00 11 00 10 10  .12 ......
        //
        // offset: [          2 ]  selection: 0 - 0
        //
        // int  8:           32    int 32:          8242    float 32:          ...
        // int 16:         8242    int 64:          8242    float 64:          ...
        //
        // [ little &endian ]  [ uns&igned ]  [ &quit ]                                0%

        let window = self.curses.window_mut();
        let bytes_per_row = self.bytes_per_row;
        
        if bytes_per_row == 0 || self.win_size.rows < 8 {
            window.move_to((0, 0))?;
            // ignore over long line errors:
            let _ = window.put_str("Window too small!");
        } else {
            let mem = self.mmap.mem();
            let size = mem.len();

            // TODO: do this on selection and viewport change, not on render?
            for item in self.matchmap.iter_mut() {
                *item = false;
            }
            if self.selection_start < self.selection_end {
                let needle = &mem[self.selection_start..self.selection_end];
                let needle_len = needle.len();
                let start_offset = if self.view_offset > (needle_len - 1) {
                    self.view_offset + 1 - needle_len
                } else {
                    0
                };
                let end_offset = self.view_offset + self.view_size + needle_len - 1;
                let end_offset = if end_offset <= size {
                    end_offset
                } else {
                    size + 1 - needle_len
                };

                let view_end_offset = min(self.view_offset + self.view_size, size);
                for offset in start_offset..end_offset {
                    if &mem[offset..offset + needle_len] == needle {
                        for match_offset in max(self.view_offset, offset)..min(view_end_offset, offset + needle_len) {
                            self.matchmap[match_offset - self.view_offset] = true;
                        }
                    }
                }
            }

            let view_end_offset = min(self.view_offset + self.view_size, size);

            // TODO: auto search selection
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

                for byte_offset in row_offset..end_byte_offset {
                    let match_index = byte_offset - self.view_offset;
                    let is_match = self.matchmap[match_index];
                    let is_selected = byte_offset >= self.selection_start && byte_offset < self.selection_end;

                    let byte = mem[byte_offset];
                    buf.clear();
                    write!(buf, "{:02X}", byte)?;

                    if byte_offset == self.cursor {
                        let attrs = if is_selected {
                            ColorPair(PAIR_SELECTED_CURSOR)
                        } else {
                            ColorPair(PAIR_CURSOR)
                        };
                        window.turn_on_attributes(attrs)?;
                        window.put_str(&buf)?;
                        window.turn_off_attributes(attrs)?;

                        if is_selected {
                            if byte_offset + 1 < self.selection_end {
                                window.turn_on_attributes(ColorPair(PAIR_SELECTION))?;
                            } else if byte_offset + 1 < end_byte_offset && self.matchmap[match_index + 1] {
                                window.turn_on_attributes(ColorPair(PAIR_MATCH))?;
                            }
                        } else if is_match {
                            if byte_offset + 1 < end_byte_offset && self.matchmap[match_index + 1] {
                                window.turn_on_attributes(ColorPair(PAIR_MATCH))?;
                            }
                        }
                    } else {
                        if is_selected {
                            if byte_offset == row_offset || byte_offset == self.selection_start {
                                window.turn_on_attributes(ColorPair(PAIR_SELECTION))?;
                            }
                        } else if is_match {
                            if byte_offset == row_offset || match_index == 0 || !self.matchmap[match_index - 1] {
                                window.turn_on_attributes(ColorPair(PAIR_MATCH))?;
                            }
                        }

                        window.put_str(&buf)?;

                        if byte_offset + 1 >= self.selection_end || byte_offset + 1 == end_byte_offset {
                            if byte_offset + 1 < end_byte_offset && self.matchmap[match_index + 1] {
                                if is_match {
                                    window.turn_on_attributes(ColorPair(PAIR_MATCH))?;
                                }
                            } else if is_selected {
                                window.turn_off_attributes(ColorPair(PAIR_SELECTION))?;
                            } else if is_match {
                                window.turn_off_attributes(ColorPair(PAIR_MATCH))?;
                            }
                        } else if byte_offset + 1 >= end_byte_offset || !self.matchmap[match_index + 1] {
                            if is_match {
                                window.turn_off_attributes(ColorPair(PAIR_MATCH))?;
                            }
                        }
                    }
                    window.put_char(' ')?;
                }

                for _ in end_byte_offset..overflow_offset {
                    window.put_str("   ")?;
                }

                window.put_char(' ')?;
                for byte_offset in row_offset..end_byte_offset {
                    let match_index = byte_offset - self.view_offset;
                    let is_match = self.matchmap[match_index];
                    let is_selected = byte_offset >= self.selection_start && byte_offset < self.selection_end;
                    let byte = mem[byte_offset];

                    buf.clear();
                    let is_ascii = byte >= 0x20 && byte <= 0x7e;
                    if is_ascii {
                        buf.push(byte as char);
                    } else {
                        buf.push('.');
                    }

                    let attrs = if byte_offset == self.cursor {
                        if is_selected {
                            ColorPair(PAIR_SELECTED_CURSOR)
                        } else {
                            ColorPair(PAIR_CURSOR)
                        }
                    } else {
                        if is_selected {
                            ColorPair(PAIR_SELECTION)
                        } else if is_match {
                            ColorPair(PAIR_MATCH)
                        } else if !is_ascii {
                            ColorPair(PAIR_NON_ASCII)
                        } else {
                            ColorPair(PAIR_NORMAL)
                        }
                    };

                    window.turn_on_attributes(attrs)?;
                    window.put_str(&buf)?;
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
                self.offset_input.redraw(window, (rows - 6, 11))?;
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

            buf.push_str("  [ &Quit ]");

            // ignore over long line errors here
            let _ = put_label(window, buf);
        }

        Ok(())
    }

    fn resize(&mut self) -> Result<()> {
        let window = self.curses.window_mut();
        let win_size = window.size();

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

            self.matchmap.resize(self.view_size, false);

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
    }

    fn handle(&mut self, input: Input) -> Result<bool> {
        match input {
            Input::KeyDown => {
                let cursor = self.cursor + self.bytes_per_row;
                if cursor < self.mmap.size() {
                    self.set_cursor(cursor);
                }
            },
            Input::KeyUp => {
                if self.cursor >= self.bytes_per_row {
                    self.set_cursor(self.cursor - self.bytes_per_row);
                }
            },
            Input::KeyLeft => {
                if self.cursor > 0 {
                    self.set_cursor(self.cursor - 1);
                }
            },
            Input::KeyRight => {
                self.set_cursor(self.cursor + 1);
            },
            Input::KeyHome => {
                if self.bytes_per_row > 0 {
                    let cursor = self.cursor - self.cursor % self.bytes_per_row;
                    self.set_cursor(cursor);
                }
            },
            Input::KeyEnd => {
                let size = self.mmap.size();
                if size > 0 && self.bytes_per_row > 0 {
                    let cursor = min(self.cursor + self.bytes_per_row - self.cursor % self.bytes_per_row , size) - 1;
                    self.set_cursor(cursor);
                }
            },
            Input::Character('\u{18}') => { // Ctrl+Home
                if self.cursor != 0 {
                    self.set_cursor(0);
                }
            },
            Input::Character('\u{13}') => { // Ctrl+End
                let size = self.mmap.size();
                if size > 0 {
                    self.set_cursor(size - 1);
                }
            },
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
            },
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
            },
            Input::KeyResize => {
                self.resize()?;
            },
            Input::Character('e') => {
                self.set_endian(match self.endian {
                    Endian::Big    => Endian::Little,
                    Endian::Little => Endian::Big,
                });
            },
            Input::Character('i') => {
                self.set_signed(!self.signed);
            },
            Input::Character('s') => {
                if self.selecting {
                    self.selecting = false;
                } else {
                    self.selection_start = self.cursor;
                    self.selection_end   = self.cursor + 1;
                    self.selecting       = true;
                }
                self.need_redraw = true;
            },
            Input::Character('S') => {
                self.selecting = false;
                self.selection_start = 0;
                self.selection_end   = 0;
                self.need_redraw = true;
            },
            Input::Character('o') => {
                self.offset_input.focus(self.cursor)?;
                self.need_redraw = true;
            },
            Input::Character('f') => {
                // TODO: search ASCII
            },
            Input::Character('w') => {
                // TODO: write selection to file
            },
            Input::Character('q') => return Ok(false),
            _input => {
                //self.curses.window_mut().put_str(format!("INPUT: {:?}\n", input))?;
            }
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
                if self.offset_input.has_focus() {
                    match self.offset_input.handle(input)? {
                        NumberResult::PropagateEvent => {
                            if !self.handle(input)? {
                                break;
                            }
                        },
                        NumberResult::Redraw => {
                            self.need_redraw = true;
                        },
                        NumberResult::SetValue(value) => {
                            self.set_cursor(value);
                        },
                        NumberResult::Ignore => {},
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
}
