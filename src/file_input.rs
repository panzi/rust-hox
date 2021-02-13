use std::path::{PathBuf};
use std::ffi::{OsStr};
use std::cmp::min;
use std::collections::vec_deque::VecDeque;

use pancurses_result::{Window, Point, Input, ColorPair, Dimension};

use crate::input_widget::{InputWidget, WidgetResult};
use crate::result::Result;
use crate::consts::*;

pub struct FileInput {
    buf: Vec<char>,
    autocomplete: Vec<char>,
    focused: bool,
    size: usize,
    cursor: usize,
    view_offset: usize,
    history: VecDeque<Vec<char>>,
    future:  VecDeque<Vec<char>>,
}

impl FileInput {
    pub fn new(size: usize) -> Self {
        Self {
            focused: false,
            buf: Vec::new(),
            autocomplete: Vec::new(),
            size,
            cursor: 0,
            view_offset: 0,
            history: VecDeque::new(),
            future:  VecDeque::new(),
        }
    }

    fn draw(&self, window: &mut Window, cursor: usize, buf: &[char], compl: &[char]) -> Result<()> {
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

                if !compl.is_empty() {
                    let compl: String = compl.into_iter().collect();
                    window.turn_on_attributes(ColorPair(PAIR_AUTO_COMPLETE))?;
                    window.put_str(compl)?;
                    window.turn_off_attributes(ColorPair(PAIR_AUTO_COMPLETE))?;
                }
            } else if !compl.is_empty() {
                window.turn_on_attributes(ColorPair(PAIR_INVERTED))?;
                window.put_str(compl[0].to_string())?;
                window.turn_off_attributes(ColorPair(PAIR_INVERTED))?;

                let compl: String = (&compl[1..]).into_iter().collect();
                window.turn_on_attributes(ColorPair(PAIR_AUTO_COMPLETE))?;
                window.put_str(compl)?;
                window.turn_off_attributes(ColorPair(PAIR_AUTO_COMPLETE))?;
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

            if compl.len() > 0 {
                let compl: String = compl.into_iter().collect();
                window.turn_on_attributes(ColorPair(PAIR_AUTO_COMPLETE))?;
                window.put_str(compl)?;
                window.turn_off_attributes(ColorPair(PAIR_AUTO_COMPLETE))?;
            }
        }

        Ok(())
    }

    fn autocomplete(&mut self) {
        self.autocomplete.clear();
        if self.buf.is_empty() {
            return;
        }

        let path = PathBuf::from(self.buf.iter().collect::<String>());
        
        if let Some(parent) = path.parent() {
            if let Some(leaf) = path.file_name() {
                let dirents = if parent == OsStr::new("") {
                    std::fs::read_dir(".")
                } else {
                    parent.read_dir()
                };
                if let Ok(dirents) = dirents {
                    let leaf = leaf.to_string_lossy().to_string();
                    let mut matches = Vec::new();
                    for dirent in dirents {
                        if let Ok(dirent) = dirent {
                            let file_name = dirent.file_name();
                            let name = file_name.to_string_lossy();
                            if name.starts_with(&leaf) {
                                matches.push(name.chars().collect::<Vec<_>>());
                            }
                        }
                    }

                    if let Some(mut prefix) = max_common_prefix(&matches) {
                        let mut path = parent.to_path_buf();
                        path.push(prefix.iter().collect::<String>());

                        if let Ok(meta) = path.metadata() {
                            if meta.file_type().is_dir() {
                                prefix.push(std::path::MAIN_SEPARATOR);
                            }
                        }

                        let index = leaf.chars().count();
                        prefix.drain(..index);

                        if prefix.len() != 1 || prefix[0] != std::path::MAIN_SEPARATOR {
                            self.autocomplete = prefix;
                        }
                    }
                }
            }
        }
    }
}

fn max_common_prefix<C>(list: &[impl AsRef<[C]>]) -> Option<Vec<C>>
where C: PartialEq, C: Copy {
    if list.is_empty() {
        return None;
    }

    let mut prefix = Vec::new();
    let mut index = 0;

    loop {
        let mut letter = None;
        for word in list {
            let word = word.as_ref();

            if index == word.len() {
                if prefix.is_empty() {
                    return None;
                }
                return Some(prefix);
            }

            if let Some(letter) = letter {
                if letter != word[index] {
                    if prefix.is_empty() {
                        return None;
                    }
                    return Some(prefix);
                }
            } else {
                letter = Some(word[index]);
            }
        }

        if let Some(letter) = letter {
            prefix.push(letter);
        }

        index += 1;
    }
}

impl InputWidget<&str, PathBuf> for FileInput {
    fn has_focus(&self) -> bool {
        self.focused
    }

    fn set_value(&mut self, value: &str) -> Result<()> {
        self.buf.splice(.., value.chars());
        self.cursor = self.buf.len();

        if self.cursor > self.size {
            self.view_offset = self.cursor - self.size;
        } else {
            self.view_offset = 0;
        }

        Ok(())
    }

    fn focus(&mut self) -> Result<()> {
        self.focused = true;

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
        let compl = &self.autocomplete;
        window.move_to(pos)?;

        let mut len = buf.len() + compl.len();

        let cursor_at_end = self.cursor == buf.len();
        if cursor_at_end && compl.len() == 0 && self.focused {
            len += 1;
        }

        if len > self.size {
            if self.view_offset > buf.len() {
                let compl = if compl.len() > self.size {
                    &compl[..self.size]
                } else {
                    &compl
                };
                self.draw(window, 0, &[], compl)?;
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
                let mut rest = if self.size > buf.len() {
                    self.size - buf.len()
                } else {
                    0
                };
                if cursor_at_end && rest > 0 {
                    rest -= 1;
                }
                let compl = if compl.len() > rest {
                    &compl[..rest]
                } else {
                    &compl
                };
                self.draw(window, cursor, buf, compl)?;
            }
        } else {
            self.draw(window, self.cursor, &buf, &compl)?;
            for _ in 0..(self.size - len) {
                window.put_char(' ')?;
            }
        }

        Ok(())
    }

    fn handle(&mut self, input: Input) -> Result<WidgetResult<PathBuf>> {
        if !self.focused {
            return Ok(WidgetResult::PropagateEvent);
        }

        match input {
            Input::Character('\t') => {
                self.buf.extend_from_slice(&self.autocomplete);
                self.cursor = self.buf.len();
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }
                self.autocomplete();
                return Ok(WidgetResult::Redraw);
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
            Input::Character(ESCAPE) | Input::Character(END_OF_TRANSMISSION) => {
                self.focused = false;
                return Ok(WidgetResult::Redraw);
            }
            Input::Character('\n') => {
                if self.buf.is_empty() {
                    return Ok(WidgetResult::Ignore);
                }
                self.focused = false;
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
                return Ok(WidgetResult::Value(PathBuf::from(self.buf.iter().collect::<String>())))
            }
            Input::Character(ch) => {
                self.buf.insert(self.cursor, ch);
                self.cursor += 1;
                if self.cursor > self.size {
                    self.view_offset = self.cursor - self.size;
                }
                self.autocomplete();
                return Ok(WidgetResult::Redraw);
            }
            Input::KeyDC => {
                if self.cursor < self.buf.len() {
                    self.buf.remove(self.cursor);
                    self.autocomplete();
                    return Ok(WidgetResult::Redraw);
                }
            }
            Input::KeyBackspace => {
                if self.cursor > 0 {
                    self.buf.remove(self.cursor - 1);
                    self.cursor -= 1;
                    if self.cursor < self.view_offset {
                        self.view_offset = self.cursor;
                    }
                    self.autocomplete();
                    return Ok(WidgetResult::Redraw);
                }
            }
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

