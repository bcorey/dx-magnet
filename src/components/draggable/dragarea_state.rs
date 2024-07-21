use super::{DragOrigin, SnapInfo};
use dioxus::prelude::*;
use dioxus_elements::geometry::euclid::Point2D;

#[derive(Clone, PartialEq, Debug)]
pub enum DragAreaStates {
    Initial,
    Dragging(DragAreaActiveDragData),
    Released(DragEndings),
}

#[derive(Clone, PartialEq, Debug)]
pub struct DragAreaActiveDragData {
    pub current_pos: Point2D<f64, f64>,
    pub starting_data: DragOrigin,
}

impl DragAreaActiveDragData {
    pub fn update_current_pos(&mut self, new_pos: Point2D<f64, f64>) {
        self.current_pos = new_pos;
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum DragEndings {
    Snapping(SnapInfo),
    Releasing(Point2D<f64, f64>),
}
pub struct DraggableStateController;

impl DraggableStateController {
    pub fn stop_drag(mut global_drag_info: Signal<GlobalDragState>) {
        if global_drag_info.peek().is_dragging() {
            global_drag_info.write().stop_drag();
        }
    }
}

const DRAG_AREA_BASE_STYLES: &str = r#"
    background-image: radial-gradient(black .05rem, transparent 0);
    background-size: .6rem .6rem;
    width: 100%;
    height: 100%;
"#;

const DRAG_AREA_ACTIVE_STYLES: &str = r#"
    -webkit-user-select: none;
    user-select: none;
"#;

#[derive(Clone)]
pub struct GlobalDragState {
    drag_state: DragAreaStates,
    snap_info: Option<SnapInfo>,
}

impl GlobalDragState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_drag_state(&self) -> DragAreaStates {
        self.drag_state.clone()
    }

    pub fn set_snap_info(&mut self, info: Option<SnapInfo>) {
        tracing::info!("set snap info on area: {:?}", info);
        self.snap_info = info;
    }

    pub fn get_snap_info(&self) -> Option<SnapInfo> {
        self.snap_info.clone()
    }

    fn stop_drag(&mut self) {
        if let DragAreaStates::Dragging(drag_data) = self.drag_state.clone() {
            self.drag_state = match self.snap_info.clone() {
                Some(info) => DragAreaStates::Released(DragEndings::Snapping(info)),
                None => DragAreaStates::Released(DragEndings::Releasing(drag_data.current_pos)),
            };
            tracing::info!("ending drag {:?}", self.drag_state);
        }
    }

    pub fn is_dragging(&self) -> bool {
        matches!(self.drag_state, DragAreaStates::Dragging(_))
    }

    pub fn start_drag(&mut self, drag_data: DragAreaActiveDragData) -> &mut Self {
        if let DragAreaStates::Initial | DragAreaStates::Released(_) = self.drag_state {
            self.drag_state = DragAreaStates::Dragging(drag_data);
        }
        self
    }

    pub fn update_drag(&mut self, pos: Point2D<f64, f64>) {
        if let DragAreaStates::Dragging(mut drag_data) = self.drag_state.clone() {
            drag_data.update_current_pos(pos);
            self.drag_state = DragAreaStates::Dragging(drag_data);
        }
    }

    pub fn get_drag_area_style(&self) -> String {
        match self.is_dragging() {
            true => format!("{}{}", DRAG_AREA_BASE_STYLES, DRAG_AREA_ACTIVE_STYLES),
            false => DRAG_AREA_BASE_STYLES.to_string(),
        }
    }
}

impl Default for GlobalDragState {
    fn default() -> Self {
        Self {
            drag_state: DragAreaStates::Initial,
            snap_info: None,
        }
    }
}
