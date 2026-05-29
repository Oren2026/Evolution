//! Dispatcher — 調度器
//!
//! 負責將任務分配給適當的節點。

/// 調度器
pub struct Dispatcher;

impl Dispatcher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_new() {
        let _disp = Dispatcher::new();
        assert!(true);
    }
}