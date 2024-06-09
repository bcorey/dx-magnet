use dioxus::prelude::*;

const BUTTON_STYLES: &str = r#"
    font-family: inherit;
    font-size: inherit;
    max-height: min-content;
    transition: box-shadow .05s ease-in-out;
    cursor: pointer;
    text-transform: uppercase;
    padding: .3rem .5rem;
    margin: .2rem .2rem .2rem 0;
    min-width: 6rem;
    max-width: 100%;
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
