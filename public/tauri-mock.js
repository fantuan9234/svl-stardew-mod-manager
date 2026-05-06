/**
 * Tauri API Mock for Browser Testing
 * 在浏览器环境中模拟 Tauri API，使 SVL 应用可以在 Playwright 测试中运行
 */

// Mock @tauri-apps/api/window
window.__TAURI__ = {
  window: {
    getCurrentWindow: () => ({
      minimize: async () => {},
      toggleMaximize: async () => {},
      close: async () => {},
      isMaximized: async () => false,
      onResized: async () => ({
        then: (fn) => { fn(() => {}); return { then: () => {} }; }
      }),
    }),
  },
  core: {
    invoke: async (cmd, args) => {
      console.log(`[Tauri Mock] invoke: ${cmd}`, args);
      // 返回模拟数据
      switch (cmd) {
        case 'scan_mods':
          return [];
        case 'list_profiles':
          return [];
        case 'get_game_path':
          return '';
        case 'get_smapi_status':
          return { installed: false, version: null };
        case 'check_conflicts':
          return [];
        case 'scan_health':
          return { outdated_mods: [], missing_deps: [], conflicts: [] };
        case 'verify_nexus_api_key':
          return { success: false, message: 'Mock: API Key 无效' };
        case 'get_saves_list':
          return [];
        default:
          return null;
      }
    },
  },
  dialog: {
    open: async () => null,
    save: async () => null,
  },
  path: {
    appDataDir: async () => 'C:\\Users\\Test\\AppData\\Roaming\\SVL',
    homeDir: async () => 'C:\\Users\\Test',
  },
  fs: {
    readTextFile: async () => '',
    writeTextFile: async () => {},
    exists: async () => false,
  },
};

// Mock @tauri-apps/api/core
window.__TAURI_INTERNALS__ = {
  invoke: async (cmd, args) => {
    return window.__TAURI__.core.invoke(cmd, args);
  },
};

console.log('[Tauri Mock] Tauri API mock loaded');
