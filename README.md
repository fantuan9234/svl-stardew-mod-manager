# SVL - Stardew Valley Mod Manager svl星露谷物语模组管理器

<div align="center">

![SVL Banner](https://img.shields.io/badge/SVL-Stardew%20Valley%20Launcher-green)
![Version](https://img.shields.io/badge/version-1.0.2-blue)
![License](https://img.shields.io/badge/license-MIT-green)

**专为《星露谷物语》打造的现代化 MOD 管理器**

基于 Tauri + React + Rust 构建，极速、稳定、美观

</div>

---

## ✨ 特性一览

- 📦 **MOD 管理** - 智能扫描、一键启用/禁用、批量操作、分组显示
- 🔍 **冲突检测** - 自动识别依赖缺失和兼容性问题，提供解决方案
- 🎮 **SMAPI 集成** - 自动检测游戏路径和 SMAPI 状态，一键启动
- 👤 **配置文件系统** - 多配置方案切换、存档绑定、导入导出
- 🌐 **Nexus Mods 对接** - 版本检查、快速搜索、更新提醒
- 📥 **拖拽安装** - 支持 .zip/.7z 压缩包和文件夹安装
- 📊 **日志分析** - 智能解读 SMAPI 日志，提供白话解决方案
- 🎨 **现代化 UI** - 基于 Ant Design，支持深色模式和多语言（中文/英文）

---

## 🚀 快速开始

### 安装

1. 从 [Releases](https://github.com/your-username/svl/releases) 下载对应系统的安装包
2. 运行安装程序完成安装
3. 启动 SVL

### 首次使用

SVL 会引导你完成以下步骤：

1. **检测游戏路径** - 自动检测或通过 Steam/GOG 库定位
2. **安装 SMAPI** - 提供 SMAPI 官方下载链接和安装指引
3. **开始使用** - 一切就绪，开始安装和管理 MOD！

---

## 📖 功能详解

### MOD 管理

- **智能扫描**：自动扫描 `Mods` 文件夹，识别所有已安装的 MOD
- **启用/禁用**：通过重命名文件夹前缀快速开关 MOD，无需删除
- **分组显示**：大型 MOD（如 SVE）自动聚合，避免列表膨胀
- **批量操作**：支持批量更新、批量删除

### 冲突检测与依赖检查

- **智能冲突检测**：识别必选依赖缺失、可选依赖缺失、ContentPack 冲突、已知不兼容
- **前置依赖检查**：递归解析 `Dependencies` 和 `ContentPacks` 字段
- **隐式依赖支持**：自动为 Content Patcher 内容包添加隐式依赖
- **解决方案推荐**：每个问题项附带"在 Nexus 搜索"按钮

### 配置文件系统

- **多配置管理**：创建不同的 MOD 配置方案，适应不同游戏场景
- **存档绑定**：将配置文件与特定存档关联，自动切换 MOD 组合
- **导入/导出**：支持配置文件的备份和分享
- **快速迁移**：在不同配置之间复制 MOD 状态

### Nexus Mods 集成

- **API 对接**：集成 Nexus Mods API，检查 MOD 版本更新
- **MOD ID 映射**：内置 MOD 字典，自动关联本地 MOD 与 N 网条目
- **一键搜索**：无法识别的 MOD 自动使用名称在 N 网搜索

### 日志分析器

- **智能解读**：自动解析 SMAPI 启动日志，识别常见错误
- **白话解决方案**：通俗易懂地解释技术问题
- **快速定位**：提供错误 MOD 的 Nexus 搜索链接

---

## 🛠 技术栈

- **前端**：React 19 + TypeScript + Ant Design + Vite
- **后端**：Rust + Tauri 2.x
- **国际化**：i18next 多语言支持
- **测试**：Vitest 前端测试 + Playwright E2E 测试

---

## 📦 开发指南

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- pnpm >= 8

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建发布版

```bash
pnpm tauri build
```

### 运行测试

```bash
# 前端测试
pnpm test:frontend

# 回归测试（需要 Playwright）
python tests/svl_regression_test.py
```

---

## 📝 更新日志

详见 [CHANGELOG.md](CHANGELOG.md)

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

## 📄 许可证

MIT License

---

<div align="center">

**SVL 让 MOD 管理变得简单、高效、愉悦。**

Made with ❤️ by the SVL Team

</div>
