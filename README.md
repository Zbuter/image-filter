# Image Filter

婚礼/领证照片筛选工具，AI 自动识别废片。

## 功能

- **废片检测**：自动识别表情包、闭眼、过曝、模糊、截图等废片
- **人脸分析**：检测闭眼、嘴巴张开、皱眉、头部倾斜等表情
- **皮肤曝光**：婚礼人像专用，检测皮肤区域过曝
- **重复检测**：基于 CLIP embedding 余弦相似度
- **自学习**：用户标记后自动训练分类器，越用越准
- **RAW 支持**：CR2/CR3/NEF/ARW/DNG 等格式预览和导出
- **双卡模式**：多目录扫描，同名 JPG+RAW 自动配对

## 安装

### 下载

从 [Releases](https://github.com/Zbuter/image-filter/releases) 下载最新版本。

### 从源码构建

```bash
# 安装依赖
npm install

# 开发模式
npx tauri dev

# 构建
npx tauri build
```

## 快捷键

| 键 | 功能 |
|---|------|
| D | 标记为废片 |
| F | 标记为非废片 |
| Del | 移除图片 |
| ←/→ | 上/下一张 |
| 空格 | 选中 + 下一张 |
| Ctrl+A | 全选 |
| Ctrl+I | 反选 |

## 技术栈

- **前端**：Vue 3 + TypeScript + Vite + Naive UI + Pinia
- **后端**：Rust + Tauri v2
- **AI**：ONNX Runtime (YuNet 人脸检测) + 图像质量分析

## 开源协议

MIT
