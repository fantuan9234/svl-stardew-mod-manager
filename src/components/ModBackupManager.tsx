import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button, message, Spin, List, Tag, Typography, Space, Popconfirm } from 'antd';
import { HistoryOutlined, RollbackOutlined, DeleteOutlined, CloudDownloadOutlined } from '@ant-design/icons';
import { backupModBeforeUpdate, listModBackups, restoreModFromBackup, deleteModBackup, type ModBackupInfo } from '../utils/advanced-features-api';

const { Text } = Typography;

interface ModBackupManagerProps {
  visible: boolean;
  onClose: () => void;
  modPath: string;
  modUniqueId: string;
  modName: string;
  onRestore: () => void;
}

export default function ModBackupManager({ visible, onClose, modPath, modUniqueId, modName, onRestore }: ModBackupManagerProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [backups, setBackups] = useState<ModBackupInfo[]>([]);
  const [totalSize, setTotalSize] = useState(0);
  const [restoring, setRestoring] = useState<string | null>(null);

  const loadBackups = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listModBackups(modUniqueId);
      setBackups(result.backups);
      setTotalSize(result.total_size_mb);
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setLoading(false);
    }
  }, [modUniqueId, t]);

  useEffect(() => {
    if (visible && modUniqueId) {
      loadBackups();
    }
  }, [visible, modUniqueId, loadBackups]);

  const handleBackup = useCallback(async () => {
    try {
      const result = await backupModBeforeUpdate(modPath);
      message.success(result.message);
      loadBackups();
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    }
  }, [modPath, loadBackups, t]);

  const handleRestore = useCallback(async (backupPath: string) => {
    setRestoring(backupPath);
    try {
      const result = await restoreModFromBackup(backupPath, modPath);
      message.success(result.message);
      onRestore();
      loadBackups();
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setRestoring(null);
    }
  }, [modPath, onRestore, loadBackups, t]);

  const handleDelete = useCallback(async (backupPath: string) => {
    try {
      const result = await deleteModBackup(backupPath);
      message.success(result.message);
      loadBackups();
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    }
  }, [loadBackups, t]);

  return (
    <Modal
      title={t('features.backupManager.title', { modName })}
      open={visible}
      onCancel={onClose}
      width={700}
      footer={
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <Text type="secondary">
            {t('features.backupManager.totalSize', { size: totalSize.toFixed(2) })}
          </Text>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button onClick={onClose}>{t('app.common.close')}</Button>
            <Button
              type="primary"
              icon={<CloudDownloadOutlined />}
              onClick={handleBackup}
              loading={loading}
            >
              {t('features.backupManager.createBackup')}
            </Button>
          </div>
        </div>
      }
    >
      <Spin spinning={loading}>
        {backups.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <HistoryOutlined style={{ fontSize: 48, color: 'var(--svl-text-tertiary)' }} />
            <p style={{ marginTop: 16, color: 'var(--svl-text-secondary)' }}>
              {t('features.backupManager.noBackups')}
            </p>
          </div>
        ) : (
          <List
            dataSource={backups}
            size="small"
            renderItem={(backup) => (
              <List.Item
                actions={[
                  <Popconfirm
                    key="restore"
                    title={t('features.backupManager.confirmRestore')}
                    onConfirm={() => handleRestore(backup.backup_path)}
                  >
                    <Button
                      icon={<RollbackOutlined />}
                      size="small"
                      type="primary"
                      loading={restoring === backup.backup_path}
                    >
                      {t('features.backupManager.restore')}
                    </Button>
                  </Popconfirm>,
                  <Popconfirm
                    key="delete"
                    title={t('features.backupManager.confirmDelete')}
                    onConfirm={() => handleDelete(backup.backup_path)}
                  >
                    <Button icon={<DeleteOutlined />} size="small" danger>
                      {t('app.common.delete')}
                    </Button>
                  </Popconfirm>,
                ]}
              >
                <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
                  <Space>
                    <Text strong>{backup.backup_name}</Text>
                    <Tag>{backup.version}</Tag>
                  </Space>
                  <Space>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {new Date(backup.created_at).toLocaleString()}
                    </Text>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      {backup.size_mb.toFixed(2)} MB
                    </Text>
                  </Space>
                </div>
              </List.Item>
            )}
          />
        )}
      </Spin>
    </Modal>
  );
}
