"""
SVL 全功能回归测试 - Playwright
测试 Stardew Valley Mod Manager (SVL) 的所有核心功能
"""
import pytest
import time
import json
import os
from pathlib import Path
from playwright.sync_api import Page, expect, BrowserContext

# 测试配置
SVL_DEV_URL = "http://localhost:1420"  # Tauri dev server 默认端口
SCREENSHOT_DIR = Path("test-screenshots")

# Tauri 2.x API Mock 脚本 - 在页面加载前注入
TAURI_MOCK_SCRIPT = """
// Mock Tauri 2.x API for browser testing
window.__TAURI_INTERNALS__ = {
  metadata: {
    currentWindow: {
      label: 'main'
    }
  },
  invoke: async (cmd, args) => {
    console.log('[Tauri Mock] invoke:', cmd, args);
    switch (cmd) {
      case 'scan_mods':
        return [
          {
            id: 'mod1',
            name: 'Test Mod 1',
            version: '1.0.0',
            author: 'TestAuthor',
            unique_id: 'TestAuthor.TestMod1',
            enabled: true,
            description: 'A test mod'
          },
          {
            id: 'mod2',
            name: 'Test Mod 2',
            version: '2.0.0',
            author: 'TestAuthor2',
            unique_id: 'TestAuthor2.TestMod2',
            enabled: false,
            description: 'Another test mod'
          }
        ];
      case 'plugin:window|get_all_windows':
        return [];
      case 'list_profiles':
        return [];
      case 'get_game_path':
        return 'C:\\\\Games\\\\Stardew Valley';
      case 'get_smapi_status':
        return { installed: true, version: '4.0.0' };
      case 'check_conflicts':
        return [];
      case 'scan_health':
        return { outdated_mods: [], missing_deps: [], conflicts: [] };
      case 'verify_nexus_api_key':
        return { success: false, message: 'Mock: API Key 无效' };
      case 'get_saves_list':
        return [
          {
            name: 'TestSave1',
            farmName: 'Test Farm',
            playerCount: 1,
            lastModified: '2024-01-01'
          }
        ];
      case 'get_profile_bindings':
        return {};
      case 'plugin:window|is_maximized':
        return false;
      case 'plugin:window|scale_factor':
        return 1;
      case 'plugin:window|inner_position':
        return { x: 0, y: 0 };
      case 'plugin:window|outer_position':
        return { x: 0, y: 0 };
      case 'plugin:window|inner_size':
        return { width: 1200, height: 800 };
      case 'plugin:window|outer_size':
        return { width: 1200, height: 800 };
      case 'plugin:window|is_fullscreen':
      case 'plugin:window|is_minimized':
      case 'plugin:window|is_focused':
      case 'plugin:window|is_decorated':
      case 'plugin:window|is_resizable':
      case 'plugin:window|is_maximizable':
      case 'plugin:window|is_minimizable':
      case 'plugin:window|is_closable':
      case 'plugin:window|is_visible':
      case 'plugin:window|is_always_on_top':
        return false;
      case 'plugin:window|title':
        return 'SVL';
      case 'plugin:window|theme':
        return 'dark';
      default:
        return null;
    }
  },
  transformCallback: (cb) => cb,
};

// Mock @tauri-apps/api/window getCurrentWindow
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
      return window.__TAURI_INTERNALS__.invoke(cmd, args);
    },
  },
  dialog: {
    open: async () => null,
    save: async () => null,
  },
  path: {
    appDataDir: async () => 'C:\\\\Users\\\\Test\\\\AppData\\\\Roaming\\\\SVL',
    homeDir: async () => 'C:\\\\Users\\\\Test',
  },
  fs: {
    readTextFile: async () => '',
    writeTextFile: async () => {},
    exists: async () => false,
  },
  event: {
    listen: async () => () => {},
    once: async () => () => {},
    emit: async () => {},
  },
};

console.log('[Tauri Mock] Tauri 2.x API mock loaded successfully');
"""

def setup_module(module):
    """测试开始前创建截图目录"""
    SCREENSHOT_DIR.mkdir(exist_ok=True)

def record_test(step: str, status: str, detail: str = "", screenshot: str = ""):
    """记录测试结果"""
    icon = "✅" if status == "PASS" else "❌"
    print(f"\n{icon} {step}: {status}")
    if detail:
        print(f"   详情: {detail}")

def take_screenshot(page: Page, name: str) -> str:
    """截图并返回路径"""
    path = SCREENSHOT_DIR / f"{name}.png"
    page.screenshot(path=str(path), full_page=True)
    return str(path)

@pytest.fixture(autouse=True)
def inject_tauri_mock(page: Page):
    """自动注入 Tauri Mock 到每个测试"""
    page.add_init_script(TAURI_MOCK_SCRIPT)

# ==================== 测试用例 ====================

class TestSVLRegression:
    """SVL 全功能回归测试"""

    def test_01_sidebar_menu_display(self, page: Page):
        """测试 1: 侧边栏菜单完整性"""
        step = "1. 侧边栏菜单显示"
        try:
            page.goto(SVL_DEV_URL, timeout=30000)
            page.wait_for_load_state("networkidle", timeout=10000)
            
            # 检查所有菜单项
            expected_menus = [
                "模组管理", "MOD 健康", "档案管理", 
                "存档管理", "联机同步", "设置", "支持作者"
            ]
            
            missing = []
            for menu_text in expected_menus:
                try:
                    locator = page.get_by_text(menu_text, exact=False)
                    locator.wait_for(state="visible", timeout=5000)
                except:
                    missing.append(menu_text)
            
            if missing:
                screenshot = take_screenshot(page, "01_sidebar_missing")
                record_test(step, "FAIL", f"缺失菜单: {', '.join(missing)}", screenshot)
                pytest.fail(f"缺失菜单: {missing}")
            else:
                record_test(step, "PASS", f"所有 {len(expected_menus)} 个菜单项均存在")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "01_sidebar_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_02_page_navigation(self, page: Page):
        """测试 2: 页面切换无白屏"""
        step = "2. 页面切换无白屏"
        try:
            routes = [
                ("/mod-manager", "模组管理"),
                ("/health", "MOD 健康"),
                ("/profiles", "档案管理"),
                ("/saves", "存档管理"),
                ("/sync", "联机同步"),
                ("/settings", "设置"),
                ("/donate", "支持作者"),
            ]
            
            failed_routes = []
            for route, name in routes:
                page.goto(f"{SVL_DEV_URL}{route}", timeout=10000)
                page.wait_for_timeout(1000)
                
                # 检查页面是否有内容（不是空白）
                body = page.locator("body")
                body_text = body.inner_text(timeout=5000)
                
                if not body_text.strip():
                    failed_routes.append(name)
            
            if failed_routes:
                screenshot = take_screenshot(page, "02_navigation_blank")
                record_test(step, "FAIL", f"白屏页面: {', '.join(failed_routes)}", screenshot)
                pytest.fail(f"白屏页面: {failed_routes}")
            else:
                record_test(step, "PASS", f"所有 {len(routes)} 个页面切换正常")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "02_navigation_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_03_settings_page_no_freeze(self, page: Page):
        """测试 3: 设置页面不卡死"""
        step = "3. 设置页面不卡死"
        try:
            # 进入设置页面
            page.goto(f"{SVL_DEV_URL}/settings", timeout=10000)
            page.wait_for_timeout(2000)
            
            # 立即点击其他菜单，验证能否跳转
            start_time = time.time()
            page.get_by_text("模组管理").click(timeout=5000)
            page.wait_for_timeout(1000)
            
            # 验证是否成功跳转
            current_url = page.url
            if "mod-manager" in current_url:
                elapsed = time.time() - start_time
                record_test(step, "PASS", f"设置页面跳转响应时间: {elapsed:.2f}s")
                assert True
            else:
                screenshot = take_screenshot(page, "03_settings_freeze")
                record_test(step, "FAIL", "无法从设置页面跳转", screenshot)
                pytest.fail("设置页面卡死")
                
        except Exception as e:
            screenshot = take_screenshot(page, "03_settings_freeze_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_04_mod_manager_display(self, page: Page):
        """测试 4: MOD 管理页面显示"""
        step = "4. MOD 管理页面显示"
        try:
            page.goto(f"{SVL_DEV_URL}/mod-manager", timeout=10000)
            page.wait_for_timeout(2000)
            
            # 检查搜索框
            search_box = page.locator("input[placeholder*='搜索']")
            search_box.wait_for(state="visible", timeout=5000)
            
            # 检查筛选按钮
            filter_all = page.get_by_text("全部")
            filter_all.wait_for(state="visible", timeout=5000)
            
            # 检查排序下拉框
            sort_dropdown = page.locator(".ant-select")
            sort_dropdown.wait_for(state="visible", timeout=5000)
            
            # 检查拖拽安装区域
            dropzone = page.get_by_text("拖拽")
            dropzone.wait_for(state="visible", timeout=5000)
            
            record_test(step, "PASS", "MOD 管理页面元素完整")
            assert True
            
        except Exception as e:
            screenshot = take_screenshot(page, "04_mod_manager_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_05_no_bulk_checkboxes(self, page: Page):
        """测试 5: 无批量选择复选框"""
        step = "5. 无批量选择复选框"
        try:
            page.goto(f"{SVL_DEV_URL}/mod-manager", timeout=10000)
            page.wait_for_timeout(2000)
            
            # 检查表格头部是否有复选框
            checkboxes = page.locator("thead .ant-checkbox-wrapper")
            count = checkboxes.count()
            
            if count > 0:
                screenshot = take_screenshot(page, "05_checkboxes_exist")
                record_test(step, "FAIL", f"发现 {count} 个批量选择复选框", screenshot)
                pytest.fail("存在批量选择复选框")
            else:
                record_test(step, "PASS", "无批量选择复选框")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "05_checkboxes_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_06_profiles_page_empty_state(self, page: Page):
        """测试 6: 档案管理空状态"""
        step = "6. 档案管理空状态显示"
        try:
            page.goto(f"{SVL_DEV_URL}/profiles", timeout=10000)
            page.wait_for_timeout(2000)
            
            # 检查是否显示空状态（不是原始键名）
            body_text = page.locator("body").inner_text(timeout=5000)
            
            if "app.profiles.empty" in body_text:
                screenshot = take_screenshot(page, "06_profiles_raw_key")
                record_test(step, "FAIL", "显示原始 i18n 键名: app.profiles.empty", screenshot)
                pytest.fail("显示原始 i18n 键名")
            else:
                record_test(step, "PASS", "空状态显示正常翻译文本")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "06_profiles_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_07_donate_page_images(self, page: Page):
        """测试 7: 支持作者页面图片显示"""
        step = "7. 支持作者页面图片显示"
        try:
            page.goto(f"{SVL_DEV_URL}/donate", timeout=10000)
            page.wait_for_timeout(2000)
            
            # 检查标题
            title = page.get_by_text("加鸡腿")
            title.wait_for(state="visible", timeout=5000)
            
            # 检查图片（至少应该有 img 标签）
            images = page.locator("img")
            count = images.count()
            
            if count >= 2:
                record_test(step, "PASS", f"找到 {count} 张图片")
                assert True
            else:
                screenshot = take_screenshot(page, "07_donate_images")
                record_test(step, "FAIL", f"只找到 {count} 张图片，预期至少 2 张", screenshot)
                pytest.fail("图片数量不足")
                
        except Exception as e:
            screenshot = take_screenshot(page, "07_donate_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_08_health_check_page(self, page: Page):
        """测试 8: MOD 健康页面"""
        step = "8. MOD 健康页面"
        try:
            page.goto(f"{SVL_DEV_URL}/health", timeout=10000)
            page.wait_for_timeout(2000)
            
            # 检查页面是否有内容
            body_text = page.locator("body").inner_text(timeout=5000)
            
            if not body_text.strip():
                screenshot = take_screenshot(page, "08_health_blank")
                record_test(step, "FAIL", "MOD 健康页面为空", screenshot)
                pytest.fail("MOD 健康页面为空")
            else:
                record_test(step, "PASS", "MOD 健康页面加载正常")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "08_health_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_09_sync_page_profiles_loading(self, page: Page):
        """测试 9: 联机同步页面档案加载"""
        step = "9. 联机同步页面档案加载"
        try:
            page.goto(f"{SVL_DEV_URL}/sync", timeout=10000)
            page.wait_for_timeout(5000)  # 等待档案加载
            
            body_text = page.locator("body").inner_text(timeout=5000)
            
            # 检查是否还在加载中
            if "加载中" in body_text or "loading" in body_text.lower():
                screenshot = take_screenshot(page, "09_sync_loading")
                record_test(step, "FAIL", "档案列表一直显示加载中", screenshot)
                pytest.fail("档案加载超时")
            else:
                record_test(step, "PASS", "档案加载完成（可能为空但不会无限加载）")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "09_sync_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))

    def test_10_i18n_no_raw_keys(self, page: Page):
        """测试 10: 无原始 i18n 键名暴露"""
        step = "10. 无原始 i18n 键名暴露"
        try:
            raw_keys = ["app.", "sidebar.", "donate.", "saves."]
            found_keys = []
            
            for route in ["/mod-manager", "/health", "/profiles", "/saves", "/sync", "/settings", "/donate"]:
                page.goto(f"{SVL_DEV_URL}{route}", timeout=10000)
                page.wait_for_timeout(1000)
                
                body_text = page.locator("body").inner_text(timeout=5000)
                
                for key_prefix in raw_keys:
                    if key_prefix in body_text:
                        # 找到可能的原始键名
                        lines = body_text.split('\n')
                        for line in lines:
                            if key_prefix in line:
                                found_keys.append(line.strip()[:50])
                                break
            
            if found_keys:
                screenshot = take_screenshot(page, "10_i18n_raw_keys")
                record_test(step, "FAIL", f"发现原始键名: {found_keys[:5]}", screenshot)
                pytest.fail(f"发现原始 i18n 键名: {found_keys}")
            else:
                record_test(step, "PASS", "所有页面均正确翻译")
                assert True
                
        except Exception as e:
            screenshot = take_screenshot(page, "10_i18n_error")
            record_test(step, "FAIL", str(e), screenshot)
            pytest.fail(str(e))


# ==================== 测试报告生成 ====================

def pytest_terminal_summary(terminalreporter, exitstatus, config):
    """测试结束后输出报告"""
    print("\n" + "="*80)
    print("SVL 全功能回归测试报告")
    print("="*80)
    
    passed = len(terminalreporter.stats.get('passed', []))
    failed = len(terminalreporter.stats.get('failed', []))
    total = passed + failed
    
    print(f"\n测试总数: {total}")
    print(f"通过数: {passed}")
    print(f"失败数: {failed}")
    print(f"通过率: {passed/total*100:.1f}%" if total > 0 else "N/A")
    
    if failed > 0:
        print(f"\n失败详情:")
        print("-"*40)
        for item in terminalreporter.stats.get('failed', []):
            print(f"❌ {item.nodeid}")
            if hasattr(item, 'longrepr'):
                print(f"   {str(item.longrepr)[:200]}")
    
    print("\n" + "="*80)
    print(f"截图已保存到: {SCREENSHOT_DIR.absolute()}")
    print("="*80)
