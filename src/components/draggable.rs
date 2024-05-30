use dioxus::prelude::*;
use dioxus_elements::geometry::{euclid::Point2D, ClientSpace, ElementSpace};
use tracing::info;

struct GlobalDragInfo {
    cur_pos: Option<Point2D<u32, ClientSpace>>,
    last_pos: Option<Point2D<u32, ClientSpace>>,
}

impl GlobalDragInfo {
    fn new() -> Self {
        Self {
            cur_pos: None,
            last_pos: None,
        }
    }

    fn stop_drag(&mut self) {
        self.cur_pos.map(|cur_pos| self.last_pos = Some(cur_pos));
        self.cur_pos = None;
    }

    fn is_dragging(&self) -> bool {
        self.cur_pos.is_some()
    }

    fn set_position(&mut self, pos: Point2D<u32, ClientSpace>) {
        self.cur_pos = Some(pos);
    }
}

struct LocalDragInfo {
    element_drag_location: Option<Point2D<u32, ElementSpace>>,
    is_dragging: bool,
    stored_style: String,
}

impl LocalDragInfo {
    fn new() -> Self {
        Self {
            element_drag_location: None,
            is_dragging: false,
            stored_style: String::new(),
        }
    }

    fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    fn stop_dragging(&mut self, style: String) {
        self.is_dragging = false;
        self.element_drag_location = None;
        self.stored_style = style;
    }

    fn start_dragging(&mut self, drag_point: Point2D<u32, ElementSpace>) {
        self.element_drag_location = Some(drag_point);
        self.is_dragging = true;
    }

    fn get_style(
        &mut self,
        cur_pos: Option<Point2D<u32, ClientSpace>>,
        last_pos: Option<Point2D<u32, ClientSpace>>,
    ) -> String {
        if !self.is_dragging() {
            return self.stored_style.clone();
        }

        if let Some(drag_info) = cur_pos {
            return Self::build_style(self.element_drag_location.unwrap(), drag_info);
        } else {
            let last_pos = last_pos.unwrap();

            let style = LocalDragInfo::build_style(self.element_drag_location.unwrap(), last_pos);

            self.stop_dragging(style.clone());
            return style;
        }
    }

    fn build_style(
        drag_point: Point2D<u32, ElementSpace>,
        pointer_pos: Point2D<u32, ClientSpace>,
    ) -> String {
        let x = pointer_pos.x - drag_point.x;
        let y = pointer_pos.y - drag_point.y;

        format!("{}\n left: {}px; top: {}px;", DRAGGABLE_STYLES, x, y)
    }
}

const DRAG_AREA_STYLES: &str = r#"
    -webkit-user-select: none;
    user-select: none;
"#;

#[component]
pub fn DragArea(active: bool, children: Element) -> Element {
    let drag_info = use_context_provider(|| Signal::new(GlobalDragInfo::new()));

    let mut style = String::new();
    if drag_info.read().is_dragging() {
        style = DRAG_AREA_STYLES.to_string();
    }

    let on_pointer_move =
        move |event: PointerEvent, mut drag_info: Signal<GlobalDragInfo>, active: bool| {
            if !active || !drag_info.read().is_dragging() {
                return;
            }
            let point = event.data.client_coordinates().to_u32();
            drag_info.write().set_position(point);
        };

    let on_pointer_up = move |mut drag_info: Signal<GlobalDragInfo>| {
        drag_info.write().stop_drag();
    };

    rsx! {
        div {
            style: style,
            onpointermove: move |event| on_pointer_move(event, drag_info, active),
            onpointerup: move |_| on_pointer_up(drag_info),
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
    let mut local_drag_info = use_context_provider(|| Signal::new(LocalDragInfo::new()));
    let global_drag_info = use_context::<Signal<GlobalDragInfo>>();
    let style = local_drag_info.write().get_style(
        global_drag_info.read().cur_pos,
        global_drag_info.read().last_pos,
    );

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
    let global_drag_info = use_context::<Signal<GlobalDragInfo>>();
    let local_drag_info = use_context::<Signal<LocalDragInfo>>();

    let on_pointer_down =
        move |event: PointerEvent,
              mut global_drag_info: Signal<GlobalDragInfo>,
              mut local_drag_info: Signal<LocalDragInfo>| {
            info!("click on drag handle");
            local_drag_info
                .write()
                .start_dragging(event.data.element_coordinates().to_u32());
            global_drag_info
                .write()
                .set_position(event.data.client_coordinates().to_u32());
        };
    rsx! {
        div {
            style: DRAG_HANDLE_STYLES,
            onpointerdown: move |event| on_pointer_down(event, global_drag_info, local_drag_info),
        }
    }
}
