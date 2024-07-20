use core::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct DragError(pub DragErrorType);

impl Display for DragError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub enum DragErrorType {
    IllegalDragStart,
}

impl Display for DragErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllegalDragStart => write!(f, "Invalid conditions for Drag Start"),
        }
    }
}
