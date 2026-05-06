import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { message, Modal, Input, Tag, Button, Empty, Table, Checkbox, Spin, Divider, Tooltip } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import {
  PlusOutlined,
  FolderOutlined,
  CheckCircleOutlined,
  EditOutlined,
  ExportOutlined,
  ImportOutlined,
  CopyOutlined,
  SwapOutlined,
  LogoutOutlined,
} from '@ant-design/icons';
import {
  profileCreate,
  profileList,
  profileGetActive,
  profileSwitch,
  profileDelete,
  updateProfileMods,
  profileGetModStates,
  profileClearActive,
  profileCopy,
  profileExport,
  profileImport,
  scanProfileMods,
  checkSmapiStatus,
  type ProfileListItem,
  type ProfileModInfo,
  type SmapiInfo,
} from '../utils/tauri-api';

export default function ProfilesPage() {
  const { t } = useTranslation();
  const [gamePath, setGamePath] = useState<string>('');
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [activeProfile, setActiveProfile] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');
  const [createModIds, setCreateModIds] = useState<Set<string>>(new Set());
  const [createAllMods, setCreateAllMods] = useState<ProfileModInfo[]>([]);
  const [createLoading, setCreateLoading] = useState(false);
  const [creating, setCreating] = useState(false);

  const [editModalOpen, setEditModalOpen] = useState(false);
  const [editProfileName, setEditProfileName] = useState('');
  const [editModStates, setEditModStates] = useState<Record<string, boolean>>({});
  const [editAllMods, setEditAllMods] = useState<ProfileModInfo[]>([]);
  const [editLoading, setEditLoading] = useState(false);
  const [editSaving, setEditSaving] = useState(false);

  const [copyModalOpen, setCopyModalOpen] = useState(false);
  const [copyFromProfile, setCopyFromProfile] = useState('');
  const [copyNewName, setCopyNewName] = useState('');
  const [copying, setCopying] = useState(false);

  const [exiting, setExiting] = useState(false);

  useEffect(() => {
    checkSmapiStatus()
      .then((info: SmapiInfo) => {
        if (info.game_path) {
          setGamePath(info.game_path);
        }
      })
      .catch(() => {});
  }, []);

  const loadProfiles = useCallback(async () => {
    if (!gamePath) return;
    setLoading(true);
    try {
      const list = await profileList(gamePath);
      setProfiles([...list]);
      const active = await profileGetActive(gamePath);
      setActiveProfile(active);
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadFailed'));
    } finally {
      setLoading(false);
    }
  }, [gamePath, t]);

  useEffect(() => {
    if (gamePath) {
      loadProfiles();
    }
  }, [gamePath, loadProfiles]);

  useEffect(() => {
    if (!gamePath) return;
    let unlisten: (() => void) | null = null;
    listen('profile-changed', () => {
      loadProfiles();
    }).then(fn => { unlisten = fn; });
    return () => { if (unlisten) unlisten(); };
  }, [gamePath, loadProfiles]);

  const handleOpenCreateModal = async () => {
    setCreateModalOpen(true);
    setCreateLoading(true);
    try {
      const mods = await scanProfileMods(gamePath);
      setCreateAllMods(mods);
      setCreateModIds(new Set(mods.map((m: ProfileModInfo) => m.unique_id)));
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadModStatesFailed'));
    } finally {
      setCreateLoading(false);
    }
  };

  const handleCloseCreateModal = () => {
    setCreateModalOpen(false);
    setNewProfileName('');
    setCreateModIds(new Set());
    setCreateAllMods([]);
  };

  const handleToggleCreateMod = (uniqueId: string) => {
    const mod = createAllMods.find(m => m.unique_id === uniqueId);
    if (mod?.is_required) return;
    setCreateModIds(prev => {
      const next = new Set(prev);
      if (next.has(uniqueId)) {
        next.delete(uniqueId);
      } else {
        next.add(uniqueId);
      }
      return next;
    });
  };

  const handleCreateSelectAll = () => {
    setCreateModIds(new Set(createAllMods.map(m => m.unique_id)));
  };

  const handleCreateDeselectAll = () => {
    const requiredIds = createAllMods.filter(m => m.is_required).map(m => m.unique_id);
    setCreateModIds(new Set(requiredIds));
  };

  const handleCreate = async () => {
    if (!newProfileName.trim()) {
      message.warning(t('app.profilesPage.nameRequired'));
      return;
    }
    setCreating(true);
    try {
      await profileCreate(gamePath, newProfileName.trim(), Array.from(createModIds));
      message.success(t('app.profilesPage.createSuccess'));
      handleCloseCreateModal();
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.createFailed'));
    } finally {
      setCreating(false);
    }
  };

  const handleSwitch = async (profileName: string) => {
    try {
      await profileSwitch(gamePath, profileName);
      message.success(t('app.profilesPage.switchSuccess', { name: profileName }));
      setActiveProfile(profileName);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.switchFailed'));
    }
  };

  const handleDelete = async (profileName: string) => {
    try {
      await profileDelete(gamePath, profileName);
      message.success(t('app.profilesPage.deleteSuccess'));
      if (activeProfile === profileName) {
        setActiveProfile(null);
      }
      await loadProfiles();
    } catch (err: any) {
      const detail = err?.toString() || '';
      message.error(`${t('app.profilesPage.deleteFailed')}${detail ? ': ' + detail : ''}`);
    }
  };

  const handleExitProfile = async () => {
    if (!activeProfile) return;
    setExiting(true);
    try {
      await profileClearActive(gamePath);
      message.success(t('app.profilesPage.exitSuccess'));
      setActiveProfile(null);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.exitFailed'));
    } finally {
      setExiting(false);
    }
  };

  const handleOpenEditModal = async (profileName: string) => {
    setEditProfileName(profileName);
    setEditModalOpen(true);
    setEditLoading(true);
    try {
      const [mods, states] = await Promise.all([
        scanProfileMods(gamePath),
        profileGetModStates(gamePath, profileName),
      ]);
      setEditAllMods(mods);
      setEditModStates({ ...states });
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.loadModStatesFailed'));
    } finally {
      setEditLoading(false);
    }
  };

  const handleToggleEditMod = (uniqueId: string) => {
    setEditModStates(prev => ({
      ...prev,
      [uniqueId]: !(prev[uniqueId] ?? false),
    }));
  };

  const handleEditSelectAll = () => {
    const next: Record<string, boolean> = {};
    editAllMods.forEach(m => { next[m.unique_id] = true; });
    setEditModStates(next);
  };

  const handleEditDeselectAll = () => {
    const next: Record<string, boolean> = {};
    editAllMods.forEach(m => { next[m.unique_id] = false; });
    setEditModStates(next);
  };

  const handleSaveEdit = async () => {
    setEditSaving(true);
    try {
      const enabledIds = Object.entries(editModStates)
        .filter(([_, enabled]) => enabled)
        .map(([id]) => id);
      await updateProfileMods(gamePath, editProfileName, enabledIds);
      message.success(t('app.profilesPage.saveModStatesSuccess'));
      setEditModalOpen(false);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.saveModStatesFailed'));
    } finally {
      setEditSaving(false);
    }
  };

  const handleOpenCopyModal = (profileName: string) => {
    setCopyFromProfile(profileName);
    setCopyNewName(`${profileName} (copy)`);
    setCopyModalOpen(true);
  };

  const handleCopy = async () => {
    if (!copyNewName.trim()) {
      message.warning(t('app.profilesPage.nameRequired'));
      return;
    }
    setCopying(true);
    try {
      await profileCopy(gamePath, copyFromProfile, copyNewName.trim());
      message.success(t('app.profilesPage.createSuccess'));
      setCopyModalOpen(false);
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.profilesPage.createFailed'));
    } finally {
      setCopying(false);
    }
  };

  const handleExport = async (profileName: string) => {
    try {
      const selected = await open({
        title: t('app.profiles.exportProfile'),
        defaultPath: `${profileName}.svl_profile`,
        filters: [{ name: 'Profile', extensions: ['svl_profile', 'json'] }],
      });
      if (selected) {
        await profileExport(gamePath, profileName, selected as string);
        message.success(t('app.profiles.exportSuccess'));
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.profiles.exportFailed'));
    }
  };

  const handleImport = async () => {
    try {
      const selected = await open({
        title: t('app.profiles.importProfile'),
        filters: [{ name: 'Profile', extensions: ['svl_profile', 'json'] }],
        multiple: false,
      });
      if (selected) {
        await profileImport(gamePath, selected as string);
        message.success(t('app.profiles.importSuccess'));
        await loadProfiles();
      }
    } catch (err: any) {
      message.error(err?.toString() || t('app.profiles.importFailed'));
    }
  };

  const columns = [
    {
      title: t('app.profilesPage.profileName'),
      dataIndex: 'name',
      key: 'name',
      render: (name: string, record: ProfileListItem) => (
        <span style={{ fontWeight: 600, color: 'var(--svl-text-primary)' }}>
          {name}
          {record.is_active && (
            <Tag icon={<CheckCircleOutlined />} color="var(--svl-primary)" style={{ marginLeft: 8 }}>
              {t('app.profilesPage.active')}
            </Tag>
          )}
          {record.is_protected && (
            <Tag color="orange" style={{ marginLeft: 4 }}>
              {t('app.profilesPage.protected')}
            </Tag>
          )}
        </span>
      ),
    },
    {
      title: t('app.profilesPage.modCount'),
      key: 'mod_count',
      width: 120,
      render: (_: any, record: ProfileListItem) => (
        <span>{record.enabled_count}/{record.total_mods}</span>
      ),
    },
    {
      title: t('app.profilesPage.createdAt'),
      dataIndex: 'created_at',
      key: 'created_at',
      width: 180,
      render: (date: string) => {
        try {
          return <span style={{ color: 'var(--svl-text-muted)', fontSize: 13 }}>{new Date(date).toLocaleString()}</span>;
        } catch {
          return <span style={{ color: 'var(--svl-text-muted)', fontSize: 13 }}>{date}</span>;
        }
      },
    },
    {
      title: t('app.profilesPage.actions'),
      key: 'actions',
      width: 320,
      render: (_: any, record: ProfileListItem) => (
        <div style={{ display: 'flex', alignItems: 'center', gap: 0, flexWrap: 'wrap' }}>
          {!record.is_active ? (
            <Tooltip title={t('app.profiles.switch')}>
              <Button
                size="small"
                type="link"
                icon={<SwapOutlined />}
                onClick={() => handleSwitch(record.name)}
              >
                {t('app.profiles.switch')}
              </Button>
            </Tooltip>
          ) : null}
          <Button
            size="small"
            type="link"
            icon={<EditOutlined />}
            onClick={() => handleOpenEditModal(record.name)}
          >
            {t('app.profiles.editMods')}
          </Button>
          <Divider type="vertical" />
          <Tooltip title={t('app.profilesPage.copy')}>
            <Button
              size="small"
              type="link"
              icon={<CopyOutlined />}
              onClick={() => handleOpenCopyModal(record.name)}
            />
          </Tooltip>
          <Tooltip title={t('app.profiles.exportProfile')}>
            <Button
              size="small"
              type="link"
              icon={<ExportOutlined />}
              onClick={() => handleExport(record.name)}
            />
          </Tooltip>
          {!record.is_protected && (
            <>
              <Divider type="vertical" />
              <Button
                size="small"
                type="link"
                danger
                onClick={() => {
                  Modal.confirm({
                    title: t('app.profilesPage.confirm'),
                    content: t('app.profiles.deleteConfirmName', { name: record.name }),
                    okText: t('app.profilesPage.confirm'),
                    cancelText: t('app.profilesPage.cancel'),
                    okButtonProps: { danger: true },
                    onOk: () => handleDelete(record.name),
                  });
                }}
              >
                {t('app.profilesPage.delete')}
              </Button>
            </>
          )}
        </div>
      ),
    },
  ];

  if (!gamePath) {
    return (
      <div className="svl-content">
        <div className="svl-profiles-page-card">
          <Empty description={t('app.profilesPage.noGamePath')} />
        </div>
      </div>
    );
  }

  return (
    <div className="svl-content">
      <div className="svl-profiles-page-card">
        <div className="svl-profiles-page-header">
          <div className="svl-profiles-page-icon">
            <FolderOutlined />
          </div>
          <div className="svl-profiles-page-title">
            {t('app.profiles.title')}
          </div>
          <div className="svl-profiles-page-actions">
            {activeProfile && (
              <>
                <Tag color="var(--svl-primary)" style={{ marginRight: 4 }}>
                  {t('app.profiles.activeProfile', { name: activeProfile })}
                </Tag>
                <Button
                  size="small"
                  icon={<LogoutOutlined />}
                  loading={exiting}
                  onClick={handleExitProfile}
                  style={{ marginRight: 8 }}
                >
                  {t('app.profiles.exitProfile')}
                </Button>
              </>
            )}
            <Button
              icon={<ImportOutlined />}
              onClick={handleImport}
              style={{ marginRight: 8 }}
            >
              {t('app.profiles.importProfile')}
            </Button>
            <Button
              className="svl-create-profile-btn"
              icon={<PlusOutlined />}
              onClick={handleOpenCreateModal}
            >
              {t('app.profiles.createNew')}
            </Button>
          </div>
        </div>

        <div className="svl-profiles-page-body">
          {loading && profiles.length === 0 ? (
            <div style={{ textAlign: 'center', padding: '60px 0' }}>
              <Spin size="large" />
            </div>
          ) : profiles.length === 0 ? (
            <Empty description={t('app.profiles.empty')} />
          ) : (
            <Table
              dataSource={profiles}
              columns={columns}
              rowKey="name"
              pagination={false}
              loading={loading}
              size="middle"
              className="svl-profiles-table"
            />
          )}
        </div>
      </div>

      <Modal
        title={t('app.profiles.createNew')}
        open={createModalOpen}
        onCancel={handleCloseCreateModal}
        width={680}
        footer={[
          <Button key="cancel" onClick={handleCloseCreateModal}>
            {t('app.profilesPage.cancel')}
          </Button>,
          <Button key="create" type="primary" onClick={handleCreate} loading={creating}>
            {t('app.profilesPage.create')}
          </Button>,
        ]}
      >
        {createLoading ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Spin size="large" />
          </div>
        ) : (
          <div style={{ marginTop: 16, display: 'flex', flexDirection: 'column', gap: 16 }}>
            <div>
              <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
                {t('app.profilesPage.profileName')}
              </div>
              <Input
                value={newProfileName}
                onChange={(e) => setNewProfileName(e.target.value)}
                placeholder={t('app.profilesPage.namePlaceholder')}
              />
            </div>
            <div>
              <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
                {t('app.profiles.selectMods')}
              </div>
              <div style={{ marginBottom: 8, display: 'flex', gap: 8 }}>
                <Button size="small" onClick={handleCreateSelectAll}>
                  {t('app.profilesPage.selectAll')}
                </Button>
                <Button size="small" onClick={handleCreateDeselectAll}>
                  {t('app.profilesPage.deselectAll')}
                </Button>
                <span style={{ marginLeft: 'auto', color: 'var(--svl-text-muted)', fontSize: 13, lineHeight: '32px' }}>
                  {t('app.profiles.totalSelected', { count: createModIds.size, total: createAllMods.length })}
                </span>
              </div>
              <div style={{ maxHeight: 300, overflow: 'auto', border: '1px solid var(--svl-border)', borderRadius: 8, padding: 8 }}>
                {createAllMods.length === 0 ? (
                  <Empty description={t('app.profilesPage.noModsFound')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
                ) : (
                  createAllMods.map((mod, idx) => (
                    <div
                      key={`${mod.unique_id}_${idx}`}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        padding: '5px 8px',
                        borderBottom: '1px solid var(--svl-border)',
                        borderRadius: 4,
                        opacity: mod.is_required ? 0.85 : 1,
                      }}
                    >
                      <Checkbox
                        checked={createModIds.has(mod.unique_id)}
                        onChange={() => handleToggleCreateMod(mod.unique_id)}
                        disabled={mod.is_required}
                      />
                      <span style={{ marginLeft: 8, flex: 1 }}>
                        {mod.name}
                        {mod.is_required && (
                          <Tag color="orange" style={{ marginLeft: 6, fontSize: 11, lineHeight: '18px' }}>
                            {t('app.modCard.requiredMod')}
                          </Tag>
                        )}
                      </span>
                      <span style={{ color: 'var(--svl-text-muted)', fontSize: 12 }}>{mod.version}</span>
                    </div>
                  ))
                )}
              </div>
            </div>
          </div>
        )}
      </Modal>

      <Modal
        title={t('app.profiles.editMods') + ' - ' + editProfileName}
        open={editModalOpen}
        onCancel={() => setEditModalOpen(false)}
        width={680}
        footer={[
          <Button key="cancel" onClick={() => setEditModalOpen(false)}>
            {t('app.profilesPage.cancel')}
          </Button>,
          <Button key="save" type="primary" onClick={handleSaveEdit} loading={editSaving}>
            {t('app.profilesPage.save')}
          </Button>,
        ]}
      >
        {editLoading ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Spin size="large" />
          </div>
        ) : (
          <div>
            <div style={{ marginBottom: 12, display: 'flex', gap: 8 }}>
              <Button size="small" onClick={handleEditSelectAll}>
                {t('app.profilesPage.selectAll')}
              </Button>
              <Button size="small" onClick={handleEditDeselectAll}>
                {t('app.profilesPage.deselectAll')}
              </Button>
              <span style={{ marginLeft: 'auto', color: 'var(--svl-text-muted)', fontSize: 13, lineHeight: '32px' }}>
                {t('app.profiles.totalSelected', { count: Object.values(editModStates).filter(Boolean).length, total: editAllMods.length })}
              </span>
            </div>
            <div style={{ maxHeight: 400, overflow: 'auto', border: '1px solid var(--svl-border)', borderRadius: 8, padding: 8 }}>
              {editAllMods.length === 0 ? (
                <Empty description={t('app.profilesPage.noModsFound')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
              ) : (
                editAllMods.map((mod, idx) => (
                  <div
                    key={`${mod.unique_id}_${idx}`}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      padding: '5px 8px',
                      borderBottom: '1px solid var(--svl-border)',
                      borderRadius: 4,
                    }}
                  >
                    <Checkbox
                      checked={editModStates[mod.unique_id] ?? false}
                      onChange={() => handleToggleEditMod(mod.unique_id)}
                      disabled={mod.is_required}
                    />
                    <span style={{ marginLeft: 8, flex: 1 }}>
                      {mod.name}
                      {mod.is_required && (
                        <Tag color="orange" style={{ marginLeft: 6, fontSize: 11, lineHeight: '18px' }}>
                          {t('app.modCard.requiredMod')}
                        </Tag>
                      )}
                    </span>
                    <span style={{ color: 'var(--svl-text-muted)', fontSize: 12 }}>{mod.version}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        )}
      </Modal>

      <Modal
        title={t('app.profilesPage.copyProfile')}
        open={copyModalOpen}
        onCancel={() => setCopyModalOpen(false)}
        onOk={handleCopy}
        confirmLoading={copying}
        okText={t('app.profilesPage.create')}
        cancelText={t('app.profilesPage.cancel')}
      >
        <div style={{ marginTop: 16 }}>
          <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
            {t('app.profilesPage.copyFrom')}: <strong>{copyFromProfile}</strong>
          </div>
          <div style={{ marginBottom: 8, color: 'var(--svl-text-secondary)' }}>
            {t('app.profilesPage.profileName')}
          </div>
          <Input
            value={copyNewName}
            onChange={(e) => setCopyNewName(e.target.value)}
            placeholder={t('app.profilesPage.namePlaceholder')}
          />
        </div>
      </Modal>
    </div>
  );
}
