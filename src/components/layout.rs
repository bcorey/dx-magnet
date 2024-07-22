use dioxus::prelude::*;

const CONTAINER_STYLE: &str = "
    width: 100%;
    height: 100%;
    padding: 0;
    margin: 0;
    display: grid;
    grid-template-rows: auto;
    position: absolute;
    top: 0;
    left: 0;
";

#[component]
pub fn Container(columns: Option<u64>, rows: Option<u64>, children: Element) -> Element {
    let mut style = CONTAINER_STYLE.to_string();

    if let Some(num) = rows {
        style = format!(
            "{}\n grid-template-rows: repeat({}, minmax(0, 1fr));",
            style, num
        )
    }

    if let Some(num) = columns {
        style = format!(
            "{}\n grid-template-columns: repeat({}, minmax(0, 1fr));",
            style, num
        );
    }
    rsx! {
        div {
            style: style,
            {children}
        }
    }
}

const CELL_STYLE: &str = "
    height: 100%;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
    display: grid;
";

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
        style = format!(
            "{}\n grid-template-rows: repeat({}, minmax(0, 1fr));",
            style, num
        )
    }

    if let Some(num) = columns {
        style = format!(
            "{}\n grid-template-columns: repeat({}, minmax(0, 1fr));",
            style, num
        );
    }

    rsx! {
        div {
            style: style,
            {children}
        }
    }
}

const WINDOW_STYLE: &str = "
    position: relative;
    border: 0.05rem solid var(--fg);
    padding-top: 1rem;
    box-sizing: border-box;
    background-color: var(--bg);
    transition: inherit;
    height: 100%;
    max-height: 100%;
    display: flex;
    align-items: center;
    align-content: center;
    text-align: center;
    flex-flow: column;
    border-radius: inherit;
";

#[component]
pub fn Window(children: Element) -> Element {
    let style = WINDOW_STYLE.to_string();
    rsx! {
        div {
            style: style,
            {children}
        }
    }
}
