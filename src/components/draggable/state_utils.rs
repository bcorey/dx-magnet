use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::{Point2D, Rect, Size2D};

#[derive(Debug, Clone, PartialEq)]
pub enum GridState {
    Initial,
    Mounted(Rect<f64, f64>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnData {
    column_width: f64,
    column_origin: Point2D<f64, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridData {
    columns: u8,
    rows: u8,
    state: GridState,
    prev_state: GridState,
}

impl GridData {
    pub fn new(columns: u8, rows: u8) -> Self {
        Self {
            columns,
            rows,
            state: GridState::Initial,
            prev_state: GridState::Initial,
        }
    }

    pub fn update_mounted(&mut self, rect: Rect<f64, f64>) {
        self.prev_state = self.state.clone();
        self.state = GridState::Mounted(rect);
    }

    pub fn get_grid_rect(&self) -> Option<Rect<f64, f64>> {
        match self.state {
            GridState::Mounted(rect) => Some(rect),
            GridState::Initial => None,
        }
    }

    pub fn get_new_child_rect(&self, grid_child_rect: Rect<f64, f64>) -> Rect<f64, f64> {
        match (&self.state, &self.prev_state) {
            (GridState::Mounted(new_grid_rect), GridState::Mounted(old_grid_rect)) => {
                let prop_x = grid_child_rect.width() / old_grid_rect.width();
                let prop_y = grid_child_rect.height() / old_grid_rect.height();
                let new_width = new_grid_rect.width() * prop_x;
                let new_height = new_grid_rect.height() * prop_y;
                let new_size: Size2D<f64, f64> = Size2D::new(new_width, new_height);
                tracing::info!(
                    "old size {:?} new size {:?}",
                    grid_child_rect.size,
                    new_size
                );

                let new_x = grid_child_rect.origin.x;
                let new_y = grid_child_rect.origin.y;
                let new_origin: Point2D<f64, f64> = Point2D::new(new_x, new_y);
                Rect::new(new_origin, new_size)
            }
            (_, _) => grid_child_rect,
        }
    }
}

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
