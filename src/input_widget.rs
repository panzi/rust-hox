use pancurses_result::{Window, Point, Input, Dimension};

use crate::result::Result;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WidgetResult<V> {
    PropagateEvent,
    Redraw,
    Ignore,
    Value(V),
}

pub trait InputWidget<InValue, OutValue=InValue> {
    fn has_focus(&self) -> bool {
        false
    }

    fn focus(&mut self, _initial_value: InValue) -> Result<()> {
        Ok(())
    }

    fn blur(&mut self) -> Result<()> {
        Ok(())
    }

    fn redraw<P>(&self, _window: &mut Window, _pos: P) -> Result<()>
    where P: Into<Point>, P: Copy {
        Ok(())
    }

    fn handle(&mut self, _input: Input) -> Result<WidgetResult<OutValue>> {
        Ok(WidgetResult::PropagateEvent)
    }

    fn resize(&mut self, _size: &Dimension) -> Result<()> {
        Ok(())
    }
}
