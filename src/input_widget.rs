use pancurses_result::{Window, Point};

use crate::result::Result;

pub trait InputWidget<V> {
    fn has_focus(&self) -> bool {
        false
    }

    fn focus(&mut self, _initial_value: V) -> Result<()> {
        Ok(())
    }

    fn blur(&mut self) -> Result<()> {
        Ok(())
    }

    fn redraw<P>(&self, _window: &mut Window, _pos: P) -> Result<()>
    where P: Into<Point>, P: Copy {
        Ok(())
    }
}
