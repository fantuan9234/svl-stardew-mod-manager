// i18n translation file tests
import { describe, it, expect } from 'vitest';
import zh from '../i18n/zh.json';
import en from '../i18n/en.json';

// Helper: flatten nested object keys
function flattenKeys(obj: Record<string, any>, prefix = ''): string[] {
  let keys: string[] = [];
  for (const key in obj) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (typeof obj[key] === 'object' && obj[key] !== null && !Array.isArray(obj[key])) {
      keys = keys.concat(flattenKeys(obj[key], fullKey));
    } else {
      keys.push(fullKey);
    }
  }
  return keys;
}

describe('i18n Translation Files', () => {
  it('should have the same number of keys in zh.json and en.json', () => {
    const zhKeys = flattenKeys(zh);
    const enKeys = flattenKeys(en);

    expect(zhKeys.length).toBe(enKeys.length);

    // Find missing keys
    const zhSet = new Set(zhKeys);
    const enSet = new Set(enKeys);

    const missingInEn = zhKeys.filter((k) => !enSet.has(k));
    const missingInZh = enKeys.filter((k) => !zhSet.has(k));

    expect(missingInEn).toEqual([]);
    expect(missingInZh).toEqual([]);
  });

  it('should have all keys used in components defined', () => {
    // List of keys used in components (manually collected from codebase)
    const usedKeys = [
      'app.title',
      'app.nav.mods',
      'app.nav.profiles',
      'app.nav.saves',
      'app.nav.sync',
      'app.nav.settings',
      'app.theme.title',
      'app.theme.colorful',
      'app.theme.eyeCare',
      'app.language.switch',
      'app.language.zh',
      'app.language.en',
      'app.pages.settings.title',
      'app.pages.settings.networkDiagHint',
      'app.profiles.empty',
      'app.profiles.createNew',
      'app.profiles.switch',
      'app.profiles.editMods',
      'app.profiles.exitProfile',
      'app.profiles.activeProfile',
      'app.profilesPage.title',
      'app.profilesPage.createProfile',
      'app.profilesPage.confirmDelete',
      'app.profilesPage.deleteSuccess',
      'app.profilesPage.deleteFailed',
      'app.profilesPage.exitSuccess',
      'app.profilesPage.exitFailed',
      'app.pages.conflicts.downloadNexus',
      'app.pages.conflicts.solution.button',
      'app.sync.export.tab',
      'app.sync.import.tab',
      'app.sync.export.profileLabel',
      'app.sync.export.exportButton',
      'app.sync.import.fileLabel',
      'app.sync.import.profileLabel',
      'app.sync.import.compareButton',
      'sidebar.donate',
      'donate.title',
      'donate.subtitle',
      'donate.wechat',
      'donate.alipay',
      'donate.thanks',
      'donate.clickToEnlarge',
      'donate.contact',
      'app.modCard.toggleFailed',
      'app.modCard.toggleSuccess',
      'app.modCard.uninstallSuccess',
      'app.modCard.uninstallFailed',
    ];

    const zhKeys = flattenKeys(zh);
    const zhSet = new Set(zhKeys);

    const missingKeys = usedKeys.filter((key) => !zhSet.has(key));

    expect(missingKeys).toEqual([]);
  });

  it('should have non-empty values for all keys', () => {
    const zhKeys = flattenKeys(zh);
    const enKeys = flattenKeys(en);

    // Check zh.json values
    for (const key of zhKeys) {
      const parts = key.split('.');
      let value: any = zh;
      for (const part of parts) {
        value = value?.[part];
      }
      expect(value).toBeDefined();
      expect(value).not.toBe('');
    }

    // Check en.json values
    for (const key of enKeys) {
      const parts = key.split('.');
      let value: any = en;
      for (const part of parts) {
        value = value?.[part];
      }
      expect(value).toBeDefined();
      expect(value).not.toBe('');
    }
  });
});
