use core::fmt;
use std::fmt::{Display, Formatter};

use crate::components::dom_utilities::DomRetrievalError;

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
    DomRetrievalError(DomRetrievalError),
}

impl Display for DragErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IllegalDragStart => write!(f, "Invalid conditions for Drag Start"),
            Self::DomRetrievalError(err) => err.fmt(f),
        }
    }
}
