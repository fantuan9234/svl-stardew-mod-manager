"""
SVL 诊断测试 - 查看页面实际内容
"""
import pytest
from playwright.sync_api import Page

TAURI_MOCK_SCRIPT = """
window.__TAURI_INTERNALS__ = {
  metadata: { currentWindow: { label: 'main' } },
  invoke: async (cmd, args) => {
    console.log('[Tauri Mock] invoke:', cmd);
    switch (cmd) {
      case 'scan_mods':
        return [{ id: 'mod1', name: 'Test Mod', version: '1.0.0', author: 'Test', unique_id: 'Test.Mod1', enabled: true }];
      case 'get_game_path':
        return 'C:\\\\Games\\\\Stardew Valley';
      case 'get_smapi_status':
        return { installed: true, version: '4.0.0' };
      case 'list_profiles':
        return [];
      case 'get_saves_list':
        return [];
      case 'check_conflicts':
        return [];
      case 'scan_health':
        return { outdated_mods: [], missing_deps: [], conflicts: [] };
      case 'verify_nexus_api_key':
        return { success: false, message: 'Mock: API Key 无效' };
      default:
        return null;
    }
  },
  transformCallback: (cb) => cb,
};

window.__TAURI__ = {
  window: {
    getCurrentWindow: () => ({
      minimize: async () => {},
      toggleMaximize: async () => {},
      close: async () => {},
      isMaximized: async () => false,
      onResized: async () => ({ then: (fn) => { fn(() => {}); return { then: () => {} }; } }),
    }),
  },
  core: { invoke: async (cmd, args) => window.__TAURI_INTERNALS__.invoke(cmd, args) },
  dialog: { open: async () => null, save: async () => null },
  path: { appDataDir: async () => 'C:\\\\Users\\\\Test\\\\AppData\\\\Roaming\\\\SVL' },
  fs: { readTextFile: async () => '', writeTextFile: async () => {}, exists: async () => false },
  event: { listen: async () => () => {}, once: async () => () => {}, emit: async () => {} },
};
"""

@pytest.fixture(autouse=True)
def inject_tauri_mock(page: Page):
    page.add_init_script(TAURI_MOCK_SCRIPT)

def test_diagnostic_homepage(page: Page):
    """诊断首页内容"""
    page.goto("http://localhost:1420", timeout=30000)
    page.wait_for_timeout(3000)
    
    # 获取页面所有文本
    body_text = page.locator("body").inner_text()
    print("\n===== 首页文本内容 =====")
    print(body_text[:2000])
    print("========================\n")
    
    # 截图
    page.screenshot(path="test-screenshots/diagnostic_homepage.png", full_page=True)
    
    # 检查是否有 React 错误
    console_logs = []
    page.on("console", lambda msg: console_logs.append(msg.text))
    
    # 检查导航项
    nav_items = page.locator(".svl-nav-item")
    count = nav_items.count()
    print(f"导航项数量: {count}")
    
    for i in range(count):
        item = nav_items.nth(i)
        text = item.inner_text()
        print(f"  导航项 {i+1}: {text}")
    
    page.screenshot(path="test-screenshots/diagnostic_nav.png", full_page=True)

def test_diagnostic_mod_manager(page: Page):
    """诊断模组管理页面"""
    page.goto("http://localhost:1420/mod-manager", timeout=30000)
    page.wait_for_timeout(3000)
    
    body_text = page.locator("body").inner_text()
    print("\n===== 模组管理页面文本 =====")
    print(body_text[:2000])
    print("============================\n")
    
    page.screenshot(path="test-screenshots/diagnostic_mod_manager.png", full_page=True)
