use std::fmt::{self, Display};

use derive_more::Deref;

pub macro selector($raw:expr) {{
    static SELECTOR: once_cell::sync::OnceCell<scraper::Selector> =
        once_cell::sync::OnceCell::new();
    SELECTOR.get_or_init(|| scraper::Selector::parse($raw).unwrap())
}}

#[derive(Debug, Deref, Clone, PartialEq, Eq, Hash)]
pub struct RawUrl(pub String);

impl From<&str> for RawUrl {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl Display for RawUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
