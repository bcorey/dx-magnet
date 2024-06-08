use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};

use crate::{dom_utilities::get_element_by_id, DraggableVariants};

#[derive(Clone, PartialEq)]
pub enum DragAreaStates {
    INITIAL,
    DRAGGING(DragAreaActiveDragData),
    RELEASED(DragReleaseStates),
}

#[derive(Clone, PartialEq)]
pub struct DragAreaActiveDragData {
    pub current_pos: Point2D<f64, ClientSpace>,
    pub starting_data: RectData,
}

impl DragAreaActiveDragData {
    pub fn update_current_pos(&mut self, new_pos: Point2D<f64, ClientSpace>) {
        self.current_pos = new_pos;
    }
}

#[derive(Clone, PartialEq)]
pub enum DragReleaseStates {
    SNAPPING(RectData),
    RELEASING(Point2D<f64, ClientSpace>),
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
        let start_rect = local_drag_info.read().get_rect();
        global_drag_info.write().start_drag(DragAreaActiveDragData {
            current_pos: event.data.client_coordinates(),
            starting_data: start_rect,
        });
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
            .on_component_update(global_drag_info.read().get_drag_state())
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
        if let DragAreaStates::DRAGGING(drag_data) = self.drag_state.clone() {
            self.drag_state = match self.snap_info {
                Some((snap_origin, snap_size)) => {
                    DragAreaStates::RELEASED(DragReleaseStates::SNAPPING(RectData {
                        position: snap_origin,
                        size: snap_size,
                    }))
                }
                None => {
                    DragAreaStates::RELEASED(DragReleaseStates::RELEASING(drag_data.current_pos))
                }
            };
        }
    }

    fn is_dragging(&self) -> bool {
        match self.drag_state {
            DragAreaStates::DRAGGING(_) => true,
            _ => false,
        }
    }

    fn start_drag(&mut self, drag_data: DragAreaActiveDragData) -> &mut Self {
        if let DragAreaStates::INITIAL | DragAreaStates::RELEASED(_) = self.drag_state {
            self.drag_state = DragAreaStates::DRAGGING(drag_data);
        }
        self
    }

    fn update_drag(&mut self, pos: Point2D<f64, ClientSpace>) {
        if let DragAreaStates::DRAGGING(mut drag_data) = self.drag_state.clone() {
            drag_data.update_current_pos(pos);
            self.drag_state = DragAreaStates::DRAGGING(drag_data);
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
    GRABBED((Point2D<f64, ElementSpace>, Point2D<f64, ClientSpace>)),
    RESTING(DraggableRestStates),
}

#[derive(Clone, Copy)]
enum DraggableRestStates {
    INITIAL,
    RELEASED(RectData),
    SNAPPED(DraggableSnapStates),
}

#[derive(Clone, PartialEq, Copy)]
enum DraggableSnapStates {
    PREVIEW { from: RectData, to: RectData },
    FINAL(RectData),
}

impl DraggableSnapStates {
    pub fn get_next_state(self, transition: DragReleaseStates) -> Self {
        match (self, transition) {
            (DraggableSnapStates::FINAL(from), DragReleaseStates::SNAPPING(to)) => {
                DraggableSnapStates::PREVIEW { from, to }
            }
            (
                DraggableSnapStates::PREVIEW { to, from },
                DragReleaseStates::SNAPPING(other_draggable_new_rect),
            ) => match other_draggable_new_rect.get_is_overlapping(from) {
                true => DraggableSnapStates::FINAL(to),
                false => DraggableSnapStates::FINAL(from),
            },
            (Self::PREVIEW { from, .. }, DragReleaseStates::RELEASING(_)) => Self::FINAL(from),
            (Self::FINAL(to), DragReleaseStates::RELEASING(_)) => Self::FINAL(to),
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
pub struct RectData {
    pub position: Point2D<f64, ClientSpace>,
    pub size: Point2D<f64, ClientSpace>,
}

impl RectData {
    pub fn get_is_within_bounds<U>(&self, point: Point2D<f64, U>) -> bool {
        (point.x >= self.position.x && point.x <= self.position.x + self.size.x)
            && (point.y >= self.position.y && point.y <= self.position.y + self.size.y)
    }

    pub fn get_is_overlapping(&self, other: RectData) -> bool {
        self.get_is_within_bounds(other.position)
    }
}

pub struct LocalDragState {
    drag_state: DraggableStates,
    draggable_variant: DraggableVariants,
    id: String,
}

impl LocalDragState {
    pub fn new(variant: DraggableVariants, id: String) -> Self {
        Self {
            drag_state: DraggableStates::RESTING(DraggableRestStates::INITIAL),
            draggable_variant: variant,
            id: id,
        }
    }

    pub fn get_element_id(&self) -> String {
        self.id.clone()
    }

    fn is_under_active_drag_pointer(&self, pointer: Point2D<f64, ClientSpace>) -> bool {
        self.get_rect().get_is_within_bounds(pointer)
    }

    fn transition_states(&mut self, release_state: DragReleaseStates) {
        self.drag_state = match (self.drag_state, release_state.clone()) {
            (
                DraggableStates::GRABBED((grab_location, _start_location)),
                DragReleaseStates::RELEASING(pointer_position),
            ) => {
                let x = pointer_position.x - grab_location.x;
                let y = pointer_position.y - grab_location.y;
                let resting_position: Point2D<f64, ClientSpace> = Point2D::new(x, y);
                let resting_size = self.get_rect().size;
                DraggableStates::RESTING(DraggableRestStates::RELEASED(RectData {
                    position: resting_position,
                    size: resting_size,
                }))
            }
            (
                // released after being dragged
                DraggableStates::GRABBED(_grab_location),
                DragReleaseStates::SNAPPING(snap_data),
            ) => DraggableStates::RESTING(DraggableRestStates::SNAPPED(
                DraggableSnapStates::FINAL(snap_data),
            )),
            (
                // snapped to a different location to get out of the way of a dragged item
                DraggableStates::RESTING(rest_type),
                DragReleaseStates::SNAPPING(snap_data),
            ) => {
                let snap_state = match rest_type {
                    DraggableRestStates::INITIAL => DraggableSnapStates::PREVIEW {
                        from: self.get_rect(),
                        to: snap_data,
                    },
                    DraggableRestStates::RELEASED(rect) => DraggableSnapStates::PREVIEW {
                        from: rect,
                        to: snap_data,
                    },
                    DraggableRestStates::SNAPPED(snap_state) => {
                        snap_state.get_next_state(release_state)
                    }
                };
                DraggableStates::RESTING(DraggableRestStates::SNAPPED(snap_state))
            }
            _ => self.drag_state.clone(),
        };
    }

    fn start_drag(&mut self, drag_point: Point2D<f64, ElementSpace>) {
        let start_location = self.get_rect().position;
        self.drag_state = DraggableStates::GRABBED((drag_point, start_location));
    }

    pub fn get_rect(&self) -> RectData {
        get_element_by_id(&self.id)
            .and_then(|el: web_sys::Element| {
                let rect = el.get_bounding_client_rect();
                let position: Point2D<f64, ClientSpace> = Point2D::new(rect.x(), rect.y());
                let size: Point2D<f64, ClientSpace> = Point2D::new(rect.width(), rect.height());
                Some(RectData { position, size })
            })
            .expect(format!("Could not get draggable {} by ID via DOM.", self.id).as_str())
    }

    // runs every component update
    pub fn on_component_update(&mut self, global_drag_state: DragAreaStates) -> String {
        match (self.drag_state.clone(), global_drag_state.clone()) {
            (
                DraggableStates::RESTING(
                    DraggableRestStates::INITIAL
                    | DraggableRestStates::RELEASED(_)
                    | DraggableRestStates::SNAPPED(DraggableSnapStates::FINAL(_)),
                ),
                DragAreaStates::DRAGGING(drag_data),
            ) if self.is_under_active_drag_pointer(drag_data.current_pos.clone()) => {
                let release_state = DragReleaseStates::SNAPPING(drag_data.starting_data);
                self.transition_states(release_state);

                self.on_component_update(global_drag_state)
            }
            (DraggableStates::RESTING(DraggableRestStates::INITIAL), _) => self.initial_style(),
            (DraggableStates::RESTING(DraggableRestStates::RELEASED(released_state)), _) => {
                self.location(released_state.position)
            }
            (
                DraggableStates::RESTING(DraggableRestStates::SNAPPED(DraggableSnapStates::FINAL(
                    snap_data,
                ))),
                _,
            ) => self.snapped_style(snap_data),
            (
                DraggableStates::RESTING(DraggableRestStates::SNAPPED(
                    DraggableSnapStates::PREVIEW {
                        from: original,
                        to: preview,
                    },
                )),
                _,
            ) => {
                match global_drag_state.clone() {
                    DragAreaStates::DRAGGING(data) => {
                        // if it is dragging inside the original rect of this element, set position to prview.
                        // otherwise change state to snapped to original rectData
                        if original.get_is_within_bounds(data.current_pos) {
                            return self.snapped_style(preview);
                        }
                        self.transition_states(DragReleaseStates::SNAPPING(original));
                        self.on_component_update(global_drag_state)
                    }
                    DragAreaStates::RELEASED(DragReleaseStates::SNAPPING(
                        other_draggable_new_rect,
                    )) => {
                        // if snap data is exactly the original rect, set position to snapped to preview rect data
                        if other_draggable_new_rect == original {
                            self.transition_states(DragReleaseStates::SNAPPING(preview));
                            return self.on_component_update(global_drag_state);
                        }
                        self.transition_states(DragReleaseStates::SNAPPING(original));
                        return self.on_component_update(global_drag_state);
                    }
                    _ => String::new(),
                }
            }
            (
                DraggableStates::GRABBED((grab_location, _original_location)),
                DragAreaStates::DRAGGING(drag_data),
            ) => self.location_with_grab_offset(grab_location, drag_data.current_pos),
            (DraggableStates::GRABBED(_grab_location), DragAreaStates::RELEASED(release_state)) => {
                self.transition_states(release_state);
                self.on_component_update(global_drag_state)
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

    fn snapped_style(&self, rect: RectData) -> String {
        format!(
            "{}{}{}\n width: {}px; height: {}px;",
            DRAGGABLE_BASE_STYLES,
            self.location(rect.position),
            SNAPPED_DRAGGABLE_STYLES,
            rect.size.x,
            rect.size.y
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
