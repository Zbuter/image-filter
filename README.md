# Vue 3 + TypeScript + Vite

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

Learn more about the recommended Project Setup and IDE Support in the [Vue Docs TypeScript Guide](https://vuejs.org/guide/typescript/overview.html#project-setup).

## 部署到 GitHub

### 1. 创建 GitHub 仓库

```bash
# 登录 GitHub CLI（如果还未登录）
gh auth login

# 创建新的公开仓库
gh repo create image-filter --public --source=. --remote=origin --push
```

或者手动在 GitHub 网站上创建仓库，然后：

```bash
git remote add origin https://github.com/YOUR_USERNAME/image-filter.git
git push -u origin main
```

### 2. 配置 GitHub Secrets

在 GitHub 仓库的 Settings > Secrets and variables > Actions 中添加以下 secrets：

- `TAURI_SIGNING_PRIVATE_KEY`: 将 `updater.key` 文件的内容复制到这里
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: 如果生成密钥时设置了密码，填写密码；否则留空

### 3. 发布新版本

```bash
# 更新版本号（在 package.json 和 src-tauri/tauri.conf.json 中）
# 例如从 0.1.0 改为 0.2.0

# 更新 CHANGELOG.md，添加新版本的变更内容

# 提交更改
git add .
git commit -m "Release v0.2.0"

# 创建标签并推送
git tag v0.2.0
git push origin main --tags
```

推送标签后，GitHub Actions 会自动构建 Windows 和 macOS 版本并发布到 Releases。

### 4. 更新 CHANGELOG

每次发布前，运行以下命令查看自上次发布以来的提交：

```bash
./scripts/generate-changelog.sh
```

然后将相关变更添加到 `CHANGELOG.md` 的新版本部分。
