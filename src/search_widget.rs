use std::cmp::min;
// use std::collections::vec_deque::VecDeque;

use pancurses_result::{Window, Point, Input, ColorPair, Dimension};

use crate::input_widget::{InputWidget, WidgetResult};
use crate::result::{Result, Error};
use crate::consts::*;
use crate::hox::Endian;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IntSize {
    I8,
    I16,
    I32,
    I64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sign {
    Signed,
    Unsigned,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SearchMode {
    String,
    Binary,
    Integer(IntSize, Sign, Endian),
}

impl SearchMode {
    pub fn parse(&self, input: &[char]) -> Result<Vec<u8>> {
        let mut data = Vec::new();
        match self {
            SearchMode::String => {
                let mut buf = [0; 4];
                for ch in input {
                    let count = ch.encode_utf8(&mut buf).len();
                    data.extend(&buf[..count]);
                }
            }
            SearchMode::Binary => {
                let mut iter = input.iter();
                loop {
                    if let Some(ch) = iter.next() {
                        let ch = *ch;
                        let mut byte = if ch >= 'a' && ch <= 'f' {
                            ch as u8 - 'a' as u8 + 10
                        } else if ch >= 'A' && ch <= 'F' {
                            ch as u8 - 'A' as u8 + 10
                        } else if ch >= '0' && ch <= '9' {
                            ch as u8 - '0' as u8
                        } else {
                            return Err(Error::message(format!("illegal byte in hex string: {:?}", input.iter().collect::<String>())));
                        };
                        if let Some(ch) = iter.next() {
                            byte <<= 4;
                            let ch = *ch;
                            byte |= if ch >= 'a' && ch <= 'f' {
                                ch as u8 - 'a' as u8 + 10
                            } else if ch >= 'A' && ch <= 'F' {
                                ch as u8 - 'A' as u8 + 10
                            } else if ch >= '0' && ch <= '9' {
                                ch as u8 - '0' as u8
                            } else {
                                return Err(Error::message(format!("illegal byte in hex string: {:?}", input.iter().collect::<String>())));
                            };
                            data.push(byte);
                            match iter.next() {
                                Some(' ') => {},
                                Some(_) => {
                                    return Err(Error::message(format!("illegal byte in hex string: {:?}", input.iter().collect::<String>())));
                                }
                                None => break,
                            }
                        } else {
                            data.push(byte);
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }

            SearchMode::Integer(IntSize::I8, _, _) if input.is_empty() => {
                data.push(0);
            }
            SearchMode::Integer(IntSize::I8, Sign::Unsigned, _) => {
                let value = input.iter().collect::<String>().parse::<u8>()?;
                data.push(value);
            }
            SearchMode::Integer(IntSize::I8, Sign::Signed, _) => {
                let value = input.iter().collect::<String>().parse::<i8>()?;
                data.push(value as u8);
            }

            SearchMode::Integer(IntSize::I16, _, _) if input.is_empty() => {
                data.extend(&[0, 0]);
            }
            SearchMode::Integer(IntSize::I16, Sign::Unsigned, Endian::Little) => {
                let value = input.iter().collect::<String>().parse::<u16>()?;
                data.extend(&value.to_le_bytes());
            }
            SearchMode::Integer(IntSize::I16, Sign::Unsigned, Endian::Big) => {
                let value = input.iter().collect::<String>().parse::<u16>()?;
                data.extend(&value.to_be_bytes());
            }
            SearchMode::Integer(IntSize::I16, Sign::Signed, Endian::Little) => {
                let value = input.iter().collect::<String>().parse::<i16>()?;
                data.extend(&value.to_le_bytes());
            }
            SearchMode::Integer(IntSize::I16, Sign::Signed, Endian::Big) => {
                let value = input.iter().collect::<String>().parse::<i16>()?;
                data.extend(&value.to_be_bytes());
            }

            SearchMode::Integer(IntSize::I32, _, _) if input.is_empty() => {
                data.extend(&[0, 0, 0, 0]);
            }
            SearchMode::Integer(IntSize::I32, Sign::Unsigned, Endian::Little) => {
                let value = input.iter().collect::<String>().parse::<u32>()?;
                data.extend(&value.to_le_bytes());
            }
            SearchMode::Integer(IntSize::I32, Sign::Unsigned, Endian::Big) => {
                let value = input.iter().collect::<String>().parse::<u32>()?;
                data.extend(&value.to_be_bytes());
            }
            SearchMode::Integer(IntSize::I32, Sign::Signed, Endian::Little) => {
                let value = input.iter().collect::<String>().parse::<i32>()?;
                data.extend(&value.to_le_bytes());
            }
            SearchMode::Integer(IntSize::I32, Sign::Signed, Endian::Big) => {
                let value = input.iter().collect::<String>().parse::<i32>()?;
                data.extend(&value.to_be_bytes());
            }

            SearchMode::Integer(IntSize::I64, _, _) if input.is_empty() => {
                data.extend(&[0, 0, 0, 0, 0, 0, 0, 0]);
            }
            SearchMode::Integer(IntSize::I64, Sign::Unsigned, Endian::Little) => {
                let value = input.iter().collect::<String>().parse::<u64>()?;
                data.extend(&value.to_le_bytes());
            }
            SearchMode::Integer(IntSize::I64, Sign::Unsigned, Endian::Big) => {
                let value = input.iter().collect::<String>().parse::<u64>()?;
                data.extend(&value.to_be_bytes());
            }
            SearchMode::Integer(IntSize::I64, Sign::Signed, Endian::Little) => {
                let value = input.iter().collect::<String>().parse::<i64>()?;
                data.extend(&value.to_le_bytes());
            }
            SearchMode::Integer(IntSize::I64, Sign::Signed, Endian::Big) => {
                let value = input.iter().collect::<String>().parse::<i64>()?;
                data.extend(&value.to_be_bytes());
            }
        }

        Ok(data)
    }
}

pub struct SearchWidget {
    buf: Vec<char>,
    focused: bool,
    size:   usize,
    cursor: usize,
    view_offset: usize,
    // history: VecDeque<Vec<char>>,
    // future:  VecDeque<Vec<char>>,
    mode: SearchMode,
}

impl SearchWidget {
    pub fn new(size: usize) -> Self {
        Self {
            focused: false,
            buf: Vec::new(),
            size,
            cursor: 0,
            view_offset: 0,
            // history: VecDeque::new(),
            // future:  VecDeque::new(),
            mode: SearchMode::String,
        }
    }

    pub fn set_search_mode(&mut self, mode: SearchMode) {
        if self.mode != mode {
            self.mode = mode;
            self.buf.clear();
            self.cursor = 0;
            self.view_offset = 0;
        }
    }

    fn draw(&self, window: &mut Window, cursor: usize, buf: &[char]) -> Result<()> {
        if self.focused {
            if cursor > 0 {
                let before: String = (&buf[..cursor]).iter().collect();
                window.turn_on_attributes(ColorPair(PAIR_NORMAL))?;
                window.put_str(before)?;
                window.turn_off_attributes(ColorPair(PAIR_NORMAL))?;
            }

            if cursor < buf.len() {
                window.turn_on_attributes(ColorPair(PAIR_INVERTED))?;
                window.put_char(buf[cursor])?;
                window.turn_off_attributes(ColorPair(PAIR_INVERTED))?;
            }

            if cursor + 1 < buf.len() {
                let after: String = (&buf[cursor + 1..]).into_iter().collect();
                window.turn_on_attributes(ColorPair(PAIR_NORMAL))?;
                window.put_str(after)?;
                window.turn_off_attributes(ColorPair(PAIR_NORMAL))?;
            } else if cursor >= buf.len() {
                window.turn_on_attributes(ColorPair(PAIR_INVERTED))?;
                window.put_char(' ')?;
                window.turn_off_attributes(ColorPair(PAIR_INVERTED))?;
            }
        } else {
            let buf: String = buf.into_iter().collect();
            window.turn_on_attributes(ColorPair(PAIR_NORMAL))?;
            window.put_str(buf)?;
            window.turn_off_attributes(ColorPair(PAIR_NORMAL))?;
        }

        Ok(())
    }
}

impl InputWidget<&str, Vec<u8>> for SearchWidget {
    fn has_focus(&self) -> bool {
        self.focused
    }

    fn focus(&mut self, _initial_value: &str) -> Result<()> {
        self.focused = true;
        self.buf.clear();
        self.cursor = 0;
        self.view_offset = 0;

        Ok(())
    }

    fn blur(&mut self) -> Result<()> {
        self.focused = false;

        Ok(())
    }

    fn redraw<P>(&self, window: &mut Window, pos: P) -> Result<()>
    where P: Into<Point>, P: Copy {
        // &Find:               [ &Mode: Binary ]
        // &Find:               [ &Mode: String ]
        if self.size == 0 {
            return Ok(());
        }

        let buf = &self.buf;
        window.move_to(pos)?;

        let mut len = buf.len();

        let cursor_at_end = self.cursor == buf.len();
        if cursor_at_end {
            len += 1;
        }

        if len > self.size {
            if self.view_offset > buf.len() {
                self.draw(window, 0, &[])?;
            } else {
                let mut index = self.view_offset;

                if cursor_at_end {
                    index += 1;
                }

                let mut end_index = index + self.size;
                if cursor_at_end {
                    end_index -= 1;
                }
                let buf = &buf[index..min(end_index, buf.len())];

                let cursor = if self.cursor >= index {
                    self.cursor - index
                } else {
                    0
                };
                self.draw(window, cursor, buf)?;
            }
        } else {
            self.draw(window, self.cursor, &buf)?;
            for _ in 0..(self.size - len) {
                window.put_char(' ')?;
            }
        }

        Ok(())
    }

    fn handle(&mut self, input: Input) -> Result<WidgetResult<Vec<u8>>> {
        if !self.focused {
            return Ok(WidgetResult::PropagateEvent);
        }

        match input {
            Input::KeyHome => {
                self.cursor = 0;
                self.view_offset = 0;
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyEnd => {
                self.cursor = self.buf.len();
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyLeft => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    if self.mode == SearchMode::Binary {
                        if self.buf[self.cursor] == ' ' {
                            self.cursor -= 1;
                        }
                    }
                    if self.cursor < self.view_offset {
                        self.view_offset = self.cursor;
                    }
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::KeyRight => {
                if self.cursor < self.buf.len() {
                    self.cursor += 1;
                    if self.mode == SearchMode::Binary {
                        if self.cursor < self.buf.len() && self.buf[self.cursor] == ' ' {
                            self.cursor += 1;
                        }
                    }
                    if self.cursor > self.size {
                        self.view_offset = self.cursor - self.size;
                    }
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::Character(ESCAPE) | Input::Character(END_OF_TRANSMISSION) => {
                self.focused = false;
                return Ok(WidgetResult::Redraw);
            }
            Input::Character('\n') => {
                if self.buf.is_empty() {
                    return Ok(WidgetResult::Ignore);
                }
                //self.focused = false;
                /* history only works for correct mode. multiple histories?
                if self.future.len() > 0 {
                    let mut future = VecDeque::new();
                    std::mem::swap(&mut future, &mut self.future);
                    self.history.extend(future.into_iter());
                }
                if self.history.is_empty() {
                    self.history.push_back(self.buf.clone());
                } else if self.history[self.history.len() - 1] != self.buf {
                    if self.history.len() == 1024 {
                        self.history.pop_front();
                    }
                    self.history.push_back(self.buf.clone());
                }
                */
                return Ok(WidgetResult::Value(self.mode.parse(&self.buf)?))
            }
            Input::Character(mut ch) => {
                match self.mode {
                    SearchMode::Integer(_, _, _) => {
                        self.buf.insert(self.cursor, ch);
                        
                        if self.mode.parse(&self.buf).is_ok() {
                            self.cursor += 1;
                        } else {
                            self.buf.remove(self.cursor);
                        }
                    }
                    SearchMode::String => {
                        self.buf.insert(self.cursor, ch);
                        self.cursor += 1;
                    }
                    SearchMode::Binary => {
                        if ch == 'q' {
                            self.focused = false;
                            return Ok(WidgetResult::Redraw);
                        } else if ch >= 'a' && ch <= 'f' {
                            ch.make_ascii_uppercase();
                        } else if !((ch >= '0' && ch <= '9') || (ch >= 'A' && ch <= 'F')) {
                            return Ok(WidgetResult::Ignore);
                        }

                        if self.cursor >= self.buf.len() {
                            self.buf.push(ch);
                            self.buf.push('0');
                            self.cursor += 1;
                        } else {
                            match self.cursor % 3 {
                                0 => {
                                    self.buf[self.cursor] = ch;
                                    self.cursor += 1;
                                }
                                1 => {
                                    self.buf[self.cursor] = ch;
                                    self.cursor += 1;
                                    if self.cursor == self.buf.len() {
                                        self.buf.push(' ');
                                        self.cursor += 1;
                                    }
                                }
                                2 => { panic!("invalid state"); }
                                _ => { panic!("x % 3 not in {0, 1, 2}!"); }
                            }
                        }
                    }
                }
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyDC => {
                if self.cursor < self.buf.len() {
                    match self.mode {
                        SearchMode::String | SearchMode::Integer(_, _, _) => {
                            self.buf.remove(self.cursor);
                        }
                        SearchMode::Binary => {
                            match self.cursor % 3 {
                                0 => {}
                                1 => {
                                    self.cursor -= 1;
                                }
                                2 => { panic!("invalid state"); }
                                _ => { panic!("x % 3 not in {0, 1, 2}!"); }
                            }
                            self.buf.remove(self.cursor);
                            self.buf.remove(self.cursor);
                            if self.cursor < self.buf.len() {
                                self.buf.remove(self.cursor);
                            }
                        }
                    }
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::KeyBackspace => {
                if self.cursor > 0 {
                    match self.mode {
                        SearchMode::String | SearchMode::Integer(_, _, _) => {
                            self.buf.remove(self.cursor - 1);
                            self.cursor -= 1;
                        }
                        SearchMode::Binary => {
                            match self.cursor % 3 {
                                0 => {
                                    self.cursor -= 3;
                                }
                                1 => {
                                    self.cursor -= 1;
                                }
                                2 => {
                                    panic!("invalid state");
                                }
                                _ => panic!("x % 3 not in {0, 1, 2}!")
                            }
                            self.buf.remove(self.cursor);
                            self.buf.remove(self.cursor);
                            if self.cursor < self.buf.len() {
                                self.buf.remove(self.cursor);
                            }
                        }
                    }
                    if self.cursor < self.view_offset {
                        self.view_offset = self.cursor;
                    }
                    return Ok(WidgetResult::Redraw);
                }
            }
            /* history only works for correct mode
            Input::KeyUp => {
                if self.history.is_empty() {
                    return Ok(WidgetResult::Ignore);
                }
                self.future.push_front(self.buf.clone());
                self.buf = self.history.pop_back().unwrap();
                self.cursor = self.buf.len();
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }

                return Ok(WidgetResult::Redraw);
            }
            Input::KeyDown => {
                if self.future.is_empty() {
                    return Ok(WidgetResult::Ignore);
                }
                self.history.push_back(self.buf.clone());
                self.buf = self.future.pop_front().unwrap();
                self.cursor = self.buf.len();
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }

                return Ok(WidgetResult::Redraw);
            }
            */
            _input => {
                return Ok(WidgetResult::PropagateEvent);
            }
        }

        return Ok(WidgetResult::Ignore);
    }

    fn resize(&mut self, size: &Dimension) -> Result<()> {
        self.size = size.columns as usize;
        if self.cursor > self.size {
            self.view_offset = self.cursor - self.size;
        }
        Ok(())
    }
}

