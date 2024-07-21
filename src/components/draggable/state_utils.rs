use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Rect;

#[derive(Debug, Clone, PartialEq)]
pub struct SnapInfo {
    pub rect: Rect<f64, f64>,
    pub target_id: Option<String>,
}

impl SnapInfo {
    pub fn new(target_id: Option<String>, rect: Rect<f64, f64>) -> Self {
        Self { rect, target_id }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragOrigin {
    Snapped(SnapInfo),
    Free(Rect<f64, f64>),
}

impl DragOrigin {
    pub fn get_snap_info(&self) -> SnapInfo {
        match self {
            Self::Snapped(snap_info) => snap_info.clone(),
            Self::Free(rect) => SnapInfo::new(None, *rect),
        }
    }
}
