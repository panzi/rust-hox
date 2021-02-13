use std::cmp::min;
use std::fmt::{Write, Display};
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

impl IntSize {
    pub fn next(&self) -> Self {
        match self {
            IntSize::I64 => IntSize::I32,
            IntSize::I32 => IntSize::I16,
            IntSize::I16 => IntSize::I8,
            IntSize::I8  => IntSize::I64,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Sign {
    Signed,
    Unsigned,
}

impl Sign {
    pub fn next(&self) -> Self {
        match self {
            Sign::Signed   => Sign::Unsigned,
            Sign::Unsigned => Sign::Signed,
        }
    }

    #[allow(unused)]
    pub fn is_signed(&self) -> bool {
        match self {
            Sign::Signed   => true,
            Sign::Unsigned => false,
        }
    }

    #[allow(unused)]
    pub fn is_unsigned(&self) -> bool {
        match self {
            Sign::Signed   => false,
            Sign::Unsigned => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SearchMode {
    String,
    Binary,
    Integer(IntSize, Sign, Endian),
}

impl Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::String => "String".fmt(f),
            SearchMode::Binary => "Binary".fmt(f),
            SearchMode::Integer(size, sign, endian) => {
                match sign {
                    Sign::Signed   => f.write_str("Int  ")?,
                    Sign::Unsigned => f.write_str("UInt ")?,
                }

                match size {
                    IntSize::I8  => f.write_str("8  ")?,
                    IntSize::I16 => f.write_str("16 ")?,
                    IntSize::I32 => f.write_str("32 ")?,
                    IntSize::I64 => f.write_str("64 ")?,
                }

                match endian {
                    Endian::Little => f.write_str("LE")?,
                    Endian::Big    => f.write_str("BE")?,
                }

                if let Some(width) = f.width() {
                    let mut count = 5 + 3 + 2;
                    while count < width {
                        write!(f, " ")?;
                        count += 1;
                    }
                }

                Ok(())
            }
        }
    }
}

impl SearchMode {
    #[allow(unused)]
    pub fn is_string(&self) -> bool {
        match self {
            SearchMode::String => true,
            _ => false,
        }
    }

    #[allow(unused)]
    pub fn is_binary(&self) -> bool {
        match self {
            SearchMode::Binary => true,
            _ => false,
        }
    }

    #[allow(unused)]
    pub fn is_integer(&self) -> bool {
        match self {
            SearchMode::Integer(_, _, _) => true,
            _ => false,
        }
    }

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
                            return Err(Error::message(format!(
                                "illegal byte in hex string: {:?}",
                                input.iter().collect::<String>())));
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
                                return Err(Error::message(format!(
                                    "illegal byte in hex string: {:?}",
                                    input.iter().collect::<String>())));
                            };
                            data.push(byte);
                            match iter.next() {
                                Some(' ') => {},
                                Some(_) => {
                                    return Err(Error::message(format!(
                                        "illegal byte in hex string: {:?}",
                                        input.iter().collect::<String>())));
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

    pub fn stringify(&self, input: &[u8]) -> Result<String> {
        match self {
            SearchMode::Binary => {
                let mut buf = String::new();
                for byte in input {
                    write!(buf, "{:02X} ", byte).unwrap();
                }
                Ok(buf)
            }
            SearchMode::String => {
                Ok(std::str::from_utf8(input)?.to_owned())
            }

            SearchMode::Integer(_, _, _) if input.is_empty() => {
                Ok("0".to_owned())
            }

            SearchMode::Integer(IntSize::I8, Sign::Unsigned, _) => {
                Ok(format!("{}", input[0]))
            }
            SearchMode::Integer(IntSize::I8, Sign::Signed, _) => {
                Ok(format!("{}", input[0] as i8))
            }

            SearchMode::Integer(IntSize::I16, Sign::Unsigned, Endian::Little) => {
                if input.len() < 2 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", u16::from_le_bytes([input[0], input[1]])))
            }
            SearchMode::Integer(IntSize::I16, Sign::Unsigned, Endian::Big) => {
                if input.len() < 2 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", u16::from_be_bytes([input[0], input[1]])))
            }
            SearchMode::Integer(IntSize::I16, Sign::Signed, Endian::Little) => {
                if input.len() < 2 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", i16::from_le_bytes([input[0], input[1]])))
            }
            SearchMode::Integer(IntSize::I16, Sign::Signed, Endian::Big) => {
                if input.len() < 2 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", i16::from_be_bytes([input[0], input[1]])))
            }

            SearchMode::Integer(IntSize::I32, Sign::Unsigned, Endian::Little) => {
                if input.len() < 4 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", u32::from_le_bytes([input[0], input[1], input[2], input[3]])))
            }
            SearchMode::Integer(IntSize::I32, Sign::Unsigned, Endian::Big) => {
                if input.len() < 4 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", u32::from_be_bytes([input[0], input[1], input[2], input[3]])))
            }
            SearchMode::Integer(IntSize::I32, Sign::Signed, Endian::Little) => {
                if input.len() < 4 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", i32::from_le_bytes([input[0], input[1], input[2], input[3]])))
            }
            SearchMode::Integer(IntSize::I32, Sign::Signed, Endian::Big) => {
                if input.len() < 4 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", i32::from_be_bytes([input[0], input[1], input[2], input[3]])))
            }

            SearchMode::Integer(IntSize::I64, Sign::Unsigned, Endian::Little) => {
                if input.len() < 8 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", u64::from_le_bytes([
                    input[0], input[1], input[2], input[3],
                    input[4], input[5], input[6], input[7]
                ])))
            }
            SearchMode::Integer(IntSize::I64, Sign::Unsigned, Endian::Big) => {
                if input.len() < 8 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", u64::from_be_bytes([
                    input[0], input[1], input[2], input[3],
                    input[4], input[5], input[6], input[7]
                ])))
            }
            SearchMode::Integer(IntSize::I64, Sign::Signed, Endian::Little) => {
                if input.len() < 8 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", i64::from_le_bytes([
                    input[0], input[1], input[2], input[3],
                    input[4], input[5], input[6], input[7]
                ])))
            }
            SearchMode::Integer(IntSize::I64, Sign::Signed, Endian::Big) => {
                if input.len() < 8 {
                    return Err(Error::message("not enough bytes"));
                }
                Ok(format!("{}", i64::from_be_bytes([
                    input[0], input[1], input[2], input[3],
                    input[4], input[5], input[6], input[7]
                ])))
            }
        }
    }

    pub fn next_major(&self) -> Self {
        match self {
            SearchMode::String => SearchMode::Binary,
            SearchMode::Binary => SearchMode::Integer(IntSize::I64, Sign::Signed, Endian::Little),
            SearchMode::Integer(_, _, _) => SearchMode::String,
        }
    }

    pub fn next_size(&self) -> Self {
        match self {
            SearchMode::Integer(size, sign, endian) => {
                SearchMode::Integer(size.next(), *sign, *endian)
            },
            other => *other
        }
    }

    pub fn next_sign(&self) -> Self {
        match self {
            SearchMode::Integer(size, sign, endian) => {
                SearchMode::Integer(*size, sign.next(), *endian)
            },
            other => *other
        }
    }

    pub fn next_endian(&self) -> Self {
        match self {
            SearchMode::Integer(size, sign, Endian::Little) => {
                SearchMode::Integer(*size, *sign, Endian::Big)
            },
            SearchMode::Integer(size, sign, Endian::Big) => {
                SearchMode::Integer(*size, *sign, Endian::Little)
            },
            other => *other
        }
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
            match mode {
                SearchMode::String => { /* keep */ },
                SearchMode::Binary => {
                    match self.mode {
                        SearchMode::String => {
                            if let Ok(buf) = mode.stringify(self.buf.iter().collect::<String>().as_bytes()) {
                                self.buf = buf.chars().collect();
                            } else {
                                self.buf.clear();
                            }
                        },
                        SearchMode::Binary => { /* keep */ }
                        SearchMode::Integer(_, _, _) => {
                            if let Ok(bytes) = self.mode.parse(&self.buf) {
                                if let Ok(buf) = mode.stringify(&bytes) {
                                    self.buf = buf.chars().collect();
                                } else {
                                    self.buf.clear();
                                }
                            } else {
                                self.buf.clear();
                            }
                        }
                    }
                }
                SearchMode::Integer(to_size, to_sign, _) => {
                    match self.mode {
                        SearchMode::Binary => {
                            if let Ok(bytes) = self.mode.parse(&self.buf) {
                                if let Ok(buf) = mode.stringify(&bytes) {
                                    self.buf = buf.chars().collect();
                                } else {
                                    self.buf.clear();
                                }
                            } else {
                                self.buf.clear();
                            }
                        }
                        SearchMode::String => {
                            if to_sign.is_signed() {
                                if let Ok(num) = self.buf.iter().collect::<String>().parse::<i64>() {
                                    self.buf = format!("{}", num).chars().collect();
                                } else {
                                    self.buf.clear();
                                }
                            } else if let Ok(num) = self.buf.iter().collect::<String>().parse::<u64>() {
                                self.buf = format!("{}", num).chars().collect();
                            } else {
                                self.buf.clear();
                            }
                        }
                        SearchMode::Integer(_, from_sign, _) => {
                            let numstr = self.buf.iter().collect::<String>();
                            if from_sign.is_signed() {
                                if let Ok(num) = numstr.parse::<i64>() {
                                    self.buf = match to_sign {
                                        Sign::Signed => match to_size {
                                            IntSize::I8  => format!("{}", num as i8),
                                            IntSize::I16 => format!("{}", num as i16),
                                            IntSize::I32 => format!("{}", num as i32),
                                            IntSize::I64 => format!("{}", num as i64),
                                        }
                                        Sign::Unsigned => match to_size {
                                            IntSize::I8  => format!("{}", num as u8),
                                            IntSize::I16 => format!("{}", num as u16),
                                            IntSize::I32 => format!("{}", num as u32),
                                            IntSize::I64 => format!("{}", num as u64),
                                        }
                                    }.chars().collect();
                                } else {
                                    self.buf.clear();
                                    self.buf.push('0');
                                }
                            } else if let Ok(num) = numstr.parse::<u64>() {
                                self.buf = match to_sign {
                                    Sign::Signed => match to_size {
                                        IntSize::I8  => format!("{}", num as i8),
                                        IntSize::I16 => format!("{}", num as i16),
                                        IntSize::I32 => format!("{}", num as i32),
                                        IntSize::I64 => format!("{}", num as i64),
                                    }
                                    Sign::Unsigned => match to_size {
                                        IntSize::I8  => format!("{}", num as u8),
                                        IntSize::I16 => format!("{}", num as u16),
                                        IntSize::I32 => format!("{}", num as u32),
                                        IntSize::I64 => format!("{}", num as u64),
                                    }
                                }.chars().collect();
                            } else {
                                self.buf.clear();
                                self.buf.push('0');
                            }
                        }
                    }
                }
            }

            self.mode = mode;
            self.cursor = self.buf.len();
            if self.cursor > self.size {
                self.view_offset = self.cursor - self.size;
            } else {
                self.view_offset = 0;
            }
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
                window.put_str(buf[cursor].to_string())?;
                window.turn_off_attributes(ColorPair(PAIR_INVERTED))?;

                if cursor + 1 < buf.len() {
                    let after: String = (&buf[cursor + 1..]).into_iter().collect();
                    window.turn_on_attributes(ColorPair(PAIR_NORMAL))?;
                    window.put_str(after)?;
                    window.turn_off_attributes(ColorPair(PAIR_NORMAL))?;
                }
            } else {
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
        // [ Binary     ]
        // [ String     ]
        // [ UInt 64 LE ]
        if self.size <= 16 {
            return Ok(());
        }

        let buf = &self.buf;
        window.move_to(pos)?;

        let mut len = buf.len();

        let cursor_at_end = self.cursor == buf.len();
        if cursor_at_end {
            len += 1;
        }

        let size = self.size - 16;
        if len > size {
            if self.view_offset > buf.len() {
                self.draw(window, 0, &[])?;
            } else {
                let mut index = self.view_offset;

                if cursor_at_end {
                    index += 1;
                }

                let mut end_index = index + size;
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
            for _ in 0..(size - len) {
                window.put_char(' ')?;
            }
        }

        let _ = window.put_str(format!(" [ {:<10} ]", self.mode));

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
            Input::Character('\n') | Input::KeyF3 => {
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
                if let Ok(bytes) = self.mode.parse(&self.buf) {
                    return Ok(WidgetResult::Value(bytes));
                }
                return Ok(WidgetResult::Ignore);
            }
            Input::Character(mut ch) => {
                match self.mode {
                    SearchMode::Integer(_, sign, _) => {
                        if ch == 'q' {
                            self.focused = false;
                            return Ok(WidgetResult::Redraw);
                        } else if self.buf.is_empty() && (ch == '+' || (sign.is_signed() && ch == '-')) {
                            self.buf.insert(self.cursor, ch);
                            self.cursor += 1;
                        } else {
                            self.buf.insert(self.cursor, ch);
                            
                            if self.mode.parse(&self.buf).is_ok() {
                                self.cursor += 1;
                            } else {
                                self.buf.remove(self.cursor);
                            }
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
                                    }
                                    self.cursor += 1;
                                }
                                2 => { panic!("invalid state"); }
                                _ => { panic!("x % 3 not in [0, 1, 2]!"); }
                            }
                        }
                    }
                }
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyIC => {
                if self.mode == SearchMode::Binary {
                    match self.cursor % 3 {
                        0 => {}
                        1 => {
                            self.cursor -= 1;
                        }
                        2 => { panic!("invalid state"); }
                        _ => { panic!("x % 3 not in [0, 1, 2]!"); }
                    }

                    self.buf.insert(self.cursor, ' ');
                    self.buf.insert(self.cursor, '0');
                    self.buf.insert(self.cursor, '0');

                    if self.cursor < self.view_offset {
                        self.view_offset = self.cursor;
                    }
                    return Ok(WidgetResult::Redraw);
                }
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
                                _ => { panic!("x % 3 not in [0, 1, 2]!"); }
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
                                _ => panic!("x % 3 not in [0, 1, 2]!")
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
            Input::KeyF5 => {
                self.set_search_mode(self.mode.next_major());
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyF6 => {
                self.set_search_mode(self.mode.next_sign());
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyF7 => {
                self.set_search_mode(self.mode.next_size());
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyF8 => {
                self.set_search_mode(self.mode.next_endian());
                return Ok(WidgetResult::Redraw);
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

