//! evolution — CLI 主程式
//!
//! 用法：
//!   evolution new <name>           建立專案
//!   evolution analyze <task>       Planner → Executor 一鍵執行
//!   evolution status              系統狀態（加 --json 輸出 JSON）
//!   evolution init [project]      生成概述檔供 OpenCode 理解
//!   evolution list-skills          列出可用技能
//!   evolution shell                互動模式

use clap::{Parser, Subcommand};

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
        #[arg(long, help = "輸出 JSON 格式")]
        json: bool,
    },
    /// 系統狀態
    Status {
        #[arg(long, help = "輸出 JSON 格式")]
        json: bool,
    },
    /// 生成概述檔（供 OpenCode 理解專案）
    Init {
        #[arg(help = "專案名稱（不指定則為系統概述）")]
        project: Option<String>,
    },
    /// 列出可用技能
    ListSkills,
    /// 互動式 Shell
    Shell,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::New { name } => {
            commands::new_project(&name);
        }
        Command::Analyze { task, project, json } => {
            commands::analyze(&task, project.as_deref());
            if json {
                // TODO: 輸出 JSON
                eprintln!("(json flag not yet implemented for analyze)");
            }
        }
        Command::Status { json } => {
            commands::status(json);
        }
        Command::Init { project } => {
            commands::init(project.as_deref());
        }
        Command::ListSkills => {
            commands::list_skills();
        }
        Command::Shell => {
            commands::shell();
        }
    }
}