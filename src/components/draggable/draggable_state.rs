use super::{
    DragAreaActiveDragData, DragAreaStates, DragEndings, DragOrigin, DraggableTransitionData,
    DraggableTransitionMode, GridData, SnapInfo,
};
use crate::components::{
    draggable::DraggableRenderData, DragError, DragErrorType, DraggableVariants,
};
use dioxus::prelude::*;
use dioxus_elements::geometry::{
    euclid::{Point2D, Rect, Size2D},
    ElementSpace,
};

#[derive(Clone, Debug, PartialEq)]
pub enum DraggableStates {
    Initial,
    Grabbed(DraggableGrabData),
    Resting(DraggableRestStates),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DraggableGrabData {
    pub grab_point: Point2D<f64, ElementSpace>,
    pub drag_origin: DragOrigin,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DraggableRestStates {
    Released(Rect<f64, f64>),
    Snapped(DraggableSnapStates),
}

#[derive(Clone, PartialEq, Debug)]
pub enum DraggableSnapStates {
    Preview(DraggableTransitionData),
    Final(SnapInfo),
    Transitioning(DraggableTransitionData),
}

#[derive(Debug)]
pub struct LocalDragState {
    drag_state: DraggableStates,
    draggable_variant: DraggableVariants,
    id: String,
}

impl LocalDragState {
    pub fn new(variant: DraggableVariants, id: String) -> Self {
        Self {
            drag_state: DraggableStates::Initial,
            draggable_variant: variant,
            id,
        }
    }

    pub fn get_drag_state(&self) -> DraggableStates {
        self.drag_state.clone()
    }

    pub fn get_element_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_is_uninitialized(&self) -> bool {
        matches!(self.drag_state, DraggableStates::Initial)
    }
    pub fn initialize(&mut self, snap: SnapInfo) {
        let rest = match self.draggable_variant {
            DraggableVariants::DOCKED => {
                DraggableRestStates::Snapped(DraggableSnapStates::Final(snap))
            }
            DraggableVariants::FLOATING(_) => DraggableRestStates::Released(snap.rect),
        };

        self.drag_state = DraggableStates::Resting(rest);
    }

    pub fn resize_snapped(&mut self, grid: GridData) {
        tracing::info!("resizing {}", self.id);
        if let DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)) = &self.drag_state
        {
            let mut snap = match snap_state {
                DraggableSnapStates::Final(snap_info) => snap_info.clone(),
                DraggableSnapStates::Preview(preview_data) => preview_data.to.clone(),
                DraggableSnapStates::Transitioning(_transition) => return,
            };
            snap.rect = grid.get_new_child_rect(snap.rect);
            self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(
                DraggableSnapStates::Final(snap.clone()),
            ));
            tracing::info!("resized:{:?}", self.drag_state);
        } else {
            tracing::error!("could not resize");
        }
    }

    pub fn start_drag(
        &mut self,
        grab_point: Point2D<f64, ElementSpace>,
    ) -> Result<DraggableGrabData, DragError> {
        tracing::info!("start data: {:?}", self.drag_state);
        let drag_origin = match self.drag_state.clone() {
            DraggableStates::Resting(rest) => match rest {
                DraggableRestStates::Snapped(snap_data) => match snap_data {
                    DraggableSnapStates::Final(final_snap) => DragOrigin::Snapped(final_snap),
                    DraggableSnapStates::Transitioning(transition) => {
                        DragOrigin::Snapped(transition.to)
                    }
                    _ => {
                        tracing::error!("bad start: {:?}", snap_data);
                        return Err(DragError(DragErrorType::IllegalDragStart));
                    }
                },
                DraggableRestStates::Released(rect) => DragOrigin::Free(rect),
            },
            _ => return Err(DragError(DragErrorType::IllegalDragStart)),
        };
        let grab_data = DraggableGrabData {
            grab_point,
            drag_origin,
        };
        self.drag_state = DraggableStates::Grabbed(grab_data.clone());
        Ok(grab_data)
    }

    fn get_transition_end_state(
        &mut self,
        transition: DraggableTransitionData,
    ) -> DraggableSnapStates {
        match transition.mode {
            DraggableTransitionMode::Avoidance => DraggableSnapStates::Preview(transition),
            DraggableTransitionMode::Resting => DraggableSnapStates::Final(transition.to),
        }
    }

    pub fn update_state(&mut self, global_drag_state: DragAreaStates, rect: Rect<f64, f64>) {
        let old = self.drag_state.clone();
        match (self.drag_state.clone(), global_drag_state) {
            (DraggableStates::Initial, _) => return,
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(
                    DraggableSnapStates::Transitioning(transition),
                )),
                _,
            ) => {
                self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(
                    self.get_transition_end_state(transition),
                ));
            }
            (
                DraggableStates::Resting(draggable_rest_state),
                DragAreaStates::Dragging(drag_area_dragging_state),
            ) => self.update_state_while_other_is_dragged(
                draggable_rest_state,
                drag_area_dragging_state,
                rect,
            ),
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)),
                DragAreaStates::Released(drag_end_data),
            ) => self.update_state_on_other_drag_end(snap_state, drag_end_data),
            (
                DraggableStates::Grabbed(draggable_grab_data),
                DragAreaStates::Released(drag_area_dragging_state),
            ) => self.update_state_on_self_drag_end(
                draggable_grab_data,
                drag_area_dragging_state,
                rect,
            ),
            (_, _) => (),
        };

        let new = self.drag_state.clone();
        if old != new {
            tracing::info!("old: {:?} ..... vs new: {:?}", old, new);
        }
    }

    fn update_state_on_self_drag_end(
        &mut self,
        draggable_grab_data: DraggableGrabData,
        drag_area_dragging_state: DragEndings,
        rect: Rect<f64, f64>,
    ) {
        self.drag_state = match drag_area_dragging_state {
            DragEndings::Releasing(pointer_position) => {
                self.get_drag_end_release_state(pointer_position, draggable_grab_data, rect)
            }
            DragEndings::Snapping(snap_data) => {
                self.get_drag_end_snap_state(snap_data, draggable_grab_data)
            }
        };
        tracing::info!("data on drag end: {:?} {:?}", self.id, self.drag_state);
    }

    fn get_drag_end_release_state(
        &self,
        pointer_position: Point2D<f64, f64>,
        draggable_grab_data: DraggableGrabData,
        rect: Rect<f64, f64>,
    ) -> DraggableStates {
        let x = pointer_position.x - draggable_grab_data.grab_point.x;
        let y = pointer_position.y - draggable_grab_data.grab_point.y;
        let resting_position: Point2D<f64, f64> = Point2D::new(x, y);
        let resting_size = rect.size;
        DraggableStates::Resting(DraggableRestStates::Released(Rect::new(
            resting_position,
            resting_size,
        )))
    }

    fn get_drag_end_snap_state(
        &self,
        snap_data: SnapInfo,
        draggable_grab_data: DraggableGrabData,
    ) -> DraggableStates {
        let from = match draggable_grab_data.drag_origin {
            DragOrigin::Free(rect) => rect,
            DragOrigin::Snapped(og_snap) => og_snap.rect,
        };
        DraggableStates::Resting(DraggableRestStates::Snapped(
            DraggableSnapStates::Transitioning(DraggableTransitionData::new(
                SnapInfo::new(None, from),
                snap_data,
                DraggableTransitionMode::Resting,
                self.id.clone(),
            )),
        ))
    }

    fn update_state_on_other_drag_end(
        &mut self,
        snap_state: DraggableSnapStates,
        drag_end_data: DragEndings,
    ) {
        let new_snap_state = match (snap_state.clone(), drag_end_data) {
            (
                DraggableSnapStates::Preview(preview_data),
                DragEndings::Snapping(other_snap_info),
            ) => match other_snap_info.rect.intersects(&preview_data.from.rect) {
                true => DraggableSnapStates::Final(preview_data.to),
                false => DraggableSnapStates::Transitioning(preview_data.reverse()),
            },
            (DraggableSnapStates::Preview(preview_data), DragEndings::Releasing(_)) => {
                DraggableSnapStates::Transitioning(preview_data.reverse())
            }
            (DraggableSnapStates::Transitioning(transition), _) => {
                let res = self.get_transition_end_state(transition);
                match res {
                    DraggableSnapStates::Preview(preview) => DraggableSnapStates::Final(preview.to),
                    DraggableSnapStates::Transitioning(transition) => {
                        DraggableSnapStates::Final(transition.to)
                    }
                    _ => res,
                }
            }
            (_, _) => snap_state,
        };

        self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(new_snap_state));
        tracing::info!(
            "data on other drag end: {:?} {:?}",
            self.id,
            self.drag_state
        );
    }

    fn update_state_while_other_is_dragged(
        &mut self,
        draggable_rest_state: DraggableRestStates,
        drag_area_dragging_state: DragAreaActiveDragData,
        rect: Rect<f64, f64>,
    ) {
        let this_rect = match draggable_rest_state.clone() {
            DraggableRestStates::Released(rect) => rect,
            DraggableRestStates::Snapped(snap_state) => match snap_state {
                DraggableSnapStates::Final(rect) => rect.rect,
                DraggableSnapStates::Preview(transition) => transition.to.rect,
                DraggableSnapStates::Transitioning(_) => rect,
            },
        };
        let intersects_this_rect =
            this_rect.contains(drag_area_dragging_state.current_pos.cast_unit());
        match (draggable_rest_state.clone(), intersects_this_rect) {
            (DraggableRestStates::Released(_), _) => (), // no action
            (DraggableRestStates::Snapped(snap_state), _) => {
                self.get_next_snap_state(
                    snap_state,
                    intersects_this_rect,
                    drag_area_dragging_state,
                );
            }
        }
    }

    fn get_next_snap_state(
        &mut self,
        snap_state: DraggableSnapStates,
        intersects_pointer: bool,
        drag_area_dragging_state: DragAreaActiveDragData,
    ) {
        let snap_state = match (snap_state.clone(), intersects_pointer) {
            (DraggableSnapStates::Final(info), true) => {
                let start_snap = drag_area_dragging_state.starting_data.get_snap_info();
                DraggableSnapStates::Transitioning(DraggableTransitionData::new(
                    info,
                    start_snap,
                    DraggableTransitionMode::Avoidance,
                    self.id.clone(),
                ))
            }
            (DraggableSnapStates::Preview(transition), true) => {
                DraggableSnapStates::Transitioning(transition.reverse())
            }
            (DraggableSnapStates::Preview(transition), false)
                if !transition
                    .from
                    .rect
                    .contains(drag_area_dragging_state.current_pos.cast_unit()) =>
            {
                DraggableSnapStates::Transitioning(transition.reverse())
            }
            (DraggableSnapStates::Transitioning(transition), _) => {
                tracing::info!("transition during other drag");
                self.get_transition_end_state(transition)
            }
            (_, false) => snap_state,
        };

        self.drag_state = DraggableStates::Resting(DraggableRestStates::Snapped(snap_state));
    }

    pub fn get_render_data(
        &self,
        global_drag_state: DragAreaStates,
        rect: Rect<f64, f64>,
    ) -> DraggableRenderData {
        tracing::info!("getting render data");
        match (self.drag_state.clone(), global_drag_state.clone()) {
            (DraggableStates::Initial, _) => DraggableRenderData::default(),
            (DraggableStates::Grabbed(grab_data), DragAreaStates::Dragging(drag_data)) => {
                let origin =
                    Self::origin_with_grab_offset(grab_data.grab_point, drag_data.current_pos);
                let size = Self::get_grabbed_size();
                DraggableRenderData::free_or_dragging(Rect::new(origin, size))
            }
            (
                DraggableStates::Grabbed(grab_data),
                DragAreaStates::Released(DragEndings::Releasing(release)),
            ) => {
                let origin = Self::origin_with_grab_offset(grab_data.grab_point, release);
                let size = Self::get_grabbed_size();
                DraggableRenderData::free_or_dragging(Rect::new(origin, size))
            }
            (
                DraggableStates::Grabbed(_grab_data),
                DragAreaStates::Released(DragEndings::Snapping(_to)),
            ) => DraggableRenderData::free_or_dragging(rect),
            (
                DraggableStates::Resting(DraggableRestStates::Snapped(snap_state)),
                DragAreaStates::Dragging(_),
            ) => self.get_render_data_for_avoidance_states(snap_state),
            (DraggableStates::Resting(rest_state), _) => {
                self.get_render_data_for_resting_states(rest_state)
            }
            (_, _) => {
                tracing::error!(
                    "Illegal state when getting render data for {:?}. local state: {:?}, global state: {:?}",
                    self.id,
                    self.drag_state,
                    global_drag_state
                );
                DraggableRenderData::default()
            }
        }
    }

    fn get_render_data_for_resting_states(
        &self,
        draggable_rest_state: DraggableRestStates,
    ) -> DraggableRenderData {
        match draggable_rest_state {
            DraggableRestStates::Released(release_rect) => {
                let size = Self::get_grabbed_size();
                DraggableRenderData::free_or_dragging(Rect::new(release_rect.origin, size))
            }
            DraggableRestStates::Snapped(snap_state) => {
                self.get_render_data_for_avoidance_states(snap_state)
            }
        }
    }

    fn get_render_data_for_avoidance_states(
        &self,
        draggable_snap_state: DraggableSnapStates,
    ) -> DraggableRenderData {
        match draggable_snap_state {
            DraggableSnapStates::Final(snap_info) => DraggableRenderData::snapped(snap_info.rect),
            DraggableSnapStates::Preview(transition) => {
                DraggableRenderData::snapped(transition.to.rect)
            }
            DraggableSnapStates::Transitioning(transition) => {
                DraggableRenderData::transitioning(transition.anim)
            }
        }
    }

    fn origin_with_grab_offset(
        drag_point: Point2D<f64, ElementSpace>,
        pointer_pos: Point2D<f64, f64>,
    ) -> Point2D<f64, f64> {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;
        Point2D::new(x, y)
    }

    fn get_grabbed_size() -> Size2D<f64, f64> {
        Size2D::new(200., 200.)
    }
}
