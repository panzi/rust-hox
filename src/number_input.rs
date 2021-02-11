use std::str::FromStr;
use std::fmt::Display;
use std::fmt::Write;
use std::cmp::min;

use crate::input_widget::{InputWidget, WidgetResult};
use crate::result::Result;
use crate::consts::{
    PAIR_NORMAL, PAIR_INPUT_ERROR, ESC, PAIR_INVERTED,
};

use pancurses_result::{
    Input, Point, Window, ColorPair,
};

pub struct NumberInput<N>
where N: FromStr, N: Display {
    focused: bool,
    size: usize,
    buf:  String,
    cursor: usize,
    view_offset: usize,
    error: bool,
    phantom: std::marker::PhantomData<N>,
}

impl<N> NumberInput<N>
where N: FromStr, N: Display {
    pub fn new(size: usize) -> Self {
        Self {
            focused: false,
            size,
            buf: String::new(),
            cursor: 0,
            view_offset: 0,
            error: false,
            phantom: std::marker::PhantomData,
        }
    }

    // since we control the characters that can be in buf we know its ASCII
    // and these slices are safe
    fn draw(&self, window: &mut Window, cursor: usize, buf: &str) -> Result<()> {
        let attrs = if self.error {
            ColorPair(PAIR_INPUT_ERROR)
        } else {
            ColorPair(PAIR_NORMAL)
        };

        if cursor > 0 {
            let before = &buf[..cursor];
            window.turn_on_attributes(attrs)?;
            window.put_str(before)?;
            window.turn_off_attributes(attrs)?;
        }

        if cursor < buf.len() {
            window.turn_on_attributes(ColorPair(PAIR_INVERTED))?;
            window.put_str(&buf[cursor..cursor + 1])?;
            window.turn_off_attributes(ColorPair(PAIR_INVERTED))?;
        }

        if cursor + 1 < buf.len() {
            let after = &buf[cursor + 1..];
            window.turn_on_attributes(attrs)?;
            window.put_str(after)?;
            window.turn_off_attributes(attrs)?;
        } else if cursor >= buf.len() && self.focused {
            window.turn_on_attributes(ColorPair(PAIR_INVERTED))?;
            window.put_char(' ')?;
            window.turn_off_attributes(ColorPair(PAIR_INVERTED))?;
        }

        Ok(())
    }

}

impl<N> InputWidget<N> for NumberInput<N>
where N: FromStr, N: Display {
    fn has_focus(&self) -> bool {
        self.focused
    }

    fn focus(&mut self, initial_value: N) -> Result<()> {
        self.focused = true;
        self.error = false;
        self.buf.clear();
        write!(self.buf, "{}", initial_value).unwrap();
        self.cursor = self.buf.len();
        if self.cursor > self.size {
            self.view_offset = self.cursor - self.size;
        } else {
            self.view_offset = 0;
        }

        Ok(())
    }

    fn blur(&mut self) -> Result<()> {
        self.focused = false;

        Ok(())
    }

    fn redraw<P>(&self, window: &mut Window, pos: P) -> Result<()>
    where P: Into<Point>, P: Copy {
        if self.size == 0 {
            return Ok(());
        }

        let buf = &self.buf;
        window.move_to(pos)?;

        let mut len = buf.len();

        let cursor_at_end = self.cursor == buf.len();
        if cursor_at_end && self.focused {
            len += 1;
        }

        if len > self.size {
            if self.view_offset > buf.len() {
                self.draw(window, 0, "")?;
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
            let attrs = if self.error {
                ColorPair(PAIR_INPUT_ERROR)
            } else {
                ColorPair(PAIR_NORMAL)
            };
            window.turn_on_attributes(attrs)?;
            for _ in 0..(self.size - len) {
                window.put_char(' ')?;
            }
            window.turn_off_attributes(attrs)?;
            self.draw(window, self.cursor, &buf)?;
        }

        Ok(())
    }

    fn handle(&mut self, input: Input) -> Result<WidgetResult<N>> {
        if !self.focused {
            return Ok(WidgetResult::PropagateEvent);
        }

        match input {
            Input::Character(ch) if ((ch >= '0' && ch <= '9') || ch == '+' || ch == '-' || ch == '.' || ch == 'e' || ch == 'E') && self.buf.len() < 20 => {
                self.buf.insert(self.cursor, ch);
                self.error = self.buf.parse::<N>().is_err();
                self.cursor += 1;
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }
                return Ok(WidgetResult::Redraw);
            }
            Input::Character('x') => {
                self.buf.clear();
                self.cursor = 0;
                self.view_offset = 0;
                self.error = false;
            }
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
                    if self.cursor < self.view_offset {
                        self.view_offset = self.cursor;
                    }
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::KeyRight => {
                if self.cursor < self.buf.len() {
                    self.cursor += 1;
                    if self.cursor > self.size {
                        self.view_offset = self.cursor - self.size;
                    }
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::KeyDC => {
                if self.cursor < self.buf.len() {
                    self.buf.remove(self.cursor);
                    self.error = if self.buf.is_empty() { false }
                                 else { self.buf.parse::<usize>().is_err() };
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::KeyBackspace => {
                if self.cursor > 0 {
                    self.buf.remove(self.cursor - 1);
                    self.cursor -= 1;
                    if self.view_offset > 0 {
                        self.view_offset -= 1;
                    }
                    self.error = if self.buf.is_empty() { false }
                                 else { self.buf.parse::<usize>().is_err() };
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::Character('q') | Input::Character(ESC) => {
                self.focused = false;
                return Ok(WidgetResult::Redraw);
            }
            Input::Character('\n') => {
                if let Ok(num) = self.buf.parse() {
                    self.focused = false;
                    self.error = false;
                    return Ok(WidgetResult::Value(num));
                } else {
                    self.error = true;
                }
            }
            Input::KeyUp | Input::KeyDown => {
                return Ok(WidgetResult::Ignore);
            }
            _input => {
                return Ok(WidgetResult::PropagateEvent);
            },
        }

        Ok(WidgetResult::Redraw)
    }
}
