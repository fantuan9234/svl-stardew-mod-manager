import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import {
  profileList,
  profileGetActive,
  profileSwitch,
  checkSmapiStatus,
  type ProfileListItem,
} from '../utils/tauri-api';

interface ProfileSelectorProps {
  onProfileChange: (profile: ProfileListItem) => void;
  onManageProfiles: () => void;
}

export default function ProfileSelector({ onProfileChange, onManageProfiles }: ProfileSelectorProps) {
  const { t } = useTranslation();
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [activeProfileName, setActiveProfileName] = useState<string>('');
  const [isOpen, setIsOpen] = useState(false);
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
    if (gamePath) {
      loadProfiles();
    }
  }, [gamePath]);

  const loadProfiles = async () => {
    if (!gamePath) return;
    try {
      const list = await profileList(gamePath);
      setProfiles(list);

      const active = await profileGetActive(gamePath);
      if (active) {
        setActiveProfileName(active);
        const activeProfile = list.find((p: ProfileListItem) => p.name === active);
        if (activeProfile) {
          onProfileChange(activeProfile);
        }
      }
    } catch (error) {
      console.error('Failed to load profiles:', error);
    }
  };

  const handleSelectProfile = async (profile: ProfileListItem) => {
    if (!gamePath) return;
    try {
      await profileSwitch(gamePath, profile.name);
      setActiveProfileName(profile.name);
      onProfileChange(profile);
      setIsOpen(false);
    } catch (error) {
      console.error('Failed to switch profile:', error);
    }
  };

  const activeProfile = profiles.find(p => p.name === activeProfileName);

  return (
    <div className="svl-profile-selector">
      <button
        className="svl-profile-btn"
        onClick={() => setIsOpen(!isOpen)}
      >
        <span className="svl-profile-icon">📁</span>
        <span className="svl-profile-name">
          {activeProfile?.name || t('app.pages.modManager.selectProfile')}
        </span>
        <span className={`svl-profile-arrow ${isOpen ? 'open' : ''}`}>▼</span>
      </button>

      {isOpen && (
        <div className="svl-profile-dropdown">
          {profiles.map(profile => (
            <div
              key={profile.name}
              className={`svl-profile-item ${profile.name === activeProfileName ? 'active' : ''}`}
              onClick={() => handleSelectProfile(profile)}
            >
              <span className="svl-profile-item-name">{profile.name}</span>
              <span className="svl-profile-item-count">
                {profile.enabled_count} {t('app.pages.modManager.modsEnabled')}
              </span>
            </div>
          ))}

          <div className="svl-profile-divider" />

          <div
            className="svl-profile-item svl-profile-item--manage"
            onClick={() => {
              setIsOpen(false);
              onManageProfiles();
            }}
          >
            <span className="svl-profile-item-name">
              {t('app.pages.modManager.manageProfiles')}
            </span>
            <span className="svl-profile-item-icon">⚙️</span>
          </div>
        </div>
      )}
    </div>
  );
}
