use dioxus::prelude::*;

const BUTTON_STYLES: &str = r#"
    font-family: inherit;
    font-size: inherit;
    max-width: fit-content;
    transition: transform .05s ease-in-out;
    transition: box-shadow .05s ease-in-out;
    cursor: pointer;
    text-transform: uppercase;
    padding: .3rem .5rem;
    margin: .2rem .2rem .2rem 0;
    min-width: 8rem;
    background-color: var(--bg);
    border: .05rem solid var(--fg);
    color: var(--fg);
    box-shadow: .3rem .2rem 0 0 var(--hint);
    border-radius: 0;
"#;

#[component]
pub fn Button(name: String) -> Element {
    rsx! {
        button {
            style: BUTTON_STYLES,
            "{name}",
        }
    }
}