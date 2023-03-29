use egui::{DroppedFile, HoveredFile};
use std::{
    borrow::Borrow,
    fmt::{self, Write},
};

/// Display trait
pub trait Trait {
    fn display(self) -> Display<Self>;
}

impl<T> Trait for T
where
    Display<T>: fmt::Display,
{
    fn display(self) -> Display<Self> {
        Display(self)
    }
}

// pub struct Masses<T>(T);
// impl<T: Borrow<[u64]>> fmt::Display for Masses<T> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         let mut offset = 0;
//         for mass in self.0.borrow() {
//             match mass - offset {
//                 f.write_char('-')?;
//             }
//             write!(f, "{mass}")?;
//             offset = mass - offset;
//         }
//         Ok(())
//     }
// }

/// Display
#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct Display<T: ?Sized>(T);

impl fmt::Display for Display<&DroppedFile> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(path) = &self.0.path {
            write!(f, "{}", path.display())?;
        } else if !self.0.name.is_empty() {
            write!(f, "{}", self.0.name)?;
        } else {
            f.write_str("??? ？")?;
        };
        if let Some(bytes) = &self.0.bytes {
            write!(f, " ({} bytes)", bytes.len()).ok();
        }
        Ok(())
    }
}

impl fmt::Display for Display<&HoveredFile> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(path) = &self.0.path {
            write!(f, "{}", path.display())?;
        } else if !self.0.mime.is_empty() {
            write!(f, "{}", self.0.mime)?;
        } else {
            f.write_str("??? ？")?;
        }
        Ok(())
    }
}
