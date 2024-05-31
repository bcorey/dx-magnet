use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};

use crate::DraggableVariants;

#[derive(Clone, PartialEq)]
pub enum DragAreaStates {
    INITIAL,
    DRAGGING(Point2D<f64, ClientSpace>),
    RELEASED(DragReleaseStates),
}

#[derive(Clone, PartialEq)]
pub enum DragReleaseStates {
    SNAPPING((Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)),
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
    snap_info: Option<(Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)>,
}

impl GlobalDragState {
    pub fn new() -> Self {
        Self {
            drag_state: DragAreaStates::INITIAL,
            snap_info: None,
        }
    }

    pub fn get_drag_state(&self) -> DragAreaStates {
        self.drag_state.clone()
    }

    pub fn set_snap_info(
        &mut self,
        info: Option<(Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)>,
    ) {
        self.snap_info = info;
    }

    pub fn get_snap_info(&self) -> Option<(Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)> {
        return self.snap_info;
    }

    fn stop_drag(&mut self) {
        if let DragAreaStates::DRAGGING(position) = self.drag_state {
            self.drag_state = match self.snap_info {
                Some((snap_origin, snap_size)) => {
                    DragAreaStates::RELEASED(DragReleaseStates::SNAPPING((snap_origin, snap_size)))
                }
                None => DragAreaStates::RELEASED(DragReleaseStates::RESTING(position)),
            };
        }
    }

    fn is_dragging(&self) -> bool {
        match self.drag_state {
            DragAreaStates::DRAGGING(_) => true,
            _ => false,
        }
    }

    fn start_drag(&mut self, position: Point2D<f64, ClientSpace>) -> &mut Self {
        if let DragAreaStates::INITIAL | DragAreaStates::RELEASED(_) = self.drag_state {
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

const DRAGGABLE_BASE_STYLES: &str = r#"
    display: flex;
    flex-flow: column;
"#;

const DRAGGABLE_STYLES: &str = r#"
    position: absolute;
    background-color: var(--accent_0);
    z-index: 10000;
    width: 180px;
    height: 200px;
    box-shadow: .4rem .3rem var(--hint);
"#;

const SNAPPED_DRAGGABLE_STYLES: &str = r#"
    transition: left .1s ease-in-out, top .1s ease-in-out, width .1s ease-in-out .1s, height .1s ease-in-out .1s, box-shadow .1s ease-in-out;
    box-shadow: 0 0 solid var(--hint);
    z-index: 100;
"#;

#[derive(Clone, Copy)]
enum DraggableStates {
    INITIAL,
    GRABBED(Point2D<f64, ElementSpace>),
    RESTING(Point2D<f64, ClientSpace>),
    SNAPPED((Point2D<f64, ClientSpace>, Point2D<f64, ClientSpace>)),
}

pub struct LocalDragState {
    drag_state: DraggableStates,
    draggable_variant: DraggableVariants,
}

impl LocalDragState {
    pub fn new(variant: DraggableVariants) -> Self {
        Self {
            drag_state: DraggableStates::INITIAL,
            draggable_variant: variant,
        }
    }

    fn stop_dragging(&mut self, release_state: DragReleaseStates) {
        self.drag_state = match (self.drag_state, release_state) {
            (
                DraggableStates::GRABBED(grab_location),
                DragReleaseStates::RESTING(pointer_position),
            ) => {
                let x = pointer_position.x - grab_location.x;
                let y = pointer_position.y - grab_location.y;
                let resting_position: Point2D<f64, ClientSpace> = Point2D::new(x, y);
                DraggableStates::RESTING(resting_position)
            }
            (
                DraggableStates::GRABBED(_grab_location),
                DragReleaseStates::SNAPPING(snap_position),
            ) => DraggableStates::SNAPPED(snap_position),
            _ => self.drag_state.clone(),
        };
    }

    fn start_drag(&mut self, drag_point: Point2D<f64, ElementSpace>) {
        self.drag_state = DraggableStates::GRABBED(drag_point);
    }

    pub fn get_position(&mut self, global_drag_state: DragAreaStates) -> String {
        match (self.drag_state.clone(), global_drag_state.clone()) {
            (DraggableStates::INITIAL, _) => self.initial_style(),
            (DraggableStates::RESTING(resting_position), _) => self.location(resting_position),
            (DraggableStates::SNAPPED(snap_data), _) => {
                self.snapped_style(snap_data.0, snap_data.1)
            }
            (
                DraggableStates::GRABBED(grab_location),
                DragAreaStates::DRAGGING(latest_pointer_position),
            ) => self.location_with_grab_offset(grab_location, latest_pointer_position),
            (DraggableStates::GRABBED(_grab_location), DragAreaStates::RELEASED(release_state)) => {
                self.stop_dragging(release_state);
                self.get_position(global_drag_state)
            }
            (DraggableStates::GRABBED(_), DragAreaStates::INITIAL) => {
                tracing::error!("Draggable grabbed with area in initial state");
                String::new()
            }
        }
    }

    fn location_with_grab_offset(
        &self,
        drag_point: Point2D<f64, ElementSpace>,
        pointer_pos: Point2D<f64, ClientSpace>,
    ) -> String {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;

        format!(
            "{}{}\n left: {}px; top: {}px;",
            DRAGGABLE_BASE_STYLES, DRAGGABLE_STYLES, x, y
        )
    }

    fn location(&self, pos: Point2D<f64, ClientSpace>) -> String {
        format!(
            "{}{}\n left: {}px; top: {}px;",
            DRAGGABLE_BASE_STYLES, DRAGGABLE_STYLES, pos.x, pos.y
        )
    }

    fn snapped_style(
        &self,
        pos: Point2D<f64, ClientSpace>,
        size: Point2D<f64, ClientSpace>,
    ) -> String {
        format!(
            "{}{}{}\n width: {}px; height: {}px;",
            DRAGGABLE_BASE_STYLES,
            self.location(pos),
            SNAPPED_DRAGGABLE_STYLES,
            size.x,
            size.y
        )
    }

    fn initial_style(&self) -> String {
        match self.draggable_variant {
            DraggableVariants::DOCKED => {
                format!("display: flex; flex-flow: column; height: 100%;")
            }
            DraggableVariants::FLOATING(_pos) => String::new(),
        }
    }
}
