# Tauri → Electron + TypeScript 迁移实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 SVL（Stardew Valley Mod Manager）从 Tauri + Rust 后端迁移到 Electron + TypeScript 全栈架构，保持所有功能不变。

**Architecture:** 前端 React + Ant Design + Vite 保持不变，后端从 Rust Tauri commands 迁移为 Electron Main 进程的 TypeScript 模块。通过 Electron IPC（contextBridge + ipcRenderer/ipcMain）替代 Tauri invoke/listen。Rust 中的核心逻辑（mod 解析、日志分析、Nexus API 等）需要在 TypeScript 中重新实现。

**Tech Stack:** Electron 33+、TypeScript 5.x、React 19、Vite 7、Ant Design 6、electron-builder

---

## 项目现状分析

### 前端文件清单（56 个文件）

#### Tauri 依赖文件（需要适配，共 25 个）

| 文件 | Tauri API 使用 | 适配难度 |
|------|---------------|---------|
| `utils/tauri-api.ts` | `invoke`, `convertFileSrc` (核心 API 层) | 🔴 高 |
| `utils/advanced-features-api.ts` | `invoke`, `listen` | 🔴 高 |
| `utils/openUrl.ts` | `@tauri-apps/plugin-opener` | 🟢 低 |
| `components/AppLayout.tsx` | `invoke`, `listen`, `getCurrentWindow` | 🟡 中 |
| `components/ModInstaller.tsx` | `@tauri-apps/plugin-dialog` | 🟢 低 |
| `components/ModBackupConfirmModal.tsx` | `@tauri-apps/plugin-dialog` | 🟢 低 |
| `components/LogParser.tsx` | `invoke`, `listen`, `plugin-opener` | 🟡 中 |
| `components/ModDetail.tsx` | `invoke`, `plugin-opener` | 🟢 低 |
| `components/ModList.tsx` | `invoke` | 🟢 低 |
| `components/DropZone.tsx` | `plugin-dialog`, `getCurrentWindow` | 🟢 低 |
| `components/ModInstallWizard.tsx` | `plugin-dialog`, `invoke` | 🟢 低 |
| `components/UpdateChecker.tsx` | `plugin-updater`, `listen`, `invoke`, `plugin-process` | 🔴 高 |
| `components/ProfileManager.tsx` | `plugin-dialog` | 🟢 低 |
| `components/SmapiLogViewer.tsx` | `getCurrentWindow`, `listen` | 🟢 低 |
| `components/StatusBar.tsx` | `invoke` | 🟢 低 |
| `pages/ModManager.tsx` | `invoke`, `getCurrentWindow`, `listen`, `plugin-opener`, `plugin-dialog` | 🔴 高 |
| `pages/NexusModBrowser.tsx` | `invoke`, `listen` | 🟡 中 |
| `pages/ProfilesPage.tsx` | `plugin-dialog`, `plugin-fs`, `listen` | 🟡 中 |
| `pages/SyncPage.tsx` | `plugin-dialog`, `getCurrentWindow` | 🟢 低 |
| `pages/Settings.tsx` | `listen` | 🟢 低 |
| `pages/LogViewer.tsx` | `invoke` | 🟢 低 |
| `pages/SavesManager.tsx` | `plugin-dialog` | 🟢 低 |
| `hooks/useImageUrl.ts` | `toAssetUrl` (convertFileSrc) | 🟡 中 |
| `hooks/useNexusStatus.ts` | `invoke` | 🟢 低 |
| `hooks/useModUrl.ts` | `invoke` | 🟢 低 |

#### 纯前端文件（可直接迁移，共 31 个）

| 文件 | 说明 |
|------|------|
| `App.tsx` | 主入口 |
| `App.css` | 全局样式 |
| `main.tsx` | React 入口 |
| `vite-env.d.ts` | 类型声明 |
| `i18n/index.ts` + 3 个 JSON | 国际化 |
| `hooks/useTheme.ts` | 主题管理（纯 localStorage） |
| `hooks/useModTags.ts` | 标签管理（纯 localStorage） |
| `components/HomeModal.tsx` | 首页弹窗 |
| `components/ConfigManager.tsx` | 配置管理 |
| `components/DependencyResolver.tsx` | 依赖解析 UI |
| `components/GameMonitor.tsx` | 游戏监控 UI |
| `components/LoadOrderModal.tsx` | 加载顺序 UI |
| `components/ModBackupManager.tsx` | 备份管理 UI |
| `components/ModCard.tsx` | Mod 卡片 |
| `components/ModConfigEditor.tsx` | 配置编辑 UI |
| `components/NetworkDiagnostic.tsx` | 网络诊断 UI |
| `components/NexusApiConfig.tsx` | API 配置 UI |
| `components/Onboarding.tsx` | 引导页 |
| `components/ProfileSelector.tsx` | 档案选择 UI |
| `components/SecurityScanner.tsx` | 安全扫描 UI |
| `components/SnapshotManager.tsx` | 快照管理 UI |
| `components/StorageAnalyzerView.tsx` | 存储分析 UI |
| `components/SyncManager.tsx` | 同步管理 UI |
| `components/AppLogViewer.tsx` | 日志查看 UI |
| `components/ApiKeyReminder.tsx` | API Key 提醒 |
| `pages/DonatePage.tsx` | 捐赠页 |
| `pages/Toolbox.tsx` | 工具箱页 |
| `pages/OnlineSync.tsx` | 在线同步页 |
| `__tests__/setup.ts` | 测试 setup |
| `__tests__/ProfilesPage.test.tsx` | 测试 |
| `__tests__/i18n.test.ts` | 测试 |

### Rust 后端 Tauri Commands 清单（约 70+ 个命令）

| 模块 | 命令数 | 核心功能 |
|------|--------|---------|
| `mod_parser.rs` | 3 | Mod 扫描、启用/禁用、文件读取 |
| `mod_installer.rs` | 5 | 从压缩包/文件夹安装、卸载、依赖检查 |
| `log_parser.rs` | 8 | SMAPI 日志分析、FTM 错误分析、依赖检查 |
| `nexus_api.rs` | 15+ | Nexus API 交互、下载、NXM 协议 |
| `profiles.rs` | 15 | 档案管理 CRUD |
| `smapi_launcher.rs` | 3 | 游戏启动/停止 |
| `saves_manager.rs` | 8 | 存档管理 |
| `update_checker.rs` | 5 | 更新检查 |
| `mod_backup.rs` | 4 | Mod 备份 |
| `mod_config.rs` | 3 | Mod 配置读写 |
| `mod_ordering.rs` | 2 | 加载顺序 |
| `conflict_checker.rs` | 1 | 冲突检测 |
| `mod_security.rs` | 2 | 安全检查 |
| `storage_analyzer.rs` | 1 | 存储分析 |
| `sync_manager.rs` | 5 | 同步管理 |
| `smapi.rs` | 3 | SMAPI 检测/安装 |
| `app_updater.rs` | 4 | 应用更新 |
| `mod_thumbnail.rs` | 3 | 缩略图 |
| `dep_resolver.rs` | 2 | 依赖解析 |
| `app_logger.rs` | 4 | 应用日志 |
| `mod_dict_updater.rs` | 1 | 字典更新 |
| `nexus_linker.rs` | 1 | Nexus 链接构建 |
| `profile_archive.rs` | 3 | 档案导入导出 |

### Tauri 特性映射到 Electron

| Tauri | Electron 等价 |
|-------|-------------|
| `invoke('cmd', { args })` | `ipcRenderer.invoke('cmd', args)` |
| `listen('event', cb)` | `ipcRenderer.on('event', cb)` |
| `app.emit('event', data)` | `mainWindow.webContents.send('event', data)` |
| `convertFileSrc(path)` | `protocol.registerFileProtocol` 或 `net.fetch` |
| `@tauri-apps/plugin-dialog` | `dialog.showOpenDialog / showSaveDialog` |
| `@tauri-apps/plugin-fs` | `fs.readFile / writeFile` |
| `@tauri-apps/plugin-opener` | `shell.openExternal / openPath` |
| `@tauri-apps/plugin-updater` | `electron-updater` |
| `@tauri-apps/plugin-process` | `app.relaunch()` |
| `getCurrentWindow()` | `BrowserWindow` API |
| `tauri.conf.json` | `electron-builder.yml` |

---

## 文件结构设计

```
stardew-mod-manager/
├── electron/                    # Electron 主进程
│   ├── main.ts                  # 主进程入口
│   ├── preload.ts               # contextBridge 暴露 API
│   ├── ipc/                     # IPC handler 注册
│   │   ├── index.ts             # 统一注册
│   │   ├── mod-parser.ts        # mod 扫描/启用禁用
│   │   ├── mod-installer.ts     # mod 安装/卸载
│   │   ├── log-parser.ts        # 日志解析
│   │   ├── nexus-api.ts         # Nexus API
│   │   ├── profiles.ts          # 档案管理
│   │   ├── smapi.ts             # SMAPI 检测/安装
│   │   ├── saves.ts             # 存档管理
│   │   ├── update-checker.ts    # 更新检查
│   │   ├── mod-backup.ts        # Mod 备份
│   │   ├── mod-config.ts        # Mod 配置
│   │   ├── mod-ordering.ts      # 加载顺序
│   │   ├── conflict-checker.ts  # 冲突检测
│   │   ├── mod-security.ts      # 安全检查
│   │   ├── storage-analyzer.ts  # 存储分析
│   │   ├── sync-manager.ts      # 同步管理
│   │   ├── app-updater.ts       # 应用更新
│   │   ├── mod-thumbnail.ts     # 缩略图
│   │   ├── dep-resolver.ts      # 依赖解析
│   │   ├── app-logger.ts        # 应用日志
│   │   └── nexus-linker.ts      # Nexus 链接
│   ├── core/                    # 核心业务逻辑（从 Rust 移植）
│   │   ├── mod-parser-logic.ts  # manifest 解析、JSON 注释剥离等
│   │   ├── log-parser-logic.ts  # SMAPI 日志规则引擎
│   │   ├── nexus-linker-data.ts # BUILTIN_DICT 等数据
│   │   └── smapi-data.ts        # SMAPI 官方数据
│   └── utils/                   # 主进程工具
│       ├── json-utils.ts        # stripComments, normalizeQuotes 等
│       ├── registry.ts          # Windows 注册表读取
│       └── game-path.ts         # 游戏路径检测
├── src/                         # 前端（React，基本不变）
│   ├── utils/
│   │   ├── electron-api.ts      # 替代 tauri-api.ts
│   │   ├── advanced-features-api.ts  # 适配 Electron IPC
│   │   └── openUrl.ts           # 适配 shell.openExternal
│   ├── hooks/
│   │   ├── useImageUrl.ts       # 适配 Electron file protocol
│   │   └── ...                  # 其余不变
│   └── ...                      # 其余前端文件基本不变
├── electron-builder.yml         # 打包配置
├── package.json                 # 添加 electron 依赖
├── tsconfig.json                # 添加 electron 目录
└── vite.config.ts               # 适配 Electron
```

---

## 迁移任务

### Task 1: Git 初始化与分支准备

**Files:**
- Create: `.gitignore` (更新)
- Modify: 无

- [ ] **Step 1: 初始化 git 仓库**

```bash
cd "d:\stardew mod mannager\stardew-mod-manager"
git init
```

- [ ] **Step 2: 创建 .gitignore**

```gitignore
node_modules/
dist/
target/
*.log
.env
.DS_Store
Thumbs.db
release/
```

- [ ] **Step 3: 提交当前 Tauri 代码到 master**

```bash
git add -A
git commit -m "chore: initial commit - Tauri + Rust codebase"
```

- [ ] **Step 4: 创建并切换到 electron 分支**

```bash
git checkout -b electron
```

- [ ] **Step 5: 验证分支状态**

```bash
git branch
git log --oneline -1
```

Expected: 当前在 `electron` 分支，有 1 个 commit

---

### Task 2: 安装 Electron + TypeScript 依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 Electron 核心依赖**

```bash
cd "d:\stardew mod mannager\stardew-mod-manager"
npm install --save-dev electron@latest electron-builder @electron/rebuild
npm install --save-dev tsx typescript @types/node
npm install --save-dev vite-plugin-electron vite-plugin-electron-renderer
```

- [ ] **Step 2: 安装 Electron 运行时依赖**

```bash
npm install electron-updater
npm install archiver extract-zip fast-xml-parser
```

- [ ] **Step 3: 验证安装**

```bash
npx electron --version
```

Expected: 输出 Electron 版本号

- [ ] **Step 4: 提交**

```bash
git add package.json package-lock.json
git commit -m "chore: add Electron + TypeScript dependencies"
```

---

### Task 3: 创建 Electron 主进程入口

**Files:**
- Create: `electron/main.ts`
- Create: `electron/preload.ts`
- Create: `electron/ipc/index.ts`

- [ ] **Step 1: 创建 `electron/main.ts`**

```typescript
import { app, BrowserWindow, ipcMain, protocol } from 'electron';
import path from 'path';

let mainWindow: BrowserWindow | null = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 960,
    minHeight: 600,
    frame: false,
    titleBarStyle: 'hidden',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  if (process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }
}

app.whenReady().then(() => {
  protocol.registerFileProtocol('svl-file', (request, callback) => {
    const filePath = request.url.replace('svl-file://', '');
    callback({ path: decodeURIComponent(filePath) });
  });

  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

import './ipc/index';
```

- [ ] **Step 2: 创建 `electron/preload.ts`**

```typescript
import { contextBridge, ipcRenderer } from 'electron';

const electronAPI = {
  invoke: (channel: string, ...args: any[]) => ipcRenderer.invoke(channel, ...args),
  on: (channel: string, callback: (...args: any[]) => void) => {
    const subscription = (_event: any, ...args: any[]) => callback(...args);
    ipcRenderer.on(channel, subscription);
    return () => ipcRenderer.removeListener(channel, subscription);
  },
  convertFileSrc: (filePath: string) => `svl-file://${filePath}`,
};

contextBridge.exposeInMainWorld('electronAPI', electronAPI);
```

- [ ] **Step 3: 创建 `electron/ipc/index.ts`**

```typescript
import { ipcMain } from 'electron';

export function registerIpcHandlers() {
  // 各模块的 handler 将在后续 Task 中逐步注册
}

registerIpcHandlers();
```

- [ ] **Step 4: 提交**

```bash
git add electron/
git commit -m "feat: add Electron main process entry, preload, and IPC skeleton"
```

---

### Task 4: 配置 Vite + Electron 开发环境

**Files:**
- Modify: `vite.config.ts`
- Modify: `package.json` (scripts)
- Modify: `tsconfig.json`

- [ ] **Step 1: 修改 `vite.config.ts`**

```typescript
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import electron from 'vite-plugin-electron';
import renderer from 'vite-plugin-electron-renderer';

export default defineConfig({
  plugins: [
    react(),
    electron([
      {
        entry: 'electron/main.ts',
        vite: {
          build: {
            outDir: 'dist-electron',
          },
        },
      },
      {
        entry: 'electron/preload.ts',
        onstart(args) {
          args.reload();
        },
        vite: {
          build: {
            outDir: 'dist-electron',
          },
        },
      },
    ]),
    renderer(),
  ],
  server: {
    port: 1420,
    strictPort: true,
  },
});
```

- [ ] **Step 2: 更新 `package.json` scripts**

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "electron:dev": "vite",
    "electron:build": "npm run build && electron-builder",
    "test:frontend": "vitest run"
  }
}
```

- [ ] **Step 3: 更新 `tsconfig.json` 添加 electron 目录**

在 `include` 中添加 `"electron"`。

- [ ] **Step 4: 验证开发模式启动**

```bash
npm run dev
```

Expected: Vite dev server 启动，Electron 窗口打开

- [ ] **Step 5: 提交**

```bash
git add vite.config.ts package.json tsconfig.json
git commit -m "feat: configure Vite + Electron dev environment"
```

---

### Task 5: 创建前端 Electron API 适配层

**Files:**
- Create: `src/utils/electron-api.ts` (替代 `tauri-api.ts`)
- Create: `src/utils/electron-advanced-api.ts` (替代 `advanced-features-api.ts`)
- Modify: `src/utils/openUrl.ts`

- [ ] **Step 1: 创建 `src/utils/electron-api.ts`**

这是最关键的适配层。将所有 `invoke('cmd', { args })` 替换为 `window.electronAPI.invoke('cmd', args)`。所有接口定义（`ModInfo`、`InstallResult` 等）保持不变。

```typescript
declare global {
  interface Window {
    electronAPI: {
      invoke: (channel: string, ...args: any[]) => Promise<any>;
      on: (channel: string, callback: (...args: any[]) => void) => () => void;
      convertFileSrc: (filePath: string) => string;
    };
  }
}

const api = window.electronAPI;

export function toAssetUrl(filePath: string): Promise<string> {
  return Promise.resolve(api.convertFileSrc(filePath));
}

// 所有 invoke 函数的模板：
// Tauri: invoke<T>('cmd_name', { arg1, arg2 })
// Electron: api.invoke('cmd_name', arg1, arg2)

export async function detectGamePath(): Promise<GamePathInfo> {
  return api.invoke('detect_game_path');
}

export async function scanMods(gamePath?: string): Promise<ModInfo[]> {
  return api.invoke('scan_mods', gamePath || null);
}

// ... 其余所有函数按相同模式迁移
// 完整文件内容在实施时生成
```

- [ ] **Step 2: 创建 `src/utils/electron-advanced-api.ts`**

将 `listen` 替换为 `api.on`：

```typescript
const api = window.electronAPI;

export async function listenToMonitorUpdates(callback: (data: ModMonitorStatus) => void): Promise<() => void> {
  return api.on('mod-monitor-update', callback);
}
```

- [ ] **Step 3: 修改 `src/utils/openUrl.ts`**

```typescript
import { message } from 'antd';
import i18n from '../i18n';

export async function openUrl(url: string, fallbackMessage?: string): Promise<void> {
  if (!url) {
    message.error(fallbackMessage || i18n.t('app.urlEmpty'));
    return;
  }
  let normalizedUrl = url.trim();
  if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
    normalizedUrl = 'https://' + normalizedUrl;
  }
  try {
    await window.electronAPI.invoke('open-external-url', normalizedUrl);
  } catch (error) {
    console.error('Failed to open URL:', normalizedUrl, error);
    message.error(fallbackMessage || i18n.t('app.openUrlFailed'));
  }
}
```

- [ ] **Step 4: 提交**

```bash
git add src/utils/electron-api.ts src/utils/electron-advanced-api.ts src/utils/openUrl.ts
git commit -m "feat: create Electron API adapter layer replacing Tauri APIs"
```

---

### Task 6: 迁移核心业务逻辑 — Mod 解析器

**Files:**
- Create: `electron/core/mod-parser-logic.ts`
- Create: `electron/core/json-utils.ts`
- Create: `electron/ipc/mod-parser.ts`

这是最复杂的迁移任务之一。Rust 中的 `mod_parser.rs` 包含约 1300 行代码，需要将以下逻辑移植为 TypeScript：

- `normalizeSmartQuotes` — 智能引号转换
- `removeTrailingCommas` — 移除 JSON 尾逗号
- `stripJsonComments` — 剥离 JSON 注释
- `parseManifest` — 解析 manifest.json
- `recursiveFindManifests` — 递归查找 manifest
- `groupContentPacks` — 内容包分组
- `groupSameFolderMods` — 同文件夹分组
- `detectCategory` — 分类检测
- `scanMods` — 主扫描函数

- [ ] **Step 1: 创建 `electron/core/json-utils.ts`**

从 Rust 的 `normalize_smart_quotes`、`remove_trailing_commas`、`strip_json_comments` 逐行翻译为 TypeScript。

- [ ] **Step 2: 创建 `electron/core/mod-parser-logic.ts`**

定义 `ModManifest`、`ManifestDependency`、`ContentPackFor` 接口，实现 `parseManifest`、`recursiveFindManifests`、`groupContentPacks` 等函数。

- [ ] **Step 3: 创建 `electron/ipc/mod-parser.ts`**

注册 `scan_mods`、`toggle_mod_enabled`、`read_file_as_data_url` 三个 IPC handler。

- [ ] **Step 4: 在 `electron/ipc/index.ts` 中注册**

- [ ] **Step 5: 编写测试验证解析逻辑**

- [ ] **Step 6: 提交**

```bash
git add electron/core/ electron/ipc/mod-parser.ts
git commit -m "feat: migrate mod parser logic from Rust to TypeScript"
```

---

### Task 7: 迁移核心业务逻辑 — 日志解析器

**Files:**
- Create: `electron/core/log-parser-logic.ts`
- Create: `electron/ipc/log-parser.ts`

从 Rust 的 `log_parser.rs`（约 1900 行）移植：
- `parseErrorsV2` — SMAPI 日志错误解析
- `scanModsBasic` — 基础 mod 扫描
- `checkSmapiLog` — SMAPI 日志检查
- `runModHealthCheck` — 健康检查
- `ERROR_INDICATORS` 正则
- 所有 RULE 规则

- [ ] **Step 1: 创建 `electron/core/log-parser-logic.ts`**
- [ ] **Step 2: 创建 `electron/ipc/log-parser.ts`**
- [ ] **Step 3: 注册 IPC handlers**
- [ ] **Step 4: 提交**

---

### Task 8: 迁移核心业务逻辑 — Nexus API 与链接器

**Files:**
- Create: `electron/core/nexus-linker-data.ts`
- Create: `electron/ipc/nexus-api.ts`

从 Rust 的 `nexus_api.rs`（约 2300 行）和 `nexus_linker.rs` 移植：
- `BUILTIN_DICT`、`FOLDER_NAME_DICT` 数据
- `buildNexusLink` 函数
- Nexus API 请求（验证、搜索、下载）
- NXM 协议处理

- [ ] **Step 1: 创建 `electron/core/nexus-linker-data.ts`**
- [ ] **Step 2: 创建 `electron/ipc/nexus-api.ts`**
- [ ] **Step 3: 注册 IPC handlers**
- [ ] **Step 4: 提交**

---

### Task 9: 迁移其余后端模块

**Files:**
- Create: `electron/ipc/mod-installer.ts`
- Create: `electron/ipc/profiles.ts`
- Create: `electron/ipc/smapi.ts`
- Create: `electron/ipc/saves.ts`
- Create: `electron/ipc/update-checker.ts`
- Create: `electron/ipc/mod-backup.ts`
- Create: `electron/ipc/mod-config.ts`
- Create: `electron/ipc/mod-ordering.ts`
- Create: `electron/ipc/conflict-checker.ts`
- Create: `electron/ipc/mod-security.ts`
- Create: `electron/ipc/storage-analyzer.ts`
- Create: `electron/ipc/sync-manager.ts`
- Create: `electron/ipc/app-updater.ts`
- Create: `electron/ipc/mod-thumbnail.ts`
- Create: `electron/ipc/dep-resolver.ts`
- Create: `electron/ipc/app-logger.ts`

每个文件遵循相同模式：
1. 从 Rust 移植核心逻辑到 TypeScript
2. 注册 `ipcMain.handle('command_name', handler)` 
3. 在 `electron/ipc/index.ts` 中导入注册

- [ ] **Step 1-16: 逐个模块迁移并提交**

每个模块一个 commit：
```bash
git commit -m "feat: migrate <module-name> from Rust to TypeScript"
```

---

### Task 10: 更新前端组件引用

**Files:**
- Modify: 所有 25 个 Tauri 依赖文件

将所有 `import { invoke } from '@tauri-apps/api/core'` 替换为 `import { ... } from '../utils/electron-api'`。
将所有 `import { listen } from '@tauri-apps/api/event'` 替换为 `window.electronAPI.on`。
将所有 `import { open } from '@tauri-apps/plugin-dialog'` 替换为 `window.electronAPI.invoke('dialog-open', ...)`。
将所有 `getCurrentWindow()` 替换为 `window.electronAPI.invoke('window-...')`。

- [ ] **Step 1: 批量替换 `utils/tauri-api.ts` → `utils/electron-api.ts` 引用**

```bash
# 在 src/ 下搜索所有引用 tauri-api 的文件
grep -rl "from.*tauri-api" src/ | head -30
```

- [ ] **Step 2: 逐文件替换 import 语句**

- [ ] **Step 3: 替换 `listen` 调用**

- [ ] **Step 4: 替换 `plugin-dialog`、`plugin-opener`、`plugin-fs` 调用**

- [ ] **Step 5: 替换 `UpdateChecker` 中的 `@tauri-apps/plugin-updater`**

- [ ] **Step 6: 验证前端编译通过**

```bash
npx tsc --noEmit
```

- [ ] **Step 7: 提交**

```bash
git add src/
git commit -m "feat: migrate all frontend components from Tauri to Electron API"
```

---

### Task 11: 配置 Electron Builder 打包

**Files:**
- Create: `electron-builder.yml`

- [ ] **Step 1: 创建 `electron-builder.yml`**

```yaml
appId: com.svl.app
productName: SVL
directories:
  output: release
files:
  - dist/**/*
  - dist-electron/**/*
  - package.json
extraResources:
  - from: src-tauri/src/assets/mod_dict.json
    to: mod_dict.json
win:
  target:
    - nsis
  icon: src-tauri/icons/icon.ico
nsis:
  oneClick: false
  allowToChangeInstallationDirectory: true
  installerIcon: src-tauri/icons/icon.ico
  uninstallerIcon: src-tauri/icons/icon.ico
```

- [ ] **Step 2: 测试打包**

```bash
npm run electron:build
```

- [ ] **Step 3: 提交**

```bash
git add electron-builder.yml package.json
git commit -m "feat: configure electron-builder for packaging"
```

---

### Task 12: 端到端验证与清理

- [ ] **Step 1: 启动开发模式验证所有功能**

```bash
npm run dev
```

逐项验证：
- [ ] Mod 列表正确显示
- [ ] 启用/禁用 Mod
- [ ] 安装/卸载 Mod
- [ ] 日志解析
- [ ] Nexus API 连接
- [ ] 档案管理
- [ ] 存档管理
- [ ] 更新检查

- [ ] **Step 2: 删除不再需要的 Tauri 依赖**

从 `package.json` 移除：
- `@tauri-apps/api`
- `@tauri-apps/plugin-*`
- `@tauri-apps/cli`

- [ ] **Step 3: 删除 `src-tauri` 目录（可选，建议保留到验证完成）**

- [ ] **Step 4: 最终提交**

```bash
git add -A
git commit -m "feat: complete Tauri → Electron migration"
```

---

## 迁移优先级与风险

| 优先级 | 模块 | 风险 | 原因 |
|--------|------|------|------|
| P0 | IPC 适配层 + preload | 低 | 模式固定，一次写对 |
| P0 | Mod 解析器 | 高 | 1300 行 Rust，含 JSON 预处理、递归扫描、分组逻辑 |
| P0 | 日志解析器 | 高 | 1900 行 Rust，含大量正则规则 |
| P1 | Nexus API | 中 | HTTP 请求逻辑，但 API 格式固定 |
| P1 | Mod 安装器 | 中 | 涉及文件系统操作 |
| P1 | 档案管理 | 低 | CRUD 逻辑简单 |
| P2 | 其余模块 | 低 | 逻辑相对简单 |

## 关键注意事项

1. **`convertFileSrc`**：Tauri 用 `convertFileSrc` 将本地文件路径转为可访问 URL。Electron 中需要注册自定义协议（`svl-file://`）或使用 `file://` 协议。

2. **事件系统**：Tauri 的 `app.emit()` 对应 Electron 的 `mainWindow.webContents.send()`。所有事件名保持不变。

3. **窗口管理**：Tauri 的 `getCurrentWindow()` 对应 Electron 的 `BrowserWindow` API。自定义标题栏需要通过 IPC 调用 `mainWindow.minimize()` / `maximize()` / `close()`。

4. **自动更新**：Tauri 的 `@tauri-apps/plugin-updater` 对应 `electron-updater`。

5. **NXM 协议**：需要注册自定义 URL 协议处理，Electron 中通过 `app.setAsDefaultProtocolClient('nxm')` 实现。

6. **Windows 注册表**：Rust 中通过 `winreg` crate 读取注册表。Electron/Node.js 中需要使用 `regedit` 或 `child_process` 执行 `reg query` 命令。
