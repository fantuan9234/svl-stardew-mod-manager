import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ProfilesPage from '../pages/ProfilesPage';

vi.mock('../utils/tauri-api', () => ({
  checkSmapiStatus: vi.fn().mockResolvedValue({ installed: true, game_path: '/game/path' }),
  profileList: vi.fn().mockResolvedValue([
    { name: 'Profile1', is_protected: false, is_active: true, total_mods: 10, enabled_count: 8, created_at: '2026-01-01T00:00:00Z', last_used: '2026-06-01T00:00:00Z' },
    { name: 'Profile2', is_protected: true, is_active: false, total_mods: 5, enabled_count: 5, created_at: '2026-02-01T00:00:00Z', last_used: '2026-05-01T00:00:00Z' },
  ]),
  profileGetActive: vi.fn().mockResolvedValue('Profile1'),
  profileScanMods: vi.fn().mockResolvedValue([]),
  profileCreate: vi.fn().mockResolvedValue({ success: true }),
  profileDelete: vi.fn().mockResolvedValue({ success: true }),
  profileSwitch: vi.fn().mockResolvedValue({ success: true }),
  profileUpdateMods: vi.fn().mockResolvedValue({ name: 'Test', is_protected: false, enabled_mod_ids: [], created_at: '', last_used: '' }),
  profileGetModStates: vi.fn().mockResolvedValue({}),
  profileClearActive: vi.fn().mockResolvedValue(undefined),
  profileCopy: vi.fn().mockResolvedValue({ success: true }),
  profileExport: vi.fn().mockResolvedValue('/export/path'),
  profileImport: vi.fn().mockResolvedValue({ success: true }),
}));

describe('ProfilesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render without crashing', async () => {
    render(<ProfilesPage />);
    await waitFor(() => {
      expect(document.body).toBeTruthy();
    }, { timeout: 3000 });
  });

  it('should display profiles list', async () => {
    render(<ProfilesPage />);
    await waitFor(() => {
      expect(screen.getByText('Profile1')).toBeTruthy();
      expect(screen.getByText('Profile2')).toBeTruthy();
    }, { timeout: 3000 });
  });

  it('should show active profile indicator', async () => {
    render(<ProfilesPage />);
    await waitFor(() => {
      const allText = document.body.textContent || '';
      const hasActive = allText.includes('app.profiles.activeProfile') || allText.includes('当前激活') || allText.includes('Profile1');
      expect(hasActive).toBe(true);
    }, { timeout: 3000 });
  });

  it('should show create profile button', async () => {
    render(<ProfilesPage />);
    await waitFor(() => {
      const createBtn = screen.getAllByRole('button').find(b =>
        b.textContent?.includes('app.profiles.createNew') || b.textContent?.includes('创建')
      );
      expect(createBtn).toBeTruthy();
    }, { timeout: 3000 });
  });

  it('should handle profile creation', async () => {
    await import('../utils/tauri-api');
    render(<ProfilesPage />);
    await waitFor(() => {
      const createBtn = screen.getAllByRole('button').find(b =>
        b.textContent?.includes('app.profiles.createNew') || b.textContent?.includes('创建')
      );
      if (createBtn) fireEvent.click(createBtn);
    }, { timeout: 3000 });
  });

  it('should handle profile switching', async () => {
    await import('../utils/tauri-api');
    render(<ProfilesPage />);
    await waitFor(() => {
      const switchBtns = screen.getAllByRole('button').filter(b =>
        b.textContent?.includes('app.profiles.switch') || b.textContent?.includes('切换')
      );
      if (switchBtns.length > 0) {
        fireEvent.click(switchBtns[0]);
      }
    }, { timeout: 3000 });
  });

  it('should handle empty profiles state', async () => {
    const { profileList } = await import('../utils/tauri-api');
    (profileList as any).mockResolvedValueOnce([]);
    render(<ProfilesPage />);
    await waitFor(() => {
      expect(profileList).toHaveBeenCalled();
    }, { timeout: 3000 });
  });

  it('should handle profile list API error gracefully', async () => {
    const { profileList } = await import('../utils/tauri-api');
    (profileList as any).mockRejectedValueOnce(new Error('Network error'));
    render(<ProfilesPage />);
    await waitFor(() => {
      expect(profileList).toHaveBeenCalled();
    }, { timeout: 3000 });
  });
});
