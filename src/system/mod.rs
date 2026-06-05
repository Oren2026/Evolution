//! system — 系統環境檢查與自動安裝
//!
//! 提供 pre-flight check 功能，在執行前自動偵測必要元件是否齊備。

pub mod checker;
pub mod installer;
pub mod state_board;
pub mod state_board_storage;
pub mod event_generator;

pub use checker::{CheckStatus, SystemReport};
pub use installer::Installer;
pub use state_board::{BoardSummary, EventEntry, EventLevel, StateBoard, StateBoardSnapshot, TaskEntry, TaskStage};
pub use state_board_storage::StateBoardStorage;
pub use event_generator::EventGenerator;
