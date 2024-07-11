use dioxus::html::geometry::euclid::Rect;

use super::SnapInfo;

#[derive(Clone, PartialEq, Debug)]
pub struct DraggableTransitionData {
    pub from: SnapInfo,
    pub to: SnapInfo,
    pub mode: DraggableTransitionMode,
    id: String,
}

impl DraggableTransitionData {
    pub fn new(from: SnapInfo, to: SnapInfo, mode: DraggableTransitionMode, id: String) -> Self {
        let current = from.rect;
        Self { from, to, mode, id }
    }

    pub fn reverse(&self) -> DraggableTransitionData {
        DraggableTransitionData {
            from: self.to.clone(),
            to: self.from.clone(),
            mode: self.mode.reverse(),
            id: self.id.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DraggableTransitionMode {
    Avoidance,
    Resting,
}

impl DraggableTransitionMode {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Avoidance => Self::Resting,
            Self::Resting => Self::Avoidance,
        }
    }
}
