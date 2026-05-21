import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { save } from '@tauri-apps/plugin-dialog';
import {
  profileList,
  profileCreate,
  profileDelete,
  profileExport,
  profileImport,
  checkSmapiStatus,
  type ProfileListItem,
} from '../utils/tauri-api';

interface ProfileManagerProps {
  isOpen: boolean;
  onClose: () => void;
  currentEnabledMods: string[];
  onProfileCreated: () => void;
}

export default function ProfileManager({ isOpen, onClose, currentEnabledMods, onProfileCreated }: ProfileManagerProps) {
  const { t } = useTranslation();
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');
  const [loading, setLoading] = useState(false);
  const [gamePath, setGamePath] = useState<string>('');

  useEffect(() => {
    checkSmapiStatus()
      .then((info) => {
        if (info.game_path) {
          setGamePath(info.game_path);
        }
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (isOpen && gamePath) {
      loadProfiles();
    }
  }, [isOpen, gamePath]);

  const loadProfiles = async () => {
    if (!gamePath) return;
    try {
      setLoading(true);
      const list = await profileList(gamePath);
      setProfiles(list);
    } catch (error) {
      console.error('Failed to load profiles:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleCreateProfile = async () => {
    if (!newProfileName.trim() || !gamePath) return;

    try {
      await profileCreate(gamePath, newProfileName, currentEnabledMods);
      setNewProfileName('');
      setShowCreateForm(false);
      onProfileCreated();
      loadProfiles();
    } catch (error) {
      console.error('Failed to create profile:', error);
    }
  };

  const handleDeleteProfile = async (profileName: string) => {
    if (!gamePath) return;
    if (!confirm(t('app.profiles.confirmDelete'))) return;

    try {
      await profileDelete(gamePath, profileName);
      loadProfiles();
    } catch (error) {
      console.error('Failed to delete profile:', error);
    }
  };

  const handleExportProfile = async (profile: ProfileListItem) => {
    if (!gamePath) return;
    try {
      const selected = await save({
        title: t('app.profiles.exportProfile'),
        defaultPath: `${profile.name}.svl_profile`,
        filters: [{ name: t('app.profileFile'), extensions: ['svl_profile', 'json'] }],
      });
      if (selected) {
        await profileExport(gamePath, profile.name, selected as string);
      }
    } catch (error) {
      console.error('Failed to export profile:', error);
    }
  };

  const handleImportProfile = async () => {
    if (!gamePath) return;
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.svl_profile,.json';

    input.onchange = async (e) => {
      const target = e.target as HTMLInputElement;
      if (target.files && target.files.length > 0) {
        const file = target.files[0];
        try {
          const filePath = (file as any).path || '';
          if (filePath) {
            await profileImport(gamePath, filePath);
            loadProfiles();
          }
        } catch (error) {
          console.error('Failed to import profile:', error);
        }
      }
    };

    input.click();
  };

  if (!isOpen) return null;

  return (
    <div className="svl-profile-modal-overlay" onClick={onClose}>
      <div className="svl-profile-modal" onClick={(e) => e.stopPropagation()}>
        <div className="svl-profile-modal-header">
          <h2>{t('app.profiles.title')}</h2>
          <button className="svl-profile-modal-close" onClick={onClose}>✕</button>
        </div>

        <div className="svl-profile-modal-content">
          <div className="svl-profile-modal-actions">
            <button
              className="svl-profile-btn-primary"
              onClick={() => setShowCreateForm(!showCreateForm)}
            >
              {t('app.profiles.createNew')}
            </button>
            <button
              className="svl-profile-btn-secondary"
              onClick={handleImportProfile}
            >
              {t('app.profiles.importProfile')}
            </button>
          </div>

          {showCreateForm && (
            <div className="svl-profile-create-form">
              <input
                type="text"
                placeholder={t('app.profilesPage.namePlaceholder')}
                value={newProfileName}
                onChange={(e) => setNewProfileName(e.target.value)}
              />
              <div className="svl-profile-form-actions">
                <button onClick={handleCreateProfile}>
                  {t('app.profilesPage.create')}
                </button>
                <button onClick={() => setShowCreateForm(false)}>
                  {t('app.profilesPage.cancel')}
                </button>
              </div>
            </div>
          )}

          <div className="svl-profile-list">
            {loading ? (
              <div className="svl-profile-loading">{t('app.profiles.loading')}</div>
            ) : (
              profiles.map(profile => (
                <div key={profile.name} className="svl-profile-list-item">
                  <div className="svl-profile-list-info">
                    <div className="svl-profile-list-name">{profile.name}</div>
                    <div className="svl-profile-list-meta">
                      {profile.enabled_count} {t('app.profiles.modEnabled')} · {new Date(profile.last_used).toLocaleDateString()}
                    </div>
                  </div>
                  <div className="svl-profile-list-actions">
                    <button
                      className="svl-profile-action-btn"
                      onClick={() => handleExportProfile(profile)}
                      title={t('app.profiles.exportProfile')}
                    >
                      📤
                    </button>
                    {!profile.is_protected && (
                      <button
                        className="svl-profile-action-btn svl-profile-action-btn--delete"
                        onClick={() => handleDeleteProfile(profile.name)}
                        title={t('app.profilesPage.delete')}
                      >
                        🗑️
                      </button>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
