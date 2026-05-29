//! evolution — CLI 主程式
//!
//! 用法：
//!   evolution new <name>           建立專案
//!   evolution analyze <task>       Planner → Executor 一鍵執行
//!   evolution list-skills          列出可用技能
//!   evolution shell                互動模式

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;

#[derive(Parser)]
#[command(
    name = "evolution",
    about = "Evolution OS — AI Native Development Framework",
    version = "0.3.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 建立新專案
    New {
        #[arg(help = "專案名稱")]
        name: String,
    },
    /// 一鍵分析任務：Planner → Executor
    Analyze {
        #[arg(help = "任務描述")]
        task: String,
        #[arg(long, help = "指定專案（預設新建）")]
        project: Option<String>,
    },
    /// 列出可用技能
    ListSkills,
    /// 互動式 Shell（基本版）
    Shell,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::New { name } => {
            commands::new_project(&name);
        }
        Command::Analyze { task, project } => {
            commands::analyze(&task, project.as_deref());
        }
        Command::ListSkills => {
            commands::list_skills();
        }
        Command::Shell => {
            commands::shell();
        }
    }
}