# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.16] - 2026-06-09

### Added
- UI 全面重构：暗色精密主题 + 琥珀色强调色
- 双卡模式：多目录扫描，同名 JPG+RAW 自动配对
- 预览用 JPG（快），导出同时包含 JPG 和 RAW
- 预览器空格键选中并自动跳转下一张
- Ctrl+I 反选功能
- 工具栏目录标签和卡片路径提示（多目录模式）
- 缩略图条自动跟随当前预览图滚动

### Fixed
- Windows 下点击面包屑导航报路径不存在
- 搜索框内 Ctrl+A 等快捷键不再被应用拦截
- 取消选择时预览器不再意外跳转或关闭
- 已选视图移除多余的全选/清除按钮
- 图片卡片选中/未选样式修复
- Rust 编译 warning 全部清除

## [0.1.15] - 2026-06-09

### Added
- GitHub 自动更新功能
- 检查更新按钮
- Windows 和 macOS 自动构建发布
- CHANGELOG 支持

### Changed
- 移除状态栏"就绪"标签
- 统一导出对话框和目录树字体颜色为白色

### Fixed
- 使用 Tauri v2 updater 原生发布链路生成和上传 `latest.json`
- 修复检查更新失败时错误信息显示不完整的问题
- 修复 Naive UI 组件在深色主题下的显示问题

## [0.1.1] - 2026-06-07

### Fixed
- 修复自动更新端点 URL，使用正确的 GitHub 仓库地址

