use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};

#[derive(Clone, PartialEq)]
enum GlobalDragStates {
    INITIAL,
    DRAGGING(Point2D<u32, ClientSpace>),
    RESTING(Point2D<u32, ClientSpace>),
}
struct DraggableStateController;

impl DraggableStateController {
    fn start_drag(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        mut local_drag_info: Signal<LocalDragState>,
    ) {
        local_drag_info
            .write()
            .start_drag(event.data.element_coordinates().to_u32());
        global_drag_info
            .write()
            .start_drag(event.data.client_coordinates().to_u32());
    }

    fn update_drag(
        event: PointerEvent,
        mut global_drag_info: Signal<GlobalDragState>,
        active: bool,
    ) {
        if !active || !global_drag_info.read().is_dragging() {
            return;
        }
        let point = event.data.client_coordinates().to_u32();
        global_drag_info.write().update_drag(point);
    }

    fn stop_drag(mut global_drag_info: Signal<GlobalDragState>) {
        global_drag_info.write().stop_drag();
    }
}

#[derive(Clone)]
struct GlobalDragState {
    drag_state: GlobalDragStates,
}

impl GlobalDragState {
    fn new() -> Self {
        Self {
            drag_state: GlobalDragStates::INITIAL,
        }
    }

    fn stop_drag(&mut self) {
        self.drag_state = match self.drag_state {
            GlobalDragStates::DRAGGING(position) => GlobalDragStates::RESTING(position),
            _ => self.drag_state.clone(),
        };
    }

    fn is_dragging(&self) -> bool {
        match self.drag_state {
            GlobalDragStates::DRAGGING(_) => true,
            _ => false,
        }
    }

    fn start_drag(&mut self, position: Point2D<u32, ClientSpace>) -> &mut Self {
        self.drag_state = match self.drag_state {
            GlobalDragStates::INITIAL | GlobalDragStates::RESTING(_) => {
                GlobalDragStates::DRAGGING(position)
            }
            _ => self.drag_state.clone(),
        };
        self
    }

    fn update_drag(&mut self, pos: Point2D<u32, ClientSpace>) {
        self.drag_state = match self.drag_state {
            GlobalDragStates::DRAGGING(_) => GlobalDragStates::DRAGGING(pos),
            _ => self.drag_state.clone(),
        };
    }

    fn get_drag_area_style(&self) -> String {
        match self.is_dragging() {
            true => DRAG_AREA_STYLES.to_string(),
            false => String::new(),
        }
    }
}

#[derive(Clone)]
enum LocalDragStates {
    INITIAL,
    GRABBED(Point2D<u32, ElementSpace>),
    RESTING(Point2D<u32, ClientSpace>),
}

struct LocalDragState {
    drag_state: LocalDragStates,
}

impl LocalDragState {
    fn new() -> Self {
        Self {
            drag_state: LocalDragStates::INITIAL,
        }
    }

    fn stop_dragging(&mut self, pointer_position: Point2D<u32, ClientSpace>) {
        self.drag_state = match self.drag_state {
            LocalDragStates::GRABBED(grab_location) => {
                let x = pointer_position.x - grab_location.x;
                let y = pointer_position.y - grab_location.y;
                let resting_position: Point2D<u32, ClientSpace> = Point2D::new(x, y);
                LocalDragStates::RESTING(resting_position)
            }
            _ => self.drag_state.clone(),
        };
    }

    fn start_drag(&mut self, drag_point: Point2D<u32, ElementSpace>) {
        self.drag_state = LocalDragStates::GRABBED(drag_point);
    }

    fn get_position(&mut self, global_drag_state: GlobalDragStates) -> String {
        // match global_drag_state {
        //     GlobalDragStates::DRAGGING(latest_pointer_position) => {
        //         Self::location_with_grab_offset(grab_location, latest_pointer_position)
        //     }
        //     GlobalDragStates::RESTING(final_pointer_position) => match self.drag_state {
        //         LocalDragStates::INITIAL => "".to_string(),
        //         LocalDragStates::RESTING(resting_position) => Self::location(resting_position),
        //         LocalDragStates::GRABBED(grab_location) => {
        //             self.stop_dragging(final_pointer_position);
        //         }
        //     },
        //     _ => "".to_string(),
        // }
        match self.drag_state {
            LocalDragStates::INITIAL => "".to_string(),
            LocalDragStates::RESTING(resting_position) => Self::location(resting_position),
            LocalDragStates::GRABBED(grab_location) => match global_drag_state {
                GlobalDragStates::DRAGGING(latest_pointer_position) => {
                    Self::location_with_grab_offset(grab_location, latest_pointer_position)
                }
                GlobalDragStates::RESTING(final_pointer_position) => {
                    self.stop_dragging(final_pointer_position);
                    self.get_position(global_drag_state)
                }
                _ => "".to_string(),
            },
        }
    }

    fn location_with_grab_offset(
        drag_point: Point2D<u32, ElementSpace>,
        pointer_pos: Point2D<u32, ClientSpace>,
    ) -> String {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;

        format!("{}\n left: {}px; top: {}px;", DRAGGABLE_STYLES, x, y)
    }

    fn location(pos: Point2D<u32, ClientSpace>) -> String {
        format!(
            "{}\n left: {}px; top: {}px;",
            DRAGGABLE_STYLES, pos.x, pos.y
        )
    }
}

const DRAG_AREA_STYLES: &str = r#"
    -webkit-user-select: none;
    user-select: none;
"#;

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let global_drag_info = use_context_provider(|| Signal::new(GlobalDragState::new()));

    let style = global_drag_info.read().get_drag_area_style();

    rsx! {
        div {
            style: style,
            onpointermove: move |event| DraggableStateController::update_drag(event, global_drag_info, active),
            onpointerup: move |_| DraggableStateController::stop_drag(global_drag_info),
            {children},
        }
    }
}

const DRAGGABLE_STYLES: &str = r#"
    position: absolute;
    background-color: var(--accent_1);
"#;

#[component]
pub fn Draggable(children: Element) -> Element {
    let mut local_drag_info = use_context_provider(|| Signal::new(LocalDragState::new()));
    let global_drag_info = use_context::<Signal<GlobalDragState>>();
    let style = local_drag_info
        .write()
        .get_position(global_drag_info.read().drag_state.clone());

    rsx! {
        div {
            style: style,
            DragHandle {}
            {children}
        }
    }
}

const DRAG_HANDLE_STYLES: &str = r#"
    width: 100%;
    height: 2rem;
    position: relative;
    background-color: var(--hint);
    z-index: 2000;
    cursor: grab;
"#;

#[component]
fn DragHandle() -> Element {
    let global_drag_info = use_context::<Signal<GlobalDragState>>();
    let local_drag_info = use_context::<Signal<LocalDragState>>();

    rsx! {
        div {
            style: DRAG_HANDLE_STYLES,
            onpointerdown: move |event| DraggableStateController::start_drag(event, global_drag_info, local_drag_info),
        }
    }
}
