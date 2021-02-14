// This file is part of rust-hox.
//
// rust-hox is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-hox is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-hox.  If not, see <https://www.gnu.org/licenses/>.

use std::cmp::min;
use pancurses_result::{Window, Input, Dimension};

use crate::result::Result;
use crate::consts::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TextBoxResult {
    PropagateEvent,
    Redraw,
    Quit,
    Ignore,
}

pub struct TextBox<'a> {
    text: &'a str,
    lines: Vec<String>,
    win_size: Dimension,
    max_line_len: usize,
    view_offset:  usize,
    vpadding: u32,
    hpadding: u32,
    vdiff: u32,
    hdiff: u32,
}

impl<'a> TextBox<'a> {
    pub fn new(text: &'a str, hpadding: u32, vpadding: u32) -> Self {
        Self {
            text,
            lines: Vec::new(),
            win_size: Dimension::from((0, 0)),
            max_line_len: 0,
            view_offset: 0,
            vpadding,
            hpadding,
            vdiff: vpadding * 2 + 2,
            hdiff: hpadding * 2 + 2,
        }
    }

    fn page_height(&self) -> usize {
        if self.win_size.rows as usize > self.vdiff as usize {
            self.win_size.rows as usize - self.vdiff as usize
        } else {
            0
        }
    }

    fn max_view_offset(&self) -> usize {
        let vspace = self.page_height();
        
        if self.lines.len() > vspace {
            self.lines.len() - vspace
        } else {
            0
        }
    }

    pub fn resize(&mut self, size: &Dimension) -> Result<()> {
        if size.columns != self.win_size.columns {
            self.win_size.columns = size.columns;
            self.max_line_len = 0;
            if size.columns as usize > self.hdiff as usize {
                self.lines = wrap_lines(self.text, size.columns as usize - self.hdiff as usize);
                for line in &self.lines {
                    let line_len = line.chars().count();
                    if line_len > self.max_line_len {
                        self.max_line_len = line_len;
                    }
                }
            } else {
                self.lines.clear();
            }
        }

        if size.rows != self.win_size.rows {
            self.win_size.rows = size.rows;
            let max_view_offset = self.max_view_offset();
            if self.view_offset > max_view_offset {
                self.view_offset = max_view_offset;
            }
        }

        Ok(())
    }

    pub fn redraw(&self, window: &mut Window) -> Result<()> {
        if self.win_size.columns as usize > self.hdiff as usize && self.win_size.columns as usize > self.vdiff as usize {
            let width  = min(self.max_line_len + self.hdiff as usize, self.win_size.columns as usize);
            let height = min(self.lines.len() - self.view_offset + self.vdiff as usize, self.win_size.rows as usize);
            let x = (self.win_size.columns as usize - width) / 2;
            let y = (self.win_size.rows    as usize - height) / 2;

            draw_box(window, x as u32, y as u32, width as u32, height as u32)?;

            let x = x as i32 + 1 + self.hpadding as i32;
            let mut y = y as i32 + 1 + self.vpadding as i32;
            for line in &self.lines[self.view_offset..self.view_offset + height - self.vdiff as usize] {
                window.move_to((y, x))?;
                window.put_str(line)?;
                y += 1;
            }
        }

        Ok(())
    }

    pub fn handle(&mut self, input: Input) -> Result<TextBoxResult> {
        match input {
            Input::KeyHome => {
                self.view_offset = 0;
                Ok(TextBoxResult::Redraw)
            }
            Input::KeyEnd => {
                self.view_offset = self.max_view_offset();
                Ok(TextBoxResult::Redraw)
            }
            Input::KeyUp => {
                if self.view_offset > 0 {
                    self.view_offset -= 1;
                    Ok(TextBoxResult::Redraw)
                } else {
                    Ok(TextBoxResult::Ignore)
                }
            }
            Input::KeyDown => {
                if self.view_offset < self.max_view_offset() {
                    self.view_offset += 1;
                    Ok(TextBoxResult::Redraw)
                } else {
                    Ok(TextBoxResult::Ignore)
                }
            }
            Input::KeyNPage => {
                let page_height = self.page_height();
                let max_view_offset = self.max_view_offset();
                let new_view_offset = if self.view_offset + page_height < max_view_offset {
                    self.view_offset + page_height
                } else {
                    max_view_offset
                };
                if new_view_offset != self.view_offset {
                    self.view_offset = new_view_offset;
                    Ok(TextBoxResult::Redraw)
                } else {
                    Ok(TextBoxResult::Ignore)
                }
            }
            Input::KeyPPage => {
                let page_height = self.page_height();
                let new_view_offset = if self.view_offset > page_height {
                    self.view_offset - page_height
                } else {
                    0
                };
                if new_view_offset != self.view_offset {
                    self.view_offset = new_view_offset;
                    Ok(TextBoxResult::Redraw)
                } else {
                    Ok(TextBoxResult::Ignore)
                }
            }
            Input::Character(CANCEL) => { // Ctrl+Home
                if self.view_offset != 0 {
                    self.view_offset = 0;
                    Ok(TextBoxResult::Redraw)
                } else {
                    Ok(TextBoxResult::Ignore)
                }
            }
            Input::Character(DEVICE_CONTROL3) => { // Ctrl+End
                let max_view_offset = self.max_view_offset();
                if self.view_offset != max_view_offset {
                    self.view_offset = max_view_offset;
                    Ok(TextBoxResult::Redraw)
                } else {
                    Ok(TextBoxResult::Ignore)
                }
            }
            Input::KeyResize => {
                Ok(TextBoxResult::PropagateEvent)
            }
            Input::Character('q') | Input::Character(ESCAPE) | Input::Character(END_OF_TRANSMISSION) => {
                Ok(TextBoxResult::Quit)
            }
            _input => {
                Ok(TextBoxResult::Ignore)
            }
        }
    }
}

fn draw_box(window: &mut Window, x: u32, y: u32, width: u32, height: u32) -> Result<()> {
    if width > 1 && height > 1 {
        let mut y = y as i32;
        let mut x = x as i32;

        window.move_to((y, x))?;
        window.put_str("╔")?;
        for _ in 0..(width - 2) {
            window.put_str("═")?;
        }
        window.put_str("╗")?;
        y += 1;

        for _ in 0..(height - 2) {
            window.move_to((y, x))?;
            window.put_str("║")?;
            for _ in 0..(width - 2) {
                window.put_char(' ')?;
            }
            let _ = window.put_str("║  ");
            y += 1;
        }

        window.move_to((y, x))?;
        window.put_str("╚")?;
        for _ in 0..(width - 2) {
            window.put_str("═")?;
        }

        // reports bogus error on small windows:
        let _ = window.put_str("╝  ");
        y += 1;
        x += 1;

        let _ = window.move_to((y, x));
        for _ in 0..(width + 1) {
            let _ = window.put_char(' ');
        }
    }

    Ok(())
}

fn wrap_lines(text: &str, max_width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    if max_width > 0 {
        let mut newline = Vec::new();
        for line in text.split('\n') {
            if line.chars().count() > max_width {
                let mut first = true;
                let mut wrap_indent = 0;

                for word in line.split(' ') {
                    let word_len = word.chars().count();
                    let mut newlen = if first {
                        newline.len() + word_len
                    } else {
                        newline.len() + word_len + 1
                    };

                    if newline.len() > wrap_indent && newlen > max_width {
                        lines.push(newline.iter().collect());
                        newline.clear();
                        for _ in 0..wrap_indent {
                            newline.push(' ');
                        }
                        first  = true;
                        newlen = newline.len() + word_len;
                    }

                    if newlen <= max_width {
                        if !first {
                            newline.push(' ');
                        }
                        if wrap_indent == 0 && word_len >= 3 && word.chars().all(|ch| ch == '.') {
                            wrap_indent = newlen + 1;
                            if wrap_indent >= max_width {
                                wrap_indent = 0;
                            }
                        }
                        newline.extend(word.chars());
                        first = false;
                    } else {
                        // word is longer than available space,
                        // so we need to break the word itself up

                        // newline must be empty here

                        let word = word.chars().collect::<Vec<_>>();
                        let mut offset = 0;
                        while offset < word_len {
                            if newline.len() > wrap_indent {
                                lines.push(newline.iter().collect());
                                newline.clear();
                                for _ in 0..wrap_indent {
                                    newline.push(' ');
                                }
                            }
                            let new_offset = min(offset + max_width - wrap_indent, word_len);
                            newline.extend(&word[offset..new_offset]);
                            offset = new_offset;
                        }

                        first = false;
                    }
                }
                if newline.len() > 0 {
                    lines.push(newline.iter().collect());
                    newline.clear();
                }
            } else {
                lines.push(line.to_owned());
            }
        }
    }

    lines
}
