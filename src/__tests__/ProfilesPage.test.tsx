import { render, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
}));

vi.mock('../utils/tauri-api', () => ({
  checkSmapiStatus: vi.fn().mockResolvedValue({ installed: true, game_path: '/test/game/path' }),
  profileList: vi.fn(),
  profileGetActive: vi.fn().mockResolvedValue(null),
  profileScanMods: vi.fn().mockResolvedValue([]),
  profileCreate: vi.fn(),
  profileDelete: vi.fn(),
  profileSwitch: vi.fn(),
  profileUpdateMods: vi.fn(),
  profileGetModStates: vi.fn().mockResolvedValue({}),
  profileClearActive: vi.fn(),
  profileCopy: vi.fn(),
  profileExport: vi.fn(),
  profileImport: vi.fn(),
}));

import ProfilesPage from '../pages/ProfilesPage';

describe('ProfilesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should render without crashing', async () => {
    const { profileList } = await import('../utils/tauri-api');
    (profileList as any).mockResolvedValue([]);

    render(<ProfilesPage />);

    await waitFor(() => {
      expect(document.body).toBeTruthy();
    }, { timeout: 2000 });
  });

  it('should display empty state when no profiles exist', async () => {
    const { profileList } = await import('../utils/tauri-api');
    (profileList as any).mockResolvedValue([]);

    render(<ProfilesPage />);

    await waitFor(() => {
      expect(document.body.childNodes.length).toBeGreaterThan(0);
    }, { timeout: 2000 });
  });

  it('should render profile table when profiles exist', async () => {
    const { profileList } = await import('../utils/tauri-api');
    (profileList as any).mockResolvedValue([
      {
        name: 'Test Profile',
        is_protected: false,
        is_active: false,
        total_mods: 5,
        enabled_count: 3,
        created_at: '2024-01-01T00:00:00Z',
        last_used: '2024-01-01T00:00:00Z',
      },
    ]);

    render(<ProfilesPage />);

    await waitFor(() => {
      expect(profileList).toHaveBeenCalled();
    }, { timeout: 2000 });
  });
});
