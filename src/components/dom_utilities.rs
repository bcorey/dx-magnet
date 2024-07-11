use core::fmt;
use std::fmt::{Display, Formatter};

// pub fn get_element_by_id(id: &str) -> Result<web_sys::Element, DomRetrievalError> {
//     match web_sys::window()
//         .and_then(|win| win.document())
//         .and_then(|doc| doc.get_element_by_id(id))
//     {
//         Some(el) => Ok(el),
//         None => Err(DomRetrievalError::new(id.to_string())),
//     }
// }

#[derive(Debug)]
pub struct DomRetrievalError(String);

impl DomRetrievalError {
    fn new(id: String) -> Self {
        DomRetrievalError(id)
    }
}

impl Display for DomRetrievalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DOM Retrieval Error: Failed to find element with ID {}",
            self.0
        )
    }
}
