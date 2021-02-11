use std::cmp::min;
use pancurses_result::{Window, Input, Dimension};

use crate::result::Result;
use crate::consts::*;

#[derive(Clone, Copy, PartialEq)]
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
            let width  = self.max_line_len + self.hdiff as usize;
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
            Input::KeyResize => {
                Ok(TextBoxResult::PropagateEvent)
            }
            Input::Character('q') | Input::Character(ESC) => {
                Ok(TextBoxResult::Quit)
            },
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
    let mut lines = Vec::new();

    if max_width > 0 {
        for line in text.split('\n') {
            if line.chars().count() > max_width {
                let mut newline = String::new();
                let mut char_count = 0;
                for word in line.split(' ') {
                    let word_len = word.chars().count();
                    if char_count + word_len < max_width {
                        if char_count != 0 {
                            newline.push(' ');
                            char_count += 1;
                        }
                        newline.push_str(word);
                        char_count += word_len;
                    } else {
                        if char_count != 0 {
                            lines.push(newline.clone());
                            newline.clear();
                        }
                        let word = word.chars().collect::<Vec<_>>();
                        let mut index = 0;
                        let reminder = word.len() % max_width;
                        let end_index = word.len() - reminder;
                        while index < end_index {
                            let next = index + max_width;
                            lines.push((&word[index..next]).iter().collect());
                            index = next;
                        }
                        newline.extend(&word[index..]);
                        char_count = reminder;
                    }
                }
                if char_count > 0 {
                    lines.push(newline);
                }
            } else {
                lines.push(line.to_owned());
            }
        }
    }

    lines
}
