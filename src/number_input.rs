use std::str::FromStr;
use std::fmt::Display;
use std::fmt::Write;

use crate::input_widget::{InputWidget, WidgetResult};
use crate::result::Result;
use crate::consts::{
    PAIR_NORMAL, PAIR_INPUT, PAIR_INPUT_ERROR, ESC,
};

use pancurses_result::{
    Input, Point, Window, ColorPair,
};

pub struct NumberInput<N>
where N: FromStr, N: Display {
    focused: bool,
    size: usize,
    buf:  String,
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
            error: false,
            phantom: std::marker::PhantomData,
        }
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

        Ok(())
    }

    fn blur(&mut self) -> Result<()> {
        self.focused = false;

        Ok(())
    }

    fn redraw<P>(&self, window: &mut Window, pos: P) -> Result<()>
    where P: Into<Point>, P: Copy {
        let buf = &self.buf;
        window.move_to(pos)?;

        let col = if !self.focused {
            ColorPair(PAIR_NORMAL)
        } else if self.error {
            ColorPair(PAIR_INPUT_ERROR)
        } else {
            ColorPair(PAIR_INPUT)
        };
        window.turn_on_attributes(col)?;
        if buf.len() > self.size {
            if self.size > 3 {
                window.put_str(format!("...{}", &buf[buf.len() - (self.size - 3)..]))?;
            } else {
                window.put_str(&buf[buf.len() - self.size..])?;
            }
        } else {
            window.put_str(format!("{:>1$}", buf, self.size))?;
        }
        window.turn_off_attributes(col)?;

        Ok(())
    }

    fn handle(&mut self, input: Input) -> Result<WidgetResult<N>> {
        if !self.focused {
            return Ok(WidgetResult::PropagateEvent);
        }

        match input {
            Input::Character('q') | Input::Character(ESC) => {
                self.focused = false;
                return Ok(WidgetResult::Redraw);
            },
            Input::Character('\n') => {
                if let Ok(num) = self.buf.parse() {
                    self.focused = false;
                    return Ok(WidgetResult::Value(num));
                } else {
                    self.error = true;
                }
            },
            Input::Character('c') | Input::KeyDC => {
                self.buf.clear();
                self.error = false;
            },
            Input::KeyBackspace => {
                self.buf.pop();
                self.error = if self.buf.is_empty() { false }
                             else { self.buf.parse::<usize>().is_err() };
            },
            Input::Character(c) if c >= '0' && c <= '9' && self.buf.len() < 20 => {
                self.buf.push(c);
                self.error = self.buf.parse::<N>().is_err();
            },
            Input::Character(_) | Input::KeyLeft | Input::KeyRight | Input::KeyUp | Input::KeyDown => {
                return Ok(WidgetResult::Ignore);
            },
            _input => {
                return Ok(WidgetResult::PropagateEvent);
            },
        }

        Ok(WidgetResult::Redraw)
    }
}
