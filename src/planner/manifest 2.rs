//! Planner Manifest — 規劃結果輸出格式
//!
//! 最終產出：JSON，包含需求確認、問題分析、派工決策、optimized prompt

use serde::{Deserialize, Serialize};

use super::decision::{ComplexityMetrics, DispatchDecision, WorkMode};
use super::stages::Stage;

/// 問題清單項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionItem {
    /// 問題 ID
    pub id: String,
    /// 問題描述
    pub question: String,
    /// 問題類別
    pub category: QuestionCategory,
    /// 預估影響範圍
    pub impact: String,
}

/// 問題類別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestionCategory {
    /// 需求模糊
    Ambiguous,
    /// 範圍不清
    Scope,
    /// 技術選擇
    Technical,
    /// 優先順序
    Priority,
    /// 假設需要確認
    Assumption,
}

impl std::fmt::Display for QuestionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuestionCategory::Ambiguous => write!(f, "Ambiguous"),
            QuestionCategory::Scope => write!(f, "Scope"),
            QuestionCategory::Technical => write!(f, "Technical"),
            QuestionCategory::Priority => write!(f, "Priority"),
            QuestionCategory::Assumption => write!(f, "Assumption"),
        }
    }
}

/// 需求項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// 需求 ID
    pub id: String,
    /// 需求描述
    pub requirement: String,
    /// 優先度
    pub priority: Priority,
    /// 所屬領域
    pub domain: String,
}

/// 優先度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Must,
    Should,
    Could,
    Wont,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Must => write!(f, "Must"),
            Priority::Should => write!(f, "Should"),
            Priority::Could => write!(f, "Could"),
            Priority::Wont => write!(f, "Wont"),
        }
    }
}

/// 預估節點結構（Fork 模式時使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimatedNode {
    /// 節點 ID
    pub id: String,
    /// 節點角色/職責
    pub role: String,
    /// 預計處理的子問題
    pub handles: Vec<String>,
    /// 依賴的其他節點
    pub depends_on: Vec<String>,
}

/// Optimized Prompt 區塊
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimizedPrompt {
    /// 系統提示詞
    pub system: String,
    /// 使用者提示詞
    pub user: String,
    /// 提示詞生成的依據
    pub rationale: String,
}

/// Planner Manifest — 完整規劃結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// 版本
    pub version: String,
    /// 原始任務描述
    pub task: String,
    /// 創建時間（ISO 8601）
    pub created_at: String,
    /// 當前階段
    pub stage: Stage,

    /// ===== Stage 1: 確認需求 =====
    /// 確認的需求項目
    pub requirements: Vec<Requirement>,
    /// 需要確認的問題
    pub questions: Vec<QuestionItem>,
    /// 是否已收斂（所有問題已確認）
    pub converged: bool,

    /// ===== Stage 2: 分析問題 =====
    /// 複雜度指標
    pub complexity: ComplexityMetrics,
    /// 預估節點結構（Fork 模式時）
    pub estimated_nodes: Vec<EstimatedNode>,

    /// ===== Stage 3: 派工決策 =====
    /// 分工模式
    pub work_mode: WorkMode,
    /// 派工決策詳情
    pub dispatch: DispatchDecision,
    /// 最終優化的 Prompt
    pub optimized_prompt: OptimizedPrompt,
}

impl Manifest {
    /// 從任務描述產生完整的 Manifest（純規則版本）
    pub fn from_task(task: &str) -> Self {
        Self::from_task_inner(task, false, None, "llama3")
    }

    /// 從任務描述產生完整的 Manifest（LLM 加持版本）
    ///
    /// 使用 llama3 分析任務複雜度，比純規則更精準。
    /// 若 LLM 不可用，自動降級回純規則。
    #[cfg(feature = "llm")]
    pub fn from_task_with_llm(task: &str, backend: &dyn crate::model::ModelDispatcher) -> Self {
        Self::from_task_inner(task, true, Some(backend), "llama3")
    }

    fn from_task_inner(task: &str, use_llm: bool, backend: Option<&dyn crate::model::ModelDispatcher>, model: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        // Stage 1: 確認需求 — 簡化版本，直接從 task 推斷
        let requirements = Self::extract_requirements(task);
        let questions = Self::generate_questions(task);
        let converged = questions.is_empty();

        // Stage 2: 分析問題
        #[cfg(feature = "llm")]
        let complexity = if use_llm {
            if let Some(b) = backend {
                // 嘗試 LLM，若失敗則降級規則
                crate::planner::ComplexityMetrics::estimate_with_llm(task, b, model)
                    .unwrap_or_else(|| crate::planner::ComplexityMetrics::estimate_from_task(task))
            } else {
                crate::planner::ComplexityMetrics::estimate_from_task(task)
            }
        } else {
            crate::planner::ComplexityMetrics::estimate_from_task(task)
        };

        #[cfg(not(feature = "llm"))]
        let complexity = crate::planner::ComplexityMetrics::estimate_from_task(task);

        // Stage 3: 派工決策
        #[cfg(feature = "llm")]
        let dispatch = if use_llm {
            DispatchDecision::from_metrics(&complexity, Self::extract_domain_tags_llm(task, backend, model))
        } else {
            DispatchDecision::from_task(task)
        };

        #[cfg(not(feature = "llm"))]
        let dispatch = DispatchDecision::from_task(task);
        let estimated_nodes = if dispatch.mode == WorkMode::Fork {
            Self::generate_estimated_nodes(&complexity, task)
        } else {
            vec![]
        };

        let optimized_prompt = Self::build_optimized_prompt(
            task,
            &requirements,
            &dispatch,
        );

        Self {
            version: "0.1.0".to_string(),
            task: task.to_string(),
            created_at: now,
            stage: if converged { Stage::Complete } else { Stage::Confirming },
            requirements,
            questions,
            converged,
            complexity,
            estimated_nodes,
            work_mode: dispatch.mode,
            dispatch,
            optimized_prompt,
        }
    }

    fn extract_requirements(task: &str) -> Vec<Requirement> {
        // 簡化：從任務描述中推斷需求
        let mut reqs = Vec::new();
        let task_lower = task.to_lowercase();

        // 前端需求
        if task_lower.contains("網頁") || task_lower.contains("前端") || task_lower.contains("ui") {
            reqs.push(Requirement {
                id: "req-1".to_string(),
                requirement: "前端介面".to_string(),
                priority: Priority::Must,
                domain: "frontend".to_string(),
            });
        }

        // 後端需求
        if task_lower.contains("後端") || task_lower.contains("api") || task_lower.contains("server") {
            reqs.push(Requirement {
                id: "req-2".to_string(),
                requirement: "後端服務".to_string(),
                priority: Priority::Must,
                domain: "backend".to_string(),
            });
        }

        // 資料庫需求
        if task_lower.contains("資料庫") || task_lower.contains("db") || task_lower.contains("sql") {
            reqs.push(Requirement {
                id: "req-3".to_string(),
                requirement: "資料儲存".to_string(),
                priority: Priority::Must,
                domain: "database".to_string(),
            });
        }

        // 認證需求
        if task_lower.contains("登入") || task_lower.contains("auth") || task_lower.contains("認證") {
            reqs.push(Requirement {
                id: "req-4".to_string(),
                requirement: "使用者認證".to_string(),
                priority: Priority::Should,
                domain: "auth".to_string(),
            });
        }

        // 如果沒識別到任何需求，給一個通用需求
        if reqs.is_empty() {
            reqs.push(Requirement {
                id: "req-1".to_string(),
                requirement: "基本功能建置".to_string(),
                priority: Priority::Must,
                domain: "general".to_string(),
            });
        }

        reqs
    }

    fn generate_questions(task: &str) -> Vec<QuestionItem> {
        let mut questions = Vec::new();
        let task_lower = task.to_lowercase();
        let mut qid = 1;

        // 檢查常見模糊點
        if !task_lower.contains("技術") && !task_lower.contains("stack") && !task_lower.contains("框架") {
            questions.push(QuestionItem {
                id: format!("q-{:03}", qid),
                question: "技術栈偏好？（例如：React + Node.js + PostgreSQL）".to_string(),
                category: QuestionCategory::Technical,
                impact: "影響架構設計和實作方式".to_string(),
            });
            qid += 1;
        }

        if !task_lower.contains("部署") && !task_lower.contains("host") {
            questions.push(QuestionItem {
                id: format!("q-{:03}", qid),
                question: "部署環境偏好？（例如：本地、云端、Serverless）".to_string(),
                category: QuestionCategory::Technical,
                impact: "影響系統架構和成本".to_string(),
            });
            qid += 1;
        }

        if task_lower.contains("管理") && !task_lower.contains("权限") && !task_lower.contains("role") {
            questions.push(QuestionItem {
                id: format!("q-{:03}", qid),
                question: "需要角色權限管理嗎？（例如：管理者 vs 一般使用者）".to_string(),
                category: QuestionCategory::Scope,
                impact: "影響認證和資料模型設計".to_string(),
            });
        }

        questions
    }

    fn generate_estimated_nodes(complexity: &ComplexityMetrics, task: &str) -> Vec<EstimatedNode> {
        let mut nodes = Vec::new();
        let task_lower = task.to_lowercase();

        // 根據領域產生對應節點
        if task_lower.contains("前端") || task_lower.contains("網頁") || task_lower.contains("ui") {
            nodes.push(EstimatedNode {
                id: "node-frontend".to_string(),
                role: "前端工程師".to_string(),
                handles: vec!["介面設計".to_string(), "使用者體驗".to_string(), "頁面渲染".to_string()],
                depends_on: vec![],
            });
        }

        if task_lower.contains("後端") || task_lower.contains("api") {
            nodes.push(EstimatedNode {
                id: "node-backend".to_string(),
                role: "後端工程師".to_string(),
                handles: vec!["API 設計".to_string(), "商業邏輯".to_string(), "資料處理".to_string()],
                depends_on: if nodes.iter().any(|n| n.id == "node-frontend") {
                    vec!["node-frontend".to_string()]
                } else {
                    vec![]
                },
            });
        }

        if task_lower.contains("資料庫") || task_lower.contains("db") || task_lower.contains("sql") {
            nodes.push(EstimatedNode {
                id: "node-database".to_string(),
                role: "資料庫工程師".to_string(),
                handles: vec!["資料模型".to_string(), "查詢優化".to_string(), "資料遷移".to_string()],
                depends_on: vec![],
            });
        }

        if task_lower.contains("登入") || task_lower.contains("auth") {
            nodes.push(EstimatedNode {
                id: "node-auth".to_string(),
                role: "安全工程師".to_string(),
                handles: vec!["認證機制".to_string(), "權限管理".to_string(), "Session 管理".to_string()],
                depends_on: vec!["node-backend".to_string()],
            });
        }

        // 最少 2 個節點
        if nodes.len() < 2 {
            nodes.push(EstimatedNode {
                id: "node-general".to_string(),
                role: "全端工程師".to_string(),
                handles: vec!["通用功能".to_string()],
                depends_on: vec![],
            });
        }

        nodes
    }

    fn build_optimized_prompt(
        task: &str,
        requirements: &[Requirement],
        dispatch: &DispatchDecision,
    ) -> OptimizedPrompt {
        // 根據分工模式建構不同的 prompt
        let system_base = match dispatch.mode {
            WorkMode::Solo => {
                "你是一個專業的軟體工程師。請根據以下需求，獨立地完成任務。"
            }
            WorkMode::Fork => {
                "你是一個專業的軟體工程師，擅長在分工環境中與其他 agent 協作。請注意你的角色和職責範圍。"
            }
        };

        let mut req_text = String::new();
        for (i, req) in requirements.iter().enumerate() {
            req_text.push_str(&format!("{}. {}\n", i + 1, req.requirement));
        }

        let user = format!(
            "任務：{}\n\n需求清單：\n{}\n\n分工模式：{}\n預估節點數：{}\n\n請開始處理。",
            task,
            req_text,
            match dispatch.mode {
                WorkMode::Solo => "獨立（Solo）",
                WorkMode::Fork => "分工（Fork）",
            },
            dispatch.estimated_nodes
        );

        OptimizedPrompt {
            system: system_base.to_string(),
            user,
            rationale: dispatch.rationale.clone(),
        }
    }

    /// 使用 LLM 從任務描述提取領域標籤
    #[cfg(feature = "llm")]
    fn extract_domain_tags_llm(
        task: &str,
        backend: Option<&dyn crate::model::ModelDispatcher>,
        model: &str,
    ) -> Vec<String> {
        let prompt = format!(
            r#"You are a domain classifier. Given this task, list the relevant domains as a JSON array of strings.
Task: "{}"
Output ONLY a JSON array like ["frontend", "backend", "database"], nothing else. If no specific domain, return []. Domains: frontend, backend, database, auth, devops, security, performance, testing."#,
            task
        );

        let req = crate::model::ModelRequest::new(model, &prompt)
            .with_temperature(0.1)
            .with_max_tokens(100);

        let resp = match backend.and_then(|b| b.dispatch(req).ok()) {
            Some(r) => r.content,
            None => return vec![],
        };

        // 從回應中提取 JSON 陣列
        if let Ok(re) = regex::Regex::new(r"\[.*\]") {
            if let Some(caps) = re.find(&resp) {
                if let Ok(tags) = serde_json::from_str::<Vec<String>>(caps.as_str()) {
                    return tags;
                }
            }
        }
        vec![]
    }

    /// 輸出為 JSON 字串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_task() {
        let manifest = Manifest::from_task("幫我建一個計數器");
        assert!(!manifest.task.is_empty());
        assert!(!manifest.dispatch.rationale.is_empty());
        println!("{}", manifest.to_json().unwrap());
    }

    #[test]
    fn test_complex_task() {
        let manifest = Manifest::from_task("幫我建一個庫存管理系統，要有前端、後端、資料庫、登入功能");
        assert_eq!(manifest.work_mode, WorkMode::Fork);
        assert!(!manifest.estimated_nodes.is_empty());
        assert!(manifest.optimized_prompt.system.contains("分工"));
        println!("{}", manifest.to_json().unwrap());
    }

    #[test]
    fn test_questions_generated() {
        let manifest = Manifest::from_task("幫我建一個電商網站");
        // 技術栈問題應該會被產生
        let has_tech_question = manifest
            .questions
            .iter()
            .any(|q| q.category == QuestionCategory::Technical);
        assert!(has_tech_question);
    }
}