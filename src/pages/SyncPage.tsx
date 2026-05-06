import { useState, useEffect, useCallback, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { message, Select, Button, Modal, Input, Tag } from 'antd';
import { ExportOutlined, CloudUploadOutlined, FileZipOutlined, UploadOutlined } from '@ant-design/icons';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  profileList,
  checkSmapiStatus,
  exportProfileToZip,
  importModpackFromZip,
  importModpackFromFolder,
  type ProfileListItem,
} from '../utils/tauri-api';

const LOAD_TIMEOUT_MS = 10000;

export default function SyncPage() {
  const { t } = useTranslation();
  const [gamePath, setGamePath] = useState<string>('');
  const [profiles, setProfiles] = useState<ProfileListItem[]>([]);
  const [profilesLoaded, setProfilesLoaded] = useState(false);
  const [profilesError, setProfilesError] = useState<string | null>(null);

  const [exportModpackProfile, setExportModpackProfile] = useState<string | null>(null);
  const [exportingModpack, setExportingModpack] = useState(false);

  const [importModpackFile, setImportModpackFile] = useState<string | null>(null);
  const [importModpackFileName, setImportModpackFileName] = useState<string>('');
  const [importingModpack, setImportingModpack] = useState(false);
  const [modpackNameModalVisible, setModpackNameModalVisible] = useState(false);
  const [newModpackName, setNewModpackName] = useState('');
  const [isDragging, setIsDragging] = useState(false);

  const dropZoneRef = useRef<HTMLDivElement>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    let disposed = false;
    const unlisten = getCurrentWindow().onDragDropEvent((event) => {
      if (disposed) return;
      
      if (event.payload.type === 'over') {
        setIsDragging(true);
      } else if (event.payload.type === 'leave') {
        setIsDragging(false);
      } else if (event.payload.type === 'drop') {
        setIsDragging(false);
        
        const paths: string[] = event.payload.paths;
        console.log('[SyncPage] Received dropped paths:', paths);
        
        for (const droppedPath of paths) {
          const isFolder = !droppedPath.match(/\.(zip|7z|rar)$/i);
          
          if (isFolder) {
            setImportModpackFile(droppedPath);
            setImportModpackFileName(droppedPath.split(/[/\\]/).pop() || 'folder');
            setNewModpackName('');
            setModpackNameModalVisible(true);
          } else if (droppedPath.endsWith('.zip')) {
            setImportModpackFile(droppedPath);
            setImportModpackFileName(droppedPath.split(/[/\\]/).pop() || 'file.zip');
            setNewModpackName('');
            setModpackNameModalVisible(true);
          } else {
            message.error(t('app.sync.importModpack.invalidFormat'));
          }
        }
      }
    });

    return () => {
      disposed = true;
      unlisten.then(fn => fn()).catch(() => {});
    };
  }, [t]);

  useEffect(() => {
    checkSmapiStatus()
      .then((info) => {
        if (info.game_path) {
          setGamePath(info.game_path);
        } else {
          setProfilesLoaded(true);
          setProfilesError(t('app.sync.export.noGamePath'));
        }
      })
      .catch(() => {
        setProfilesLoaded(true);
        setProfilesError(t('app.sync.export.noGamePath'));
      });
  }, [t]);

  const loadProfiles = useCallback(async () => {
    if (!gamePath) return;

    setProfilesLoaded(false);
    setProfilesError(null);

    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    timeoutRef.current = setTimeout(() => {
      setProfilesLoaded(true);
      setProfilesError(t('app.sync.loadTimeout'));
    }, LOAD_TIMEOUT_MS);

    try {
      const list = await profileList(gamePath);
      setProfiles(list);
      setProfilesLoaded(true);
      setProfilesError(null);
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    } catch (err: any) {
      setProfiles([]);
      setProfilesLoaded(true);
      setProfilesError(err?.toString() || t('app.sync.profilesLoadFailed'));
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    }
  }, [gamePath, t]);

  useEffect(() => {
    if (gamePath) {
      loadProfiles();
    }
  }, [gamePath, loadProfiles]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  const handleExportModpack = async () => {
    if (!exportModpackProfile) {
      message.warning(t('app.sync.exportModpack.selectProfile'));
      return;
    }
    if (!gamePath) {
      message.error(t('app.sync.exportModpack.noGamePath'));
      return;
    }

    try {
      setExportingModpack(true);
      const result = await exportProfileToZip(exportModpackProfile, gamePath);
      message.success(t('app.sync.exportModpack.success', { count: result.mod_count, path: result.zip_path }));
    } catch (err: any) {
      const msg = err?.toString() || '';
      if (msg.includes('Cancelled') || msg.includes('cancelled')) {
        return;
      }
      message.error(msg || t('app.sync.exportModpack.failed'));
    } finally {
      setExportingModpack(false);
    }
  };

  const handleSelectModpackFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Modpacks', extensions: ['zip'] }]
      });
      
      if (selected && typeof selected === 'string') {
        console.log('[SyncPage] Selected modpack file via dialog:', selected);
        setImportModpackFile(selected);
        setImportModpackFileName(selected.split(/[/\\]/).pop() || 'file.zip');
        setNewModpackName('');
        setModpackNameModalVisible(true);
      }
    } catch (err) {
      console.error('[SyncPage] Failed to select file:', err);
      message.error(t('app.sync.importModpack.selectFailed'));
    }
  };

  const handleConfirmImportModpack = async () => {
    if (!newModpackName.trim()) {
      message.warning(t('app.sync.importModpack.nameRequired'));
      return;
    }
    if (!importModpackFile) {
      message.error(t('app.sync.importModpack.noFile'));
      return;
    }
    if (!gamePath) {
      message.error(t('app.sync.importModpack.noGamePath'));
      return;
    }

    try {
      setImportingModpack(true);
      setModpackNameModalVisible(false);
      
      const isFolder = !importModpackFile.match(/\.(zip|7z|rar)$/i);
      let result;
      
      if (isFolder) {
        result = await importModpackFromFolder(
          importModpackFile,
          newModpackName.trim(),
          gamePath
        );
      } else {
        result = await importModpackFromZip(
          importModpackFile,
          newModpackName.trim(),
          gamePath
        );
      }
      
      message.success(t('app.sync.importModpack.success', { count: result.mod_count, name: result.profile_name }));
      setImportModpackFile(null);
      setImportModpackFileName('');
      setNewModpackName('');
      await loadProfiles();
    } catch (err: any) {
      message.error(err?.toString() || t('app.sync.importModpack.failed'));
      setModpackNameModalVisible(false);
    } finally {
      setImportingModpack(false);
    }
  };

  const profileOptions = profiles.map((p) => ({
    value: p.name,
    label: `${p.name} (${p.enabled_count}/${p.total_mods} ${t('app.sync.mods')})`,
  }));

  const getNotFoundContent = () => {
    if (profilesError) {
      return profilesError;
    }
    if (!profilesLoaded) {
      return t('app.sync.export.loading');
    }
    if (profiles.length === 0) {
      return t('app.sync.export.noProfiles');
    }
    return t('app.sync.export.noProfiles');
  };

  return (
    <div className="svl-content">
      <div className="svl-sync-page-card">
        <div className="svl-sync-page-header">
          <div className="svl-sync-page-icon">
            <ExportOutlined />
          </div>
          <div className="svl-sync-page-title">
            {t('app.sync.modpack.title')}
          </div>
        </div>
        <div className="svl-sync-page-body">
          <div className="svl-sync-tab-content">
            {/* Export Section */}
            <div style={{ marginBottom: 40 }}>
              <h3 style={{ marginBottom: 16 }}>{t('app.sync.exportModpack.title')}</h3>
              <p className="svl-sync-tab-desc">{t('app.sync.exportModpack.description')}</p>
              <div className="svl-sync-export-form">
                <div className="svl-sync-form-field">
                  <label>{t('app.sync.exportModpack.profileLabel')}</label>
                  <Select
                    style={{ width: '100%', maxWidth: 400 }}
                    placeholder={t('app.sync.exportModpack.profilePlaceholder')}
                    value={exportModpackProfile}
                    onChange={setExportModpackProfile}
                    options={profileOptions}
                    notFoundContent={getNotFoundContent()}
                    loading={!profilesLoaded}
                  />
                </div>
                <Button
                  className="svl-sync-export-btn"
                  type="primary"
                  icon={<ExportOutlined />}
                  onClick={handleExportModpack}
                  loading={exportingModpack}
                  disabled={!exportModpackProfile}
                >
                  {exportingModpack ? t('app.sync.exportModpack.exporting') : t('app.sync.exportModpack.exportButton')}
                </Button>
              </div>
            </div>

            {/* Import Section */}
            <div style={{ borderTop: '1px solid var(--ant-color-border)', paddingTop: 32 }}>
              <h3 style={{ marginBottom: 16 }}>{t('app.sync.importModpack.title')}</h3>
              <p className="svl-sync-tab-desc">{t('app.sync.importModpack.description')}</p>
              
              <div
                ref={dropZoneRef}
                style={{
                  border: `2px dashed ${isDragging ? 'var(--ant-color-primary)' : 'var(--ant-color-border)'}`,
                  borderRadius: 8,
                  padding: 40,
                  textAlign: 'center',
                  backgroundColor: isDragging ? 'var(--ant-color-primary-bg)' : 'transparent',
                  transition: 'all 0.3s ease',
                  cursor: 'pointer',
                  marginBottom: 16,
                }}
                onClick={handleSelectModpackFile}
              >
                <div style={{ fontSize: 48, marginBottom: 16, color: 'var(--ant-color-primary)' }}>
                  <CloudUploadOutlined />
                </div>
                <p style={{ fontSize: 16, marginBottom: 8, color: 'var(--ant-color-text)' }}>
                  {t('app.sync.importModpack.dragText')}
                </p>
                <p style={{ fontSize: 14, color: 'var(--ant-color-text-secondary)' }}>
                  {t('app.sync.importModpack.hint')}
                </p>
              </div>

              <div style={{ textAlign: 'center' }}>
                <Button
                  type="primary"
                  icon={<UploadOutlined />}
                  onClick={handleSelectModpackFile}
                  disabled={importingModpack}
                  size="large"
                >
                  {t('app.sync.importModpack.selectButton')}
                </Button>
              </div>
              
              {importModpackFile && (
                <Tag color="blue" style={{ marginTop: 16, fontSize: 14, padding: '6px 12px' }}>
                  <FileZipOutlined style={{ marginRight: 6 }} />
                  {importModpackFileName}
                </Tag>
              )}
              
              <p style={{ fontSize: 12, color: 'var(--ant-color-text-tertiary)', marginTop: 12, textAlign: 'center' }}>
                {t('app.sync.importModpack.supportedFormats')}
              </p>
            </div>
          </div>
        </div>
      </div>

      <Modal
        title={t('app.sync.importModpack.modalTitle')}
        open={modpackNameModalVisible}
        onOk={handleConfirmImportModpack}
        onCancel={() => {
          setModpackNameModalVisible(false);
          setImportModpackFile(null);
        }}
        confirmLoading={importingModpack}
        okText={t('app.sync.importModpack.modalOk')}
        cancelText={t('common.cancel')}
      >
        <p>{t('app.sync.importModpack.modalDesc')}</p>
        <Input
          value={newModpackName}
          onChange={(e) => setNewModpackName(e.target.value)}
          placeholder={t('app.sync.importModpack.modalPlaceholder')}
          onPressEnter={handleConfirmImportModpack}
          autoFocus
          size="large"
        />
      </Modal>
    </div>
  );
}
