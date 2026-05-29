//! shell — 基本互動式 REPL

use std::io::{self, Write};

const PROMPT: &str = "evolution$ ";

pub fn shell() {
    println!();
    println!("╔══════════════════════════════════════════════╗");
    println!("║  Evolution OS Shell — 輸入 'help' 查看指令  ║");
    println!("║  輸入 'exit' 離開                            ║");
    println!("╚══════════════════════════════════════════════╝");
    println!();

    let mut input = String::new();

    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        input.clear();
        let bytes = io::stdin().read_line(&mut input).unwrap();
        if bytes == 0 {
            // EOF (Ctrl+D)
            println!("\nexit");
            break;
        }

        let cmd = input.trim();
        if cmd.is_empty() {
            continue;
        }

        match cmd {
            "exit" | "quit" | "q" => {
                println!("再見！");
                break;
            }
            "help" | "h" | "?" => {
                print_help();
            }
            s if s.starts_with("analyze ") => {
                let task = &s[8..];
                let _ = s;
                println!("執行分析中...");
                crate::commands::analyze::analyze(task, None);
            }
            s if s.starts_with("new ") => {
                let name = &s[4..];
                let _ = s;
                crate::commands::new_project::new_project(name);
            }
            "skills" | "list-skills" | "list" => {
                crate::commands::list_skills::list_skills();
            }
            s if s.starts_with("cd ") => {
                println!("(cd 尚未實作)");
            }
            s if s.starts_with("ls") => {
                // 列出專案
                let root = crate::commands::new_project::evolution_root()
                    .join("projects");
                if !root.exists() {
                    println!("尚無專案");
                    continue;
                }
                match std::fs::read_dir(&root) {
                    Ok(entries) => {
                        let count = entries.count();
                        if count == 0 {
                            println!("尚無專案");
                        } else {
                            println!("專案列表:");
                            for entry in std::fs::read_dir(&root).unwrap() {
                                if let Ok(e) = entry {
                                    println!("  {}", e.file_name().to_string_lossy());
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("讀取失敗: {}", e),
                }
            }
            _ => {
                eprintln!("未知指令: '{}'", cmd);
                println!("輸入 'help' 查看可用指令");
            }
        }
    }
}

fn print_help() {
    println!();
    println!(" Evolution OS Shell — 指令");
    println!("═{}", "═".repeat(29));
    println!();
    println!("  new <name>         建立新專案");
    println!("  analyze <task>     分析任務");
    println!("  skills / list      列出可用技能");
    println!("  ls                 列出專案");
    println!("  help               顯示說明");
    println!("  exit               離開");
    println!();
    println!("═{}", "═".repeat(29));
}