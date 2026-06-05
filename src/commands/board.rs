//! board — StateBoard CLI 命令
//!
//! `evolution board` — 顯示任務進度
//! `evolution board events` — 顯示未送達事件
//! `evolution board <task_id>` — 顯示單一任務詳情
//! `evolution board clear` — 清除已完成任務

use evolution_os::system::{BoardSummary, EventEntry, EventLevel, StateBoard, StateBoardStorage, TaskEntry, TaskStage};
use std::env;
use std::path::PathBuf;

fn evolution_root() -> PathBuf {
    env::var("EVOLUTION_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".evolution")
        })
}

fn storage_path() -> PathBuf {
    evolution_root().join("state_board.json")
}

fn load_board() -> StateBoard {
    let path = storage_path();
    let storage = StateBoardStorage::new(&path.to_string_lossy());
    storage.load().unwrap_or_else(|_| StateBoard::new())
}

fn save_board(board: &StateBoard) {
    let path = storage_path();
    let storage = StateBoardStorage::new(&path.to_string_lossy());
    let _ = storage.save(board);
}

// ═══════════════════════════════════════════════════════════════════
// 主要命令實作
// ═══════════════════════════════════════════════════════════════════

pub fn board(args: &[String]) -> Result<String, String> {
    let arg0 = args.get(0).map(|s| s.as_str());
    match (args.is_empty(), arg0) {
        (true, _) | (false, Some("status")) => show_board_summary(),
        (false, Some("events")) => show_events(),
        (false, Some("clear")) => clear_done(),
        _ => {
            // 嘗試當作 UUID
            if let Some(id) = args.first() {
                if id.len() == 36 {
                    return show_task_detail(id);
                }
            }
            Ok(help())
        }
    }
}

fn show_board_summary() -> Result<String, String> {
    let board = load_board();
    let summary = board.summary();

    let mut output = String::new();
    output.push_str("╔══════════════════════════════════════╗\n");
    output.push_str("║       Evolution State Board          ║\n");
    output.push_str("╚══════════════════════════════════════╝\n\n");

    output.push_str(&format!("總任務數：{}  tasks\n\n", summary.total));

    for (stage, count) in &summary.by_stage {
        let emoji = match stage.as_str() {
            "Pending" => "⚪",
            "Planning" => "🔵",
            "Executing" => "🟡",
            "Done" => "✅",
            "Blocked" => "🔴",
            _ => "⚫",
        };
        output.push_str(&format!("{} {}  -  {} tasks\n", emoji, stage, count));
    }

    if summary.undelivered_events > 0 {
        output.push_str(&format!(
            "\n📬 有 {} 個未送達事件，執行 `evolution board events` 查看\n",
            summary.undelivered_events
        ));
    }

    let all_tasks = board.list_tasks();
    if !all_tasks.is_empty() {
        output.push_str("\n─── 任務列表 ───\n");
        for task in all_tasks {
            let stage_emoji = match task.stage.as_str() {
                "Pending" => "⚪",
                "Planning" => "🔵",
                "Executing" => "🟡",
                "Done" => "✅",
                "Blocked" => "🔴",
                _ => "⚫",
            };
            output.push_str(&format!("{} `{}` {}\n", stage_emoji, task.id, task.name));
        }
    }

    Ok(output)
}

fn show_events() -> Result<String, String> {
    let board = load_board();
    let events: Vec<_> = board.undelivered_events().into_iter().cloned().collect();

    if events.is_empty() {
        return Ok("✅ 目前沒有未送達的事件".to_string());
    }

    let mut output = String::new();
    output.push_str("╔══════════════════════════════════════╗\n");
    output.push_str("║         未送達事件                   ║\n");
    output.push_str("╚══════════════════════════════════════╝\n\n");

    for (i, event) in events.iter().enumerate() {
        let level_icon = match event.level {
            EventLevel::Info => "ℹ️",
            EventLevel::Done => "✅",
            EventLevel::NeedsUser => "⚠️",
            EventLevel::Warning => "⚠️",
        };
        let task_info = event
            .task_id
            .as_deref()
            .map(|id| format!(" [任務: {}]", id))
            .unwrap_or_default();
        output.push_str(&format!(
            "{}. {} {}{}\n",
            i + 1,
            level_icon,
            event.message,
            task_info
        ));
    }

    Ok(output)
}

fn show_task_detail(task_id: &str) -> Result<String, String> {
    let board = load_board();
    let task = board
        .get_task(task_id)
        .ok_or_else(|| format!("找不到任務：{}", task_id))?;

    let mut output = String::new();
    output.push_str("╔══════════════════════════════════════╗\n");
    output.push_str("║         任務詳情                    ║\n");
    output.push_str("╚══════════════════════════════════════╝\n\n");

    let stage_emoji = match task.stage.as_str() {
        "Pending" => "⚪",
        "Planning" => "🔵",
        "Executing" => "🟡",
        "Done" => "✅",
        "Blocked" => "🔴",
        _ => "⚫",
    };
    output.push_str(&format!(
        "ID：{}  {} {}\n",
        task.id, stage_emoji, task.stage
    ));
    output.push_str(&format!(
        "建立：{}\n",
        task.created_at.format("%Y-%m-%d %H:%M:%S")
    ));
    output.push_str(&format!(
        "更新：{}\n",
        task.updated_at.format("%Y-%m-%d %H:%M:%S")
    ));

    if !task.history.is_empty() {
        output.push_str("\n─── 階段歷史 ───\n");
        for (i, h) in task.history.iter().enumerate() {
            output.push_str(&format!(
                "{}. {} → {}",
                i + 1,
                h.from.as_str(),
                h.to.as_str()
            ));
            if let Some(note) = &h.note {
                output.push_str(&format!(" [{}]", note));
            }
            output.push_str(&format!(" ({})\n", h.at.format("%m/%d %H:%M")));
        }
    }

    Ok(output)
}

fn clear_done() -> Result<String, String> {
    let mut board = load_board();
    let count = board.clear_done();
    save_board(&board);
    Ok(format!("已清除 {} 個已完成任務", count))
}

fn help() -> String {
    "evolution board — 狀態看板

用法：
  evolution board          顯示所有任務概覽
  evolution board events   顯示未送達事件
  evolution board <uuid>   顯示單一任務詳情
  evolution board clear    清除已完成任務
  evolution board --help   顯示幫助

範例：
  evolution board
  evolution board events
  evolution board 550e8400-e29b-41d4-a716-446655440000
  evolution board clear"
        .to_string()
}