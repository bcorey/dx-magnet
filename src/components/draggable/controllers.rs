use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};

#[derive(Clone, PartialEq)]
pub enum DragAreaStates {
    INITIAL,
    DRAGGING(Point2D<f64, ClientSpace>),
    RESTING(Point2D<f64, ClientSpace>),
}
pub struct DraggableStateController;

impl DraggableStateController {
    pub fn start_drag(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        mut local_drag_info: Signal<LocalDragState>,
    ) {
        local_drag_info
            .write()
            .start_drag(event.data.element_coordinates());
        global_drag_info
            .write()
            .start_drag(event.data.client_coordinates());
    }

    pub fn update_drag_area(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        active: bool,
    ) {
        if !active || !global_drag_info.read().is_dragging() {
            return;
        }
        let point = event.data.client_coordinates();
        global_drag_info.write().update_drag(point);
    }

    pub fn update_draggable_position(
        mut local_drag_info: Signal<LocalDragState>,
        global_drag_info: Signal<GlobalDragState>,
    ) -> String {
        local_drag_info
            .write()
            .get_position(global_drag_info.read().get_drag_state())
    }

    pub fn stop_drag(mut global_drag_info: Signal<GlobalDragState>) {
        global_drag_info.write().stop_drag();
    }
}

const DRAG_AREA_STYLES: &str = r#"
    -webkit-user-select: none;
    user-select: none;
"#;

#[derive(Clone)]
pub struct GlobalDragState {
    drag_state: DragAreaStates,
}

impl GlobalDragState {
    pub fn new() -> Self {
        Self {
            drag_state: DragAreaStates::INITIAL,
        }
    }

    pub fn get_drag_state(&self) -> DragAreaStates {
        self.drag_state.clone()
    }

    fn stop_drag(&mut self) {
        if let DragAreaStates::DRAGGING(position) = self.drag_state {
            self.drag_state = DragAreaStates::RESTING(position);
        }
    }

    fn is_dragging(&self) -> bool {
        match self.drag_state {
            DragAreaStates::DRAGGING(_) => true,
            _ => false,
        }
    }

    fn start_drag(&mut self, position: Point2D<f64, ClientSpace>) -> &mut Self {
        if let DragAreaStates::INITIAL | DragAreaStates::RESTING(_) = self.drag_state {
            self.drag_state = DragAreaStates::DRAGGING(position);
        }
        self
    }

    fn update_drag(&mut self, pos: Point2D<f64, ClientSpace>) {
        if let DragAreaStates::DRAGGING(_) = self.drag_state {
            self.drag_state = DragAreaStates::DRAGGING(pos)
        }
    }

    pub fn get_drag_area_style(&self) -> String {
        match self.is_dragging() {
            true => DRAG_AREA_STYLES.to_string(),
            false => String::new(),
        }
    }
}

const DRAGGABLE_STYLES: &str = r#"
    position: absolute;
    background-color: var(--accent_1);
"#;

#[derive(Clone, Copy)]
enum DraggableStates {
    INITIAL,
    GRABBED(Point2D<f64, ElementSpace>),
    RESTING(Point2D<f64, ClientSpace>),
}

pub struct LocalDragState {
    drag_state: DraggableStates,
}

impl LocalDragState {
    pub fn new() -> Self {
        Self {
            drag_state: DraggableStates::INITIAL,
        }
    }

    fn stop_dragging(&mut self, pointer_position: Point2D<f64, ClientSpace>) {
        self.drag_state = match self.drag_state {
            DraggableStates::GRABBED(grab_location) => {
                let x = pointer_position.x - grab_location.x;
                let y = pointer_position.y - grab_location.y;
                let resting_position: Point2D<f64, ClientSpace> = Point2D::new(x, y);
                DraggableStates::RESTING(resting_position)
            }
            _ => self.drag_state.clone(),
        };
    }

    fn start_drag(&mut self, drag_point: Point2D<f64, ElementSpace>) {
        self.drag_state = DraggableStates::GRABBED(drag_point);
    }

    pub fn get_position(&mut self, global_drag_state: DragAreaStates) -> String {
        match (self.drag_state.clone(), global_drag_state.clone()) {
            (DraggableStates::INITIAL, _) => String::new(),
            (DraggableStates::RESTING(resting_position), _) => Self::location(resting_position),
            (
                DraggableStates::GRABBED(grab_location),
                DragAreaStates::DRAGGING(latest_pointer_position),
            ) => Self::location_with_grab_offset(grab_location, latest_pointer_position),
            (
                DraggableStates::GRABBED(_grab_location),
                DragAreaStates::RESTING(final_pointer_position),
            ) => {
                self.stop_dragging(final_pointer_position);
                self.get_position(global_drag_state)
            }
            (DraggableStates::GRABBED(_), DragAreaStates::INITIAL) => {
                tracing::error!("Draggable grabbed with area in initial state");
                String::new()
            }
        }
    }

    fn location_with_grab_offset(
        drag_point: Point2D<f64, ElementSpace>,
        pointer_pos: Point2D<f64, ClientSpace>,
    ) -> String {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;

        format!("{}\n left: {}px; top: {}px;", DRAGGABLE_STYLES, x, y)
    }

    fn location(pos: Point2D<f64, ClientSpace>) -> String {
        format!(
            "{}\n left: {}px; top: {}px;",
            DRAGGABLE_STYLES, pos.x, pos.y
        )
    }
}
