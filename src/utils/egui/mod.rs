pub use self::{
    collapsing_state::CollapsingStateExt,
    dropped_file::DroppedFileExt,
    response::{InnerResponseExt, ResponseExt},
    ui::UiExt,
};

use egui::{Response, Sense, Ui, Vec2, Widget};
use std::{fmt::Display, ops::BitOr};

// pub trait OptionalWidget<T> {
//     fn optional_widget<F: FnMut() -> Widget>(&mut self, f: F) -> Response;
// }

pub trait SelectableValueFromIter<T> {
    fn selectable_value_from_iter(
        &mut self,
        current_value: &mut T,
        values: impl Iterator<Item = T>,
    ) -> Response;
}

impl<T> SelectableValueFromIter<T> for Ui
where
    T: PartialEq + Display + Copy,
{
    fn selectable_value_from_iter(
        &mut self,
        current_value: &mut T,
        values: impl Iterator<Item = T>,
    ) -> Response {
        values
            .map(|value| self.selectable_value(current_value, value, format!("{value}")))
            .reduce(BitOr::bitor)
            .unwrap_or_else(|| {
                self.colored_label(self.style().visuals.error_fg_color, "⚠ No items")
            })
    }
}

mod collapsing_state;
mod dropped_file;
mod response;
mod ui;
