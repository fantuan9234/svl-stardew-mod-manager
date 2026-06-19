import '@testing-library/jest-dom';
import { vi } from 'vitest';

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

Object.defineProperty(window, 'getComputedStyle', {
  value: () => ({
    getPropertyValue: () => '',
  }),
});

Object.defineProperty(window, 'IntersectionObserver', {
  value: vi.fn().mockImplementation(() => ({
    observe: vi.fn(),
    unobserve: vi.fn(),
    disconnect: vi.fn(),
  })),
});

class ResizeObserverMock {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}
Object.defineProperty(window, 'ResizeObserver', {
  value: ResizeObserverMock,
});

HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue(null) as any;

window.scrollTo = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `http://localhost/${path}`),
}));

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    minimize: vi.fn(),
    toggleMaximize: vi.fn(),
    close: vi.fn(),
    isMaximized: vi.fn().mockResolvedValue(false),
    onResized: vi.fn().mockResolvedValue(() => {}),
    onDragDropEvent: vi.fn().mockResolvedValue(() => {}),
    setTitle: vi.fn(),
    innerSize: vi.fn().mockResolvedValue({ width: 1280, height: 800 }),
    listen: vi.fn().mockResolvedValue(() => {}),
    emit: vi.fn(),
  })),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
  once: vi.fn().mockResolvedValue(vi.fn()),
  emit: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
  save: vi.fn().mockResolvedValue(null),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  revealItemInDir: vi.fn().mockResolvedValue(undefined),
  openUrl: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readDir: vi.fn().mockResolvedValue([]),
  readTextFile: vi.fn().mockResolvedValue(''),
  writeTextFile: vi.fn().mockResolvedValue(undefined),
  exists: vi.fn().mockResolvedValue(false),
  mkdir: vi.fn().mockResolvedValue(undefined),
  remove: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn().mockResolvedValue(null),
}));

vi.mock('@tauri-apps/plugin-process', () => ({
  exit: vi.fn(),
  relaunch: vi.fn(),
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, any>) => {
      if (params) {
        return Object.entries(params).reduce(
          (acc, [k, v]) => acc.replace(`{{${k}}}`, String(v)),
          key
        );
      }
      return key;
    },
    i18n: {
      changeLanguage: vi.fn(),
      language: 'zh',
    },
  }),
  initReactI18next: {
    type: '3rdParty',
    init: vi.fn(),
  },
  Trans: ({ children }: any) => children,
}));

vi.mock('../hooks/useTheme', () => ({
  useTheme: () => ({
    theme: 'oceanBlue',
    switchTheme: vi.fn(),
    getAntdThemeConfig: () => ({
      primaryColor: '#5c8d4a',
      primaryHover: '#6fa05d',
      bgCard: '#1a1a2e',
      bgCardHover: '#252540',
      borderColor: '#333355',
      textColor: '#e0e0e0',
      textPlaceholder: '#888',
      headerBg: '#222240',
    }),
  }),
  ThemeProvider: ({ children }: any) => children,
}));

vi.mock('../hooks/useSplashDone', () => ({
  useSplashDone: () => true,
  SplashProvider: ({ children }: any) => children,
}));

vi.mock('../hooks/usePageActive', () => ({
  usePageActive: () => true,
  PageActiveProvider: ({ children }: any) => children,
}));

vi.mock('../hooks/useNexusStatus', () => ({
  getNexusStatus: vi.fn(() => ({ hasApiKey: false, lastChecked: null })),
  setNexusStatus: vi.fn(),
  verifyNexusConnection: vi.fn().mockResolvedValue(undefined),
  useNexusStatus: () => ({
    hasApiKey: false,
    isPremium: false,
    userName: null,
    lastChecked: null,
    setApiKey: vi.fn(),
    clearApiKey: vi.fn(),
    verify: vi.fn().mockResolvedValue(false),
  }),
}));

vi.mock('../utils/openUrl', () => ({
  openUrl: vi.fn().mockResolvedValue(undefined),
}));

export function createMockModInfo(overrides: Partial<import('../utils/tauri-api').ModInfo> = {}): import('../utils/tauri-api').ModInfo {
  return {
    name: 'Test Mod',
    version: '1.0.0',
    author: 'TestAuthor',
    description: 'A test mod',
    unique_id: 'TestAuthor.TestMod',
    enabled: true,
    is_required: false,
    has_dependencies: false,
    dependency_count: 0,
    is_content_pack: false,
    content_pack_for: null,
    folder_path: '/game/Mods/TestMod',
    has_conflict: false,
    conflict_warning: null,
    url: null,
    category: 'other',
    screenshot_path: null,
    thumbnail_path: null,
    has_update: false,
    latest_version: null,
    update_url: null,
    update_notes: null,
    nexus_id: null,
    nexus_mod_id: null,
    dependencies: [],
    manifest_content: null,
    sub_mods: [],
    is_group: false,
    ...overrides,
  };
}

export function createMockSmapiInfo(overrides: Partial<import('../utils/tauri-api').SmapiInfo> = {}): import('../utils/tauri-api').SmapiInfo {
  return {
    installed: true,
    version: '4.0.0',
    game_path: '/game/path',
    error: null,
    ...overrides,
  };
}

export function createMockSaveInfo(overrides: Partial<import('../utils/tauri-api').SaveInfo> = {}): import('../utils/tauri-api').SaveInfo {
  return {
    name: 'TestSave',
    farm_name: 'TestFarm',
    farm_type: 'Standard',
    farm_type_id: 0,
    game_version: '1.6.0',
    hours_played: 100,
    days_played: 50,
    money: 50000,
    total_money_earned: 100000,
    day_of_month: 15,
    current_season: 'Spring',
    year: 2,
    time_of_day: 1200,
    deepest_mine_level: 100,
    grandpa_score: 3,
    perfection_score: 50,
    total_skill_levels: 25,
    farming_level: 5,
    mining_level: 5,
    foraging_level: 5,
    fishing_level: 5,
    combat_level: 5,
    spouse: '',
    friendship_count: 10,
    building_count: 5,
    quest_count: 3,
    item_count: 100,
    recipes_known: 50,
    has_finished_community_center: false,
    ginger_island_unlocked: false,
    stardrops_found: 5,
    activated_golden_parrot: false,
    file_size_mb: 2.5,
    last_modified: '2026-01-01T00:00:00Z',
    save_path: '/saves/TestSave',
    backup_count: 0,
    linked_profile: null,
    character_name: 'TestPlayer',
    details_loadable: true,
    ...overrides,
  };
}

export function createMockProfileData(overrides: Partial<import('../utils/tauri-api').ProfileData> = {}): import('../utils/tauri-api').ProfileData {
  return {
    name: 'TestProfile',
    is_protected: false,
    enabled_mod_ids: ['TestAuthor.TestMod'],
    created_at: '2026-01-01T00:00:00Z',
    last_used: '2026-01-01T00:00:00Z',
    ...overrides,
  };
}
