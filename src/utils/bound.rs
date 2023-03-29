use eframe::emath::Numeric;
use std::ops::{Bound, RangeBounds};

/// Extension methods for [`Bound`]
pub trait BoundExt<T> {
    fn value(&self) -> Option<&T>;

    fn variant_name(&self) -> &'static str;
}

impl<T> BoundExt<T> for Bound<T> {
    fn value(&self) -> Option<&T> {
        match self {
            Self::Included(value) => Some(value),
            Self::Excluded(value) => Some(value),
            Self::Unbounded => None,
        }
    }

    fn variant_name(&self) -> &'static str {
        match self {
            Self::Included(_) => "Included",
            Self::Excluded(_) => "Excluded",
            Self::Unbounded => "Unbounded",
        }
    }
}

/// Extension methods for [`RangeBounds`]
pub trait RangeBoundsExt<T> {
    fn start(&self) -> T;
    fn end(&self) -> T;
}

impl<T: Numeric, U: RangeBounds<T>> RangeBoundsExt<T> for U {
    fn start(&self) -> T {
        match self.start_bound() {
            Bound::Included(value) | Bound::Excluded(value) => *value,
            Bound::Unbounded => T::MIN,
        }
    }

    fn end(&self) -> T {
        match self.end_bound() {
            Bound::Included(value) | Bound::Excluded(value) => *value,
            Bound::Unbounded => T::MAX,
        }
    }
}
