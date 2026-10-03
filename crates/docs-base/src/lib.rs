//! Shared browser components for build-time compiled Markdown documentation.
//! Content, branding, routes and interactive examples belong to the consuming app.

mod article;
mod content;
mod markdown;
mod navigation;
mod shell;

pub use article::{Article, ArticleExtras, ArticleInputs};
pub use content::{Link, Page, Section, Site};
pub use navigation::{Location, Navigation};
pub use shell::{Shell, ShellInputs};
