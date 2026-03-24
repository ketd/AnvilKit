#!/usr/bin/env bash
# =============================================================================
# AnvilKit Demo GIF Generator
#
# 运行所有 demo 示例，录制帧序列，通过 ffmpeg 转换为优化的 GIF，
# 并自动复制到对应的文档图片目录。
#
# 依赖:
#   - Rust toolchain (cargo)
#   - ffmpeg (brew install ffmpeg)
#
# 用法:
#   bash tools/capture_demos.sh               # 全量生成
#   bash tools/capture_demos.sh demo_bloom    # 仅生成指定 demo
#   bash tools/capture_demos.sh --list        # 列出所有 demo
#   bash tools/capture_demos.sh --docs-only   # 仅更新文档引用（不重新录制）
#
# 环境变量:
#   CAPTURE_FRAMES=300   # 覆盖默认帧数
#   GIF_FPS=15           # GIF 帧率
#   GIF_WIDTH=640        # GIF 宽度
#   GIF_COLORS=128       # GIF 调色板颜色数
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FRAME_DIR="/tmp/anvilkit_frames"
OUTPUT_DIR="$PROJECT_ROOT/docs/public/images"
GIF_FPS="${GIF_FPS:-15}"
GIF_WIDTH="${GIF_WIDTH:-640}"
GIF_COLORS="${GIF_COLORS:-128}"

# ─── Demo 配置 ───────────────────────────────────────────────────────
# 格式: name:frames
# 每个 demo 对应一个 [[example]] in anvilkit-render Cargo.toml
DEMO_LIST=(
    "demo_showcase:300"
    "demo_pbr:300"
    "demo_shadows:300"
    "demo_bloom:300"
    "demo_ssao:300"
    "demo_dof:300"
    "demo_motion_blur:300"
    "demo_color_grading:300"
    "demo_ibl:300"
    "demo_multi_light:300"
    "demo_particles:300"
    "demo_physics:300"
    "demo_navmesh:300"
    "demo_debug_renderer:300"
    "demo_camera_effects:300"
)

# ─── Demo → 文档页面映射 ─────────────────────────────────────────────
# 一个 demo GIF 可以在多个文档页面使用
declare -A DOC_PAGES=(
    [demo_showcase]="en/rendering/pipeline zh/rendering/pipeline"
    [demo_pbr]="en/rendering/pbr-shadows zh/rendering/pbr-shadows"
    [demo_shadows]="en/rendering/pbr-shadows zh/rendering/pbr-shadows"
    [demo_bloom]="en/rendering/bloom zh/rendering/bloom"
    [demo_ssao]="en/rendering/ssao zh/rendering/ssao"
    [demo_dof]="en/rendering/dof zh/rendering/dof"
    [demo_motion_blur]="en/rendering/motion-blur zh/rendering/motion-blur"
    [demo_color_grading]="en/rendering/color-grading zh/rendering/color-grading"
    [demo_ibl]="en/rendering/pipeline zh/rendering/pipeline"
    [demo_multi_light]="en/rendering/pbr-shadows zh/rendering/pbr-shadows"
    [demo_particles]="en/rendering/pipeline zh/rendering/pipeline"
    [demo_physics]="en/gameplay/physics zh/gameplay/physics"
    [demo_navmesh]="en/gameplay/navigation zh/gameplay/navigation"
    [demo_debug_renderer]="en/devtools/debug-renderer zh/devtools/debug-renderer"
    [demo_camera_effects]="en/core/camera zh/core/camera"
)

# ─── 辅助函数 ────────────────────────────────────────────────────────

check_deps() {
    local missing=0
    if ! command -v ffmpeg &>/dev/null; then
        echo "Error: ffmpeg not found. Install with: brew install ffmpeg"
        missing=1
    fi
    if ! command -v cargo &>/dev/null; then
        echo "Error: cargo not found. Install Rust toolchain."
        missing=1
    fi
    [ "$missing" -eq 0 ] || exit 1
}

list_demos() {
    echo "Available demos:"
    echo ""
    printf "  %-25s %s   %s\n" "NAME" "FRAMES" "DOC PAGES"
    printf "  %-25s %s   %s\n" "----" "------" "---------"
    for entry in "${DEMO_LIST[@]}"; do
        local name="${entry%%:*}"
        local frames="${entry##*:}"
        local pages="${DOC_PAGES[$name]:-none}"
        printf "  %-25s %s   %s\n" "$name" "$frames" "$pages"
    done
}

get_demo_frames() {
    local target="$1"
    for entry in "${DEMO_LIST[@]}"; do
        local name="${entry%%:*}"
        local frames="${entry##*:}"
        if [ "$name" = "$target" ]; then
            echo "${CAPTURE_FRAMES:-$frames}"
            return 0
        fi
    done
    echo "0"
    return 1
}

demo_exists() {
    local target="$1"
    for entry in "${DEMO_LIST[@]}"; do
        local name="${entry%%:*}"
        [ "$name" = "$target" ] && return 0
    done
    return 1
}

# ─── 核心函数 ────────────────────────────────────────────────────────

build_demos() {
    echo "==> Building all demos..."
    cd "$PROJECT_ROOT"
    cargo build -p anvilkit-render --features capture --examples 2>&1 | tail -3
    echo ""
}

run_demo() {
    local name="$1"
    local frames
    frames=$(get_demo_frames "$name")
    local frame_dir="$FRAME_DIR/$name"
    local output="$OUTPUT_DIR/$name.gif"

    echo "==> [$name] Capturing $frames frames..."

    rm -rf "$frame_dir"
    mkdir -p "$frame_dir"

    cd "$PROJECT_ROOT"
    cargo run -p anvilkit-render --features capture --example "$name" -- \
        --capture-dir "$frame_dir" \
        --capture-frames "$frames" \
        2>&1 | grep -E "initialized|complete|error" || true

    # 检查帧文件
    local frame_count
    frame_count=$(find "$frame_dir" -name "frame_*.png" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$frame_count" -eq 0 ]; then
        echo "    ✗ No frames captured, skipping"
        return 1
    fi
    echo "    ✓ $frame_count frames captured"

    # ffmpeg: 帧序列 → 优化 GIF (palette + lanczos)
    mkdir -p "$OUTPUT_DIR"
    ffmpeg -y -framerate 30 \
        -i "$frame_dir/frame_%04d.png" \
        -vf "fps=$GIF_FPS,scale=$GIF_WIDTH:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=$GIF_COLORS[p];[s1][p]paletteuse" \
        "$output" \
        2>/dev/null

    local size
    size=$(du -h "$output" | cut -f1)
    echo "    ✓ $output ($size)"
}

run_all_demos() {
    build_demos
    local total=0
    local success=0
    for entry in "${DEMO_LIST[@]}"; do
        local name="${entry%%:*}"
        total=$((total + 1))
        if run_demo "$name"; then
            success=$((success + 1))
        fi
        echo ""
    done
    echo "═══ Results: $success/$total demos captured ═══"
}

print_summary() {
    echo ""
    echo "═══ Generated GIFs ═══"
    if ls "$OUTPUT_DIR"/*.gif &>/dev/null; then
        printf "  %-30s %s\n" "FILE" "SIZE"
        printf "  %-30s %s\n" "----" "----"
        for f in "$OUTPUT_DIR"/*.gif; do
            local name
            name=$(basename "$f")
            local size
            size=$(du -h "$f" | cut -f1)
            printf "  %-30s %s\n" "$name" "$size"
        done
    else
        echo "  (no GIFs found)"
    fi
    echo ""
    echo "Output directory: $OUTPUT_DIR"
    echo "Docs site: cd docs && pnpm dev"
}

# ─── Main ────────────────────────────────────────────────────────────

check_deps

case "${1:-all}" in
    --list|-l)
        list_demos
        ;;
    --help|-h)
        echo "Usage: $0 [demo_name...] [--list] [--help]"
        echo ""
        echo "  (no args)        Run all demos and generate GIFs"
        echo "  demo_name        Run specific demo(s)"
        echo "  --list, -l       List available demos"
        echo "  --help, -h       Show this help"
        ;;
    all)
        run_all_demos
        print_summary
        ;;
    *)
        build_demos
        for target in "$@"; do
            if ! demo_exists "$target"; then
                echo "Unknown demo: $target"
                echo ""
                list_demos
                exit 1
            fi
            run_demo "$target"
            echo ""
        done
        print_summary
        ;;
esac
