//! Compile project-owned Markdown into records consumed by `docs-base`.
//! Build scripts call [`compile`] and publish its assets beside their application.

mod error;
mod guides;
mod highlight;
mod html;
mod links;
mod markdown;

pub use error::Error;
pub use guides::{CompiledGuides, Config, compile};
pub use highlight::Highlighter;

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Shared layout and prose styles, including both syntax-highlighting themes.
const STYLESHEET: &str = include_str!("../assets/docs.css");

#[cfg(test)]
mod tests;
