//! Runtime — 執行期
//!
//! v0.1: 僅預留結構，不實作功能。
//! 後續版本將陸續填充：
//! - v0.3: Executor（執行器）
//! - v0.4: Dispatcher（調度器）
//! - v0.6: ModelDispatcher（AI syscall）

pub mod dispatcher;
pub mod executor;
pub mod graph_executor;

pub use dispatcher::Dispatcher;
pub use executor::Executor;
pub use graph_executor::GraphExecutor;