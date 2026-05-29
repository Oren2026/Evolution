# 版本控制規劃（Version Control Plan）

## 版本號命名規則

采用 **Semantic Versioning (SemVer)**：
- `MAJOR.MINOR.PATCH`（如 `0.1.0`、`1.2.3`）
- 当前版本 `0.1.0` 表示 prototype 阶段

###  version 閾值定義

| MAJOR | MINOR | PATCH | 觸發條件 |
|-------|-------|-------|---------|
| API 不兼容變更 | 新功能向下兼容 | Bug fix 或微小改動 |
| `1.x.x` = 正式版 | `0.x.x` = 原型 | 原型期 |

### 當前版本含義

```
0.1.0
└─ prototype（原型階段）
```

---

## Branch 模型

采用 **GitHub Flow** 的簡化版本（單人開發）：

```
main ──────────────────────── 发布
  │
  └─ feature/xxx ─────────── 本地開發
```

### Branch 類型

| Pattern | 用途 | 範例 |
|---------|------|------|
| `main` | 主干，始終可发布 | `main` |
| `feature/*` | 功能開發 | `feature/planner-cli`, `feature/planner-tests` |
| `fix/*` | Bug 修復 | `fix/decision-logic` |
| `doc/*` | 文件更新 | `doc/changelog` |

### 合併規則

- `feature/*` → `main`：**Squash Merge**（保持主干整潔）
- `fix/*` → `main`：**Merge Commit**（保留歷史）
- Commit message 格式：`[<type>] <描述>`

---

## Commit Message 格式

参考 **Conventional Commits**：

```
<type>(<scope>): <subject>

[optional body]
[optional footer]
```

### Type 列表

| type | 說明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修復 |
| `refactor` | 重構（無功能變更） |
| `test` | 新增測試 |
| `docs` | 文件更新 |
| `chore` | 構建/工具變更 |
| `perf` | 效能優化 |

### 範例

```
feat(planner): add CLI entry point
test(planner): add integration tests for DispatchDecision
fix(decision): fix domain tag matching for frontend
docs(changelog): add v0.1.0 entry
```

---

## CHANGELOG

文件位於 `CHANGELOG.md`，格式采用 **`keep a changelog`** 標準：

```markdown
# Changelog

## [0.1.0] — YYYY-MM-DD

### Added
- `planner` module with stages, decision, manifest
- `planner_cli` binary entry point
- Integration tests for planner and node modules

### Changed
- `lib.rs` now exports `planner` module

### Fixed
- `decision.rs` domain tag matching
```

### 發布時機

1. 每次 `MINOR` 或 `PATCH` 提升前更新 CHANGELOG
2. Tag 格式：`v{version}`（如 `v0.1.0`）
3. Tag 由 `cargo release` 管理

---

## 測試覆蓋率目標

| 模組 | 單元測試 | 整合測試 | 覆盖率目標 |
|------|---------|---------|-----------|
| `planner/stages` | ✓ 已內建 | — | 90% |
| `planner/decision` | ✓ 已內建 | `tests/decision.rs` | 95% |
| `planner/manifest` | ✓ 已內建 | `tests/planner_integration.rs` | 90% |
| `node` | ✓ 已內建 | `tests/node_integration.rs` | 90% |
| `runtime` | ✓ 已內建 | — | 85% |

---

## 測試命令

```bash
# 單元測試（所有模組）
cargo test --lib

# 整合測試
cargo test --test '*'

# 單一模組測試
cargo test -p evolution_os --lib planner

# 單元 + 整合 + 文件檢查
cargo test && cargo fmt -- --check && cargo clippy -- -D warnings
```

---

## 發布流程

```bash
# 1. 確認所有測試通過
cargo test

# 2. 確認代码格式
cargo fmt -- --check

# 3. 更新 CHANGELOG.md（手動）
#    - 對齊 version ，加 fill_date

# 4. Commit + Tag
git add CHANGELOG.md
git commit -m "chore(release): bump to v0.2.0"
git tag -a v0.2.0 -m "v0.2.0: add planner module"

# 5. Push
git push origin main --tags
```

> **注意**：原型阶段不发布到 crates.io，仅用于本地开发记录