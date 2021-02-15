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

use pancurses_result::{Window, Point, Input, Dimension};

use crate::result::Result;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WidgetResult<V> {
    PropagateEvent,
    Redraw,
    Ignore,
    Beep,
    Value(V),
}

pub trait InputWidget<InValue, OutValue=InValue> {
    fn has_focus(&self) -> bool {
        false
    }

    fn set_value(&mut self, _value: InValue) -> Result<()> {
        Ok(())
    }

    fn focus(&mut self) -> Result<()> {
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
