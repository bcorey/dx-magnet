use dioxus::prelude::*;

const CONTAINER_STYLE: &str = r#"
    width: 100%;
    height: 100%;
    padding: 0;
    margin: 0;
    background-color: var(--bg);
    display: grid;
    grid-template-columns: repeat(var(--display_columns), 1fr);
    grid-template-rows: auto;
    position: absolute;
    top: 0;
    left: 0;
"#;

#[component]
pub fn Container(children: Element) -> Element {
    rsx! {
        div {
            style: CONTAINER_STYLE,
            {children},
        }
    }
}

const CELL_STYLE: &str = r#"
    height: 100%;
    padding: 0;
    margin: 0;
    border: .05rem solid var(--fg);
    box-sizing: border-box;
    display: grid;
"#;

#[component]
pub fn Cell(
    span: std::ops::Range<u64>,
    rows: Option<u64>,
    columns: Option<u64>,
    children: Element,
) -> Element {
    let mut style = format!(
        "{}\n grid-column: {}/span {};",
        CELL_STYLE, span.start, span.end
    );

    if let Some(num) = rows {
        style = format!("{}\n grid-template-rows: repeat({}, auto);", style, num)
    }

    if let Some(num) = columns {
        style = format!("{}\n grid-template-columns: repeat({}, auto);", style, num);
    }

    rsx! {
        div {
            style: style,
            {children}
        }
    }
}

const WINDOW_STYLE: &str = r#"
    box-shadow: .4rem .3rem var(--hint);
    width: content-width;
    height: content-height;
    position: relative;
    z-index: 1000;
    border: 0.05rem solid var(--fg);
    padding: 1rem;
    box-sizing: border-box;
    background-color: var(--bg);
"#;

#[component]
pub fn Window(children: Element) -> Element {
    rsx! {
        div {
            style: WINDOW_STYLE,
            {children}
        }
    }
}
