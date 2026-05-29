//! commands — CLI commands implementation

pub mod new_project;
pub mod analyze;
pub mod list_skills;
pub mod shell;

pub use new_project::new_project;
pub use analyze::analyze;
pub use list_skills::list_skills;
pub use shell::shell;