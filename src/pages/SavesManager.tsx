import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Card, Button, Tag, Modal, message, Empty, Select, List, Popconfirm, Tooltip } from 'antd';
import {
  FolderOpenOutlined,
  SaveOutlined,
  UndoOutlined,
  LinkOutlined,
  DisconnectOutlined,
  ClockCircleOutlined,
  DatabaseOutlined,
  PlayCircleOutlined,
  UserOutlined,
  HomeOutlined,
  LockOutlined,
} from '@ant-design/icons';
import { open } from '@tauri-apps/plugin-dialog';
import {
  scanSaves,
  backupSave,
  restoreSave,
  listSaveBackups,
  linkSaveToProfile,
  unlinkSaveFromProfile,
  launchGameWithSaveProfile,
  openSaveLocation,
  openBackupDialog,
  profileList,
  profileGetActive,
  checkSmapiStatus,
  scanProfileMods,
  type SaveInfo,
  type BackupInfo,
  type ProfileListItem,
  type SmapiInfo,
  type ProfileModInfo,
} from '../utils/tauri-api';

export default function SavesManager() {
  const { t } = useTranslation();
  const [gamePath, setGamePath] = useState<string>('');
  const [saves, setSaves] = useState<SaveInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [backups, setBackups] = useState<BackupInfo[]>([]);
  const [selectedSave, setSelectedSave] = useState<SaveInfo | null>(null);
  const [showBackupModal, setShowBackupModal] = useState(false);
  const [showRestoreModal, setShowRestoreModal] = useState(false);
  const [backingUp, setBackingUp] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [profilesLoaded, setProfilesLoaded] = useState(false);
  const [launchingSave, setLaunchingSave] = useState<string | null>(null);
  const [changingProfile, setChangingProfile] = useState<string | null>(null);
  const [activeProfileName, setActiveProfileName] = useState<string | null>(null);
  const [requiredMods, setRequiredMods] = useState<ProfileModInfo[]>([]);

  useEffect(() => {
    checkSmapiStatus()
      .then((info: SmapiInfo) => {
        if (info.game_path) {
          setGamePath(info.game_path);
        }
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    loadSaves();
  }, []);

  useEffect(() => {
    if (gamePath && !profilesLoaded) {
      loadProfiles();
      loadActiveProfile();
      loadRequiredMods();
    }
  }, [gamePath]);

  const loadRequiredMods = async () => {
    if (!gamePath) return;
    try {
      const allMods = await scanProfileMods(gamePath);
      const required = allMods.filter((mod: ProfileModInfo) => mod.is_required);
      setRequiredMods(required);
    } catch {
      console.error('Failed to load required mods');
    }
  };

  const loadActiveProfile = async () => {
    if (!gamePath) return;
    try {
      const activeName = await profileGetActive(gamePath);
      setActiveProfileName(activeName);
    } catch {
      setActiveProfileName(null);
    }
  };

  const loadSaves = async () => {
    setLoading(true);
    try {
      const list = await scanSaves();
      setSaves(list);
    } catch {
      message.error(t('app.saves.loadFailed'));
    } finally {
      setLoading(false);
    }
  };

  const loadProfiles = async () => {
    if (!gamePath) return;
    try {
      const list = await profileList(gamePath);
      setProfiles(list);
      setProfilesLoaded(true);
    } catch {
      message.error(t('app.saves.loadProfilesFailed'));
    }
  };

  const handleOpenSaveLocation = async () => {
    try {
      await openSaveLocation();
    } catch {
      message.error(t('app.saves.openLocationFailed'));
    }
  };

  const handleBackup = async (save: SaveInfo) => {
    setSelectedSave(save);
    setShowBackupModal(true);
  };

  const doBackup = async () => {
    if (!selectedSave) return;

    const backupDir = await open({
      title: t('app.saves.selectBackupDir'),
      directory: true,
    });

    if (!backupDir) return;

    setBackingUp(true);
    try {
      const result = await backupSave(selectedSave.save_path, backupDir as string);
      message.success(result.message);
      setShowBackupModal(false);
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.backupFailed'));
    } finally {
      setBackingUp(false);
    }
  };

  const handleRestore = async (save: SaveInfo) => {
    setSelectedSave(save);
    try {
      const backups = await listSaveBackups(save.save_path);
      setBackups(backups);
      setShowRestoreModal(true);
    } catch {
      message.error(t('app.saves.listBackupsFailed'));
    }
  };

  const doRestore = async (backup: BackupInfo) => {
    if (!selectedSave) return;

    setRestoring(true);
    try {
      const savesDir = await openBackupDialog();
      const result = await restoreSave(backup.backup_path, savesDir);
      message.success(result.message);
      setShowRestoreModal(false);
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.restoreFailed'));
    } finally {
      setRestoring(false);
    }
  };

  const handleProfileChange = async (save: SaveInfo, profileName: string) => {
    console.log('[SVL Debug] handleProfileChange called:', {
      saveName: save.name,
      savePath: save.save_path,
      profileName,
    });
    setChangingProfile(save.save_path);
    try {
      if (profileName === '__none__') {
        console.log('[SVL Debug] Unlinking save from profile');
        await unlinkSaveFromProfile(save.save_path);
        message.success(t('app.saves.unlinkSuccess'));
      } else {
        console.log('[SVL Debug] Linking save to profile:', profileName);
        await linkSaveToProfile(save.save_path, profileName);
        message.success(t('app.saves.linkSuccess'));
      }
      loadSaves();
    } catch (err: any) {
      console.error('[SVL Debug] Profile change failed:', err);
      message.error(err?.toString() || t('app.saves.linkFailed'));
    } finally {
      setChangingProfile(null);
    }
  };

  const handleUnlink = async (save: SaveInfo) => {
    try {
      await unlinkSaveFromProfile(save.save_path);
      message.success(t('app.saves.unlinkSuccess'));
      loadSaves();
    } catch (err: any) {
      message.error(err?.toString() || t('app.saves.unlinkFailed'));
    }
  };

  const handleLaunchWithProfile = async (save: SaveInfo) => {
    if (!gamePath) {
      message.error(t('app.errors.gamePathNotFound'));
      return;
    }

    console.log('[SVL Debug] handleLaunchWithProfile called:', {
      gamePath,
      saveName: save.name,
      savePath: save.save_path,
      linkedProfile: save.linked_profile,
    });

    setLaunchingSave(save.save_path);
    try {
      const result = await launchGameWithSaveProfile(gamePath, save.save_path);
      console.log('[SVL Debug] Launch result:', result);
      if (result.success) {
        message.success(result.message);
      } else {
        message.error(result.message);
      }
    } catch (err: any) {
      console.error('[SVL Debug] Launch failed:', err);
      message.error(err?.toString() || t('app.saves.launchFailed'));
    } finally {
      setLaunchingSave(null);
    }
  };

  const formatHours = (hours: number) => {
    if (hours >= 1) {
      const h = Math.floor(hours);
      const m = Math.round((hours - h) * 60);
      return m > 0 ? `${h}h ${m}m` : `${h}h`;
    }
    return `${Math.round(hours * 60)}m`;
  };

  const getProfileSelectValue = (save: SaveInfo) => {
    if (save.linked_profile) {
      const found = profiles.find(p => p.name === save.linked_profile);
      if (found) return found.name;
    }
    return '__none__';
  };

  return (
    <div className="svl-content">
      <div className="svl-saves-page">
        <div className="svl-saves-header">
          <div className="svl-saves-title">
            <DatabaseOutlined /> {t('app.saves.title')}
          </div>
          <div className="svl-saves-actions">
            <Button
              icon={<FolderOpenOutlined />}
              onClick={handleOpenSaveLocation}
            >
              {t('app.saves.openSaveLocation')}
            </Button>
            <Button
              icon={<ClockCircleOutlined />}
              onClick={loadSaves}
              loading={loading}
            >
              {t('app.saves.refresh')}
            </Button>
          </div>
        </div>

        {saves.length === 0 && !loading ? (
          <Empty description={t('app.saves.noSaves')} />
        ) : (
          <div className="svl-saves-grid">
            {saves.map((save) => (
              <Card
                key={save.save_path}
                className="svl-save-card"
                title={
                  <div className="svl-save-card-title">
                    <UserOutlined /> {save.character_name || save.name}
                  </div>
                }
                extra={
                  <Tag color="var(--svl-primary)">{save.farm_type}</Tag>
                }
              >
                <div className="svl-save-info">
                  <div className="svl-save-info-row">
                    <span className="svl-save-info-label">
                      <HomeOutlined style={{ marginRight: 4 }} />
                      {t('app.saves.farmName')}:
                    </span>
                    <span>{save.farm_name}</span>
                  </div>
                  <div className="svl-save-info-row">
                    <span className="svl-save-info-label">
                      <ClockCircleOutlined style={{ marginRight: 4 }} />
                      {t('app.saves.playTime')}:
                    </span>
                    <span>{formatHours(save.hours_played)}</span>
                  </div>
                  <div className="svl-save-info-row">
                    <span className="svl-save-info-label">{t('app.saves.lastModified')}:</span>
                    <span>{save.last_modified}</span>
                  </div>
                  <div className="svl-save-info-row">
                    <span className="svl-save-info-label">{t('app.saves.backups')}:</span>
                    <span>{save.backup_count}</span>
                  </div>
                </div>

                {requiredMods.length > 0 && (
                  <div className="svl-save-required-mods">
                    <div className="svl-save-required-mods-title">
                      <LockOutlined style={{ marginRight: 4 }} />
                      {t('app.saves.requiredMods')}:
                    </div>
                    <div className="svl-save-required-mods-list">
                      {requiredMods.map((mod) => (
                        <Tag key={mod.unique_id} className="svl-tag-info svl-required-mod-tag">
                          <LockOutlined style={{ marginRight: 4, fontSize: 10 }} />
                          {mod.name}
                        </Tag>
                      ))}
                    </div>
                  </div>
                )}

                <div className="svl-save-profile-section">
                  <div className="svl-save-profile-label">
                    <LinkOutlined style={{ marginRight: 4 }} />
                    {t('app.saves.boundProfile')}:
                  </div>
                  <div className="svl-save-profile-selector">
                    <Select
                      size="small"
                      style={{ flex: 1, minWidth: 0 }}
                      value={getProfileSelectValue(save)}
                      onChange={(value) => handleProfileChange(save, value)}
                      loading={changingProfile === save.save_path}
                      placeholder={t('app.saves.bindProfile')}
                      options={[
                        { value: '__none__', label: t('app.saves.noBinding') },
                        ...profiles.map((p) => ({
                          value: p.name,
                          label: `${p.name} (${p.enabled_count}/${p.total_mods} ${t('app.saves.mods')})`,
                        })),
                      ]}
                    />
                    {save.linked_profile && (
                      <Tooltip title={t('app.saves.unbind')}>
                        <Button
                          size="small"
                          type="text"
                          danger
                          icon={<DisconnectOutlined />}
                          onClick={() => handleUnlink(save)}
                        />
                      </Tooltip>
                    )}
                  </div>
                  {save.linked_profile && (
                    <div className="svl-save-profile-status">
                      <Tag className="svl-tag-success">{save.linked_profile}</Tag>
                      <Tag className="svl-tag-info" style={{ marginLeft: 4 }}>
                        {profiles.find(p => p.name === save.linked_profile)?.total_mods ?? '?'} {t('app.saves.mods')}
                      </Tag>
                      {activeProfileName && save.linked_profile !== activeProfileName && (
                        <Tag className="svl-tag-warning" style={{ marginLeft: 4 }}>
                          {t('app.saves.profileMismatch')}
                        </Tag>
                      )}
                    </div>
                  )}
                </div>

                <div className="svl-save-actions">
                  <Button
                    size="small"
                    icon={<SaveOutlined />}
                    onClick={() => handleBackup(save)}
                  >
                    {t('app.saves.backup')}
                  </Button>
                  <Button
                    size="small"
                    icon={<UndoOutlined />}
                    onClick={() => handleRestore(save)}
                    disabled={save.backup_count === 0}
                  >
                    {t('app.saves.restore')}
                  </Button>
                  <Button
                    size="small"
                    type="primary"
                    icon={<PlayCircleOutlined />}
                    loading={launchingSave === save.save_path}
                    onClick={() => handleLaunchWithProfile(save)}
                    className="svl-save-launch-btn"
                  >
                    {t('app.saves.launchWithProfile')}
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        )}

        <Modal
          title={t('app.saves.backup')}
          open={showBackupModal}
          onOk={doBackup}
          onCancel={() => setShowBackupModal(false)}
          okText={t('app.saves.backup')}
          cancelText={t('app.common.cancel')}
          confirmLoading={backingUp}
        >
          <p>{t('app.saves.backupConfirm', { name: selectedSave?.name })}</p>
        </Modal>

        <Modal
          title={t('app.saves.restore')}
          open={showRestoreModal}
          onCancel={() => setShowRestoreModal(false)}
          footer={null}
          width={600}
        >
          <List
            dataSource={backups}
            renderItem={(backup) => (
              <List.Item
                actions={[
                  <Popconfirm
                    title={t('app.saves.restoreConfirm')}
                    onConfirm={() => doRestore(backup)}
                    okText={t('app.common.confirm')}
                    cancelText={t('app.common.cancel')}
                  >
                    <Button
                      size="small"
                      type="primary"
                      loading={restoring}
                    >
                      {t('app.saves.restore')}
                    </Button>
                  </Popconfirm>,
                ]}
              >
                <List.Item.Meta
                  title={backup.name}
                  description={`${backup.backup_time} · ${backup.size_mb.toFixed(2)} MB`}
                />
              </List.Item>
            )}
          />
        </Modal>
      </div>
    </div>
  );
}
