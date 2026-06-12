# AGENTS.md

## Project Overview

Tauri v2 desktop app for filtering/selecting images with AI-powered waste detection.

- **Frontend**: Vue 3 + TypeScript + Vite + Naive UI + Pinia
- **Backend**: Rust with ONNX Runtime for ML inference
- **Build**: `@tauri-apps/cli` v2, builds for Windows and macOS

## Development

### Frontend Only (no Rust)
```bash
npm run dev      # Vite dev server on port 1420
```

### Full App (Frontend + Rust)
```bash
npx tauri dev    # Starts Vite + Rust dev server
```

### Build
```bash
npm run build    # vue-tsc --noEmit && vite build
npx tauri build  # Full Tauri bundle
```

## Architecture

### Frontend (`src/`)
- `src/main.ts` — Entry, wraps app in Naive UI dark theme
- `src/stores/app.ts` — Single Pinia store, all state logic
- `src/components/` — UI components (ImageGrid, ImagePreview, AiPanel, etc.)
- `src/types/index.ts` — Shared TypeScript types

### Backend (`src-tauri/src/`)
- `main.rs` — Tauri builder, registers all commands
- `commands/` — Tauri command modules:
  - `filesystem.rs` — Directory listing
  - `image.rs` — Image scanning, RAW preview, export
  - `updater.rs` — Auto-update via GitHub releases
- `waste_detector.rs` — 统一废片检测入口（CLIP + 图像特征 + 人脸分析）
- `wedding_analyzer.rs` — 婚礼场景专项分析（表情、动作、废片判定）
- `quality_analyzer.rs` — 图像质量特征分析（20维）
- `face_detector.rs` — ONNX-based face detection + 表情分析
- `image_decoder/` — RAW image decoding

### 废片检测流程
```
图像输入 → 图像质量分析 → 人脸检测+表情分析 → 特征融合 → 废片判定
```

### 废片类型
| 类型 | 检测方法 |
|------|----------|
| 表情包/怪表情 | CLIP分类 + 嘴巴张开度 + 眼睛开合度 |
| 闭眼 | 眼睛关键点垂直距离 |
| 惊讶表情 | 嘴巴张开度 > 0.7 |
| 皱眉 | 眼睛眯缝 + 嘴巴紧闭 |
| 头部倾斜 | 眼睛关键点水平偏移 |
| 过曝/欠曝 | 亮度直方图分析 |
| 皮肤过曝 | 皮肤区域曝光比例（婚礼人像专用） |
| 人脸模糊 | 人脸区域 Laplacian 方差 |
| 运动模糊 | 锐度均匀性分析 |
| 重复图 | CLIP embedding 余弦相似度 |
| 截图 | UI元素特征 + 纯色背景 + 文字截图检测 |

### Key Dependencies
- **ONNX Runtime**: Pre-built binaries in `src-tauri/onnxruntime/{win-x64,osx-universal2,linux-x64}`
- **ORT env vars** (set by CI, needed for local Rust builds):
  - `ORT_LIB_LOCATION` → path to platform-specific ONNX Runtime libs
  - `ORT_PREFER_DYNAMIC_LINK=1` (macOS)

## Commands

### Tauri Commands (Rust → Frontend)
Registered in `main.rs`, invoked via `invoke('command_name', args)`:
- `scan_images`, `get_raw_preview`, `export_images`
- `init_waste_detector`, `analyze_waste_images`, `mark_waste_feedback`
- `get_waste_feedback_count`, `get_waste_config`, `update_waste_config`
- `detect_duplicates`, `mark_duplicates_as_waste`
- `check_for_updates`, `install_update`

### File Type Handling
- RAW extensions defined in `src/stores/app.ts`: cr2, cr3, nef, arw, dng, orf, rw2, pef, srw, raf
- RAW files get JPG preview via `get_raw_preview`, but export includes original RAW

## Build & Release

### Local Build Requirements
- Node.js (LTS)
- Rust stable
- ONNX Runtime libs (set `ORT_LIB_LOCATION`)
- macOS: `brew install libpng`

### Release Flow
1. Update version in `package.json` AND `src-tauri/tauri.conf.json` AND `src-tauri/Cargo.toml`
2. Update `CHANGELOG.md`
3. `git tag v<version> && git push origin main --tags`
4. GitHub Actions builds Windows + macOS, publishes to Releases

### GitHub Secrets Required
- `TAURI_SIGNING_PRIVATE_KEY` — contents of `updater.key`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — empty if no password

## Gotchas

- Vite dev server runs on port 1420 (fixed, fails if occupied)
- `src-tauri/` is ignored by Vite file watcher
- `updater.key` and `updater.key.pub` are gitignored — keep secure
- AI models are downloaded at runtime, not bundled
- `package-lock.json` is gitignored — use `npm install` to regenerate
- Windows: ONNX Runtime DLL must be in the binary's directory or system PATH
- Version number must be pure numeric (e.g. `1.0.1`), MSI doesn't support `beta` suffix
