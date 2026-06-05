//! commands — CLI commands implementation

pub mod intent_router;
pub mod new_project;
pub mod analyze;
pub mod list_skills;
pub mod shell;
pub mod status;
pub mod init;
pub mod board;

pub mod ollama_check;
pub mod boot;
pub mod test_record;

pub use new_project::new_project;
pub use analyze::analyze;
pub use list_skills::list_skills;
pub use shell::shell;
pub use status::status;
pub use init::init;
pub use ollama_check::{check as ollama_check, install as ollama_install, start as ollama_start};
pub use boot::{boot, boot_json, evolution_root};
pub use test_record::{record_test, start_test, list_tests, all_tests_json};
pub use board::board;