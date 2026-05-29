//! Mailbox — 行程間通訊信箱
//!
//! 每個行程有自己的信箱，訊息經由 Kernel::syscall(Send) 投遞。
//! FIFO 佇列，非阻塞讀取。

use std::collections::VecDeque;

/// 信箱 — 行程的收件匣
#[derive(Debug, Clone)]
pub struct Mailbox {
    queue: VecDeque<String>,
}

impl Mailbox {
    /// 建立空信箱
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// 投遞訊息（經常由 Kernel 呼叫）
    pub fn send(&mut self, msg: String) {
        self.queue.push_back(msg);
    }

    /// 嘗試接收訊息（非阻塞，沒有則回 None）
    pub fn try_receive(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    /// 查看下一個訊息（不移除）
    pub fn peek(&self) -> Option<&String> {
        self.queue.front()
    }

    /// 是否有訊息
    pub fn has_messages(&self) -> bool {
        !self.queue.is_empty()
    }

    /// 訊息數量
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// 是否為空
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

impl Default for Mailbox {
    fn default() -> Self {
        Self::new()
    }
}