#!/bin/bash
#===============================================================================
# Evolution OS 安裝腳本
#
# 功能：
#   1. 檢查並安裝 Ollama（若未安裝）
#   2. 啟動 Ollama 服務
#   3. 安裝 llama3 模型
#   4. 編譯 Evolution OS（可選）
#
# 用法：
#   ./install.sh          # 完整安裝
#   ./install.sh --ollama-only  # 只安裝 Ollama + llama3
#   ./install.sh --help       # 顯示幫助
#
#===============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OLLAMA_MODEL="llama3"
LOG_FILE="$SCRIPT_DIR/install.log"

# 顏色輸出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok()     { echo -e "${GREEN}[OK]${NC}   $1"; }
log_warn()   { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error()  { echo -e "${RED}[ERROR]${NC} $1"; }

# 寫入日誌
log_to_file() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

#-------------------------------------------------------------------------------
# 檢查作業系統
#-------------------------------------------------------------------------------
check_os() {
    log_info "檢查作業系統..."
    case "$(uname -s)" in
        Darwin*)
            OS="macOS"
            log_ok "偵測到 macOS"
            ;;
        Linux*)
            OS="Linux"
            log_ok "偵測到 Linux"
            ;;
        *)
            log_error "不支援的作業系統：$(uname -s)"
            exit 1
            ;;
    esac
}

#-------------------------------------------------------------------------------
# 檢查並安裝 Ollama
#-------------------------------------------------------------------------------
install_ollama() {
    log_info "檢查 Ollama..."

    if command -v ollama &> /dev/null; then
        log_ok "Ollama 已安裝：$(ollama --version 2>/dev/null || echo 'version unknown')"
        OLLAMA_VERSION=$(ollama --version 2>/dev/null || echo "")
        log_to_file "Ollama already installed: $OLLAMA_VERSION"
    else
        log_warn "Ollama 未安裝，正在安裝..."
        log_to_file "Installing Ollama..."

        if [ "$OS" = "macOS" ]; then
            # 方法 1: Homebrew（優先）
            if command -v brew &> /dev/null; then
                log_info "使用 Homebrew 安裝 Ollama..."
                brew install ollama
            else
                # 方法 2: 直接下載
                log_info "使用官方安裝腳本安裝 Ollama..."
                curl -fsSL https://ollama.com/install.sh | sh
            fi
        elif [ "$OS" = "Linux" ]; then
            log_info "使用官方安裝腳本安裝 Ollama..."
            curl -fsSL https://ollama.com/install.sh | sh
        fi

        if command -v ollama &> /dev/null; then
            log_ok "Ollama 安裝成功"
            log_to_file "Ollama installed successfully"
        else
            log_error "Ollama 安裝失敗，請手動安裝：https://ollama.com/download"
            exit 1
        fi
    fi
}

#-------------------------------------------------------------------------------
# 啟動 Ollama 服務
#-------------------------------------------------------------------------------
start_ollama() {
    log_info "檢查 Ollama 服務狀態..."

    # macOS: 使用 launchctl 檢查
    if [ "$OS" = "macOS" ]; then
        # 嘗試直接啟動 ollama serve
        if ! curl -s http://localhost:11434/api/tags &> /dev/null; then
            log_info "啟動 Ollama 服務..."
            log_to_file "Starting Ollama serve..."

            # 在背景啟動（不綁定終端）
            (ollama serve &) > /dev/null 2>&1 &

            # 等待服務就緒
            WAIT_COUNT=0
            while ! curl -s http://localhost:11434/api/tags &> /dev/null; do
                sleep 1
                WAIT_COUNT=$((WAIT_COUNT + 1))
                if [ $WAIT_COUNT -ge 30 ]; then
                    log_error "Ollama 服務啟動逾時（30秒）"
                    log_to_file "Ollama serve timeout"
                    exit 1
                fi
            done
            log_ok "Ollama 服務已啟動"
            log_to_file "Ollama serve started"
        else
            log_ok "Ollama 服務已在運行"
            log_to_file "Ollama already running"
        fi
    else
        # Linux: systemd 或直接啟動
        if systemctl is-active --quiet ollama 2>/dev/null; then
            log_ok "Ollama 服務運行中（systemd）"
        elif ! curl -s http://localhost:11434/api/tags &> /dev/null; then
            log_info "啟動 Ollama 服務..."
            (ollama serve &) > /dev/null 2>&1 &
            sleep 3
            log_ok "Ollama 服務已啟動"
        else
            log_ok "Ollama 服務已在運行"
        fi
    fi
}

#-------------------------------------------------------------------------------
# 安裝 llama3 模型
#-------------------------------------------------------------------------------
install_llama3() {
    log_info "檢查 $OLLAMA_MODEL 模型..."

    # 檢查模型是否已存在
    if ollama list 2>/dev/null | grep -q "$OLLAMA_MODEL"; then
        log_ok "模型 $OLLAMA_MODEL 已存在"
        log_to_file "Model $OLLAMA_MODEL already exists"
    else
        log_info "正在下載模型 $OLLAMA_MODEL（約 4.6GB，首次可能需要幾分鐘）..."
        log_to_file "Pulling model $OLLAMA_MODEL..."

        if ollama pull "$OLLAMA_MODEL"; then
            log_ok "模型 $OLLAMA_MODEL 安裝成功"
            log_to_file "Model $OLLAMA_MODEL pulled successfully"
        else
            log_error "模型 $OLLAMA_MODEL 安裝失敗"
            log_to_file "Model $OLLAMA_MODEL pull failed"
            exit 1
        fi
    fi

    # 驗證模型可正常呼叫
    log_info "驗證模型..."
    if echo "test" | ollama run "$OLLAMA_MODEL" > /dev/null 2>&1; then
        log_ok "模型 $OLLAMA_MODEL 驗證成功"
        log_to_file "Model $OLLAMA_MODEL verified"
    else
        log_warn "模型驗證有警告，但繼續安裝..."
    fi
}

#-------------------------------------------------------------------------------
# 編譯 Evolution OS
#-------------------------------------------------------------------------------
build_evolution() {
    log_info "編譯 Evolution OS（LLM 功能）..."
    log_to_file "Building Evolution OS with LLM feature..."

    cd "$SCRIPT_DIR"

    if cargo build --features llm 2>&1 | tee -a "$LOG_FILE"; then
        log_ok "Evolution OS 編譯成功"
        log_to_file "Evolution OS build successful"
    else
        log_error "Evolution OS 編譯失敗"
        log_to_file "Evolution OS build failed"
        exit 1
    fi
}

#-------------------------------------------------------------------------------
# 顯示幫助
#-------------------------------------------------------------------------------
show_help() {
    echo "Evolution OS 安裝腳本"
    echo ""
    echo "用法: $0 [選項]"
    echo ""
    echo "選項："
    echo "  --ollama-only   只安裝 Ollama + llama3，不編譯 Evolution OS"
    echo "  --build-only    只編譯 Evolution OS（假設 Ollama 已安裝）"
    echo "  --help          顯示此幫助訊息"
    echo ""
    echo "不加參數：完整安裝（Ollama + llama3 + Evolution OS）"
}

#-------------------------------------------------------------------------------
# 主流程
#-------------------------------------------------------------------------------
main() {
    echo "============================================"
    echo "  Evolution OS 安裝腳本"
    echo "============================================"
    echo ""

    # 解析參數
    MODE="full"
    case "${1:-}" in
        --ollama-only)
            MODE="ollama-only"
            ;;
        --build-only)
            MODE="build-only"
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        "")
            MODE="full"
            ;;
        *)
            log_error "未知參數：$1"
            show_help
            exit 1
            ;;
    esac

    # 初始化日誌
    echo "# Evolution OS 安裝日誌 - $(date)" > "$LOG_FILE"

    case "$MODE" in
        ollama-only)
            check_os
            install_ollama
            start_ollama
            install_llama3
            ;;
        build-only)
            build_evolution
            ;;
        full)
            check_os
            install_ollama
            start_ollama
            install_llama3
            build_evolution
            ;;
    esac

    echo ""
    echo "============================================"
    log_ok "安裝完成！"
    echo "============================================"
    echo ""
    echo "使用方式："
    echo "  # 規則模式（不需要 LLM）"
    echo "  cargo run --bin planner -- \"幫我建一個庫存管理系統\""
    echo ""
    echo "  # LLM 模式（使用 llama3）"
    echo "  cargo run --bin planner --features llm -- --llm -- \"幫我建一個庫存管理系統\""
    echo ""
}

main "$@"
