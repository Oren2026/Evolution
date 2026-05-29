//! system — 系統環境檢查與自動安裝
//!
//! 提供 pre-flight check 功能，在執行前自動偵測必要元件是否齊備。

pub mod checker;
pub mod installer;

pub use checker::{CheckStatus, SystemReport};
pub use installer::Installer;
