import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button, message, Spin, List, Tag, Typography, Space, Popconfirm, Empty, Tooltip } from 'antd';
import { HistoryOutlined, RollbackOutlined, DeleteOutlined, SaveOutlined, FolderOpenOutlined } from '@ant-design/icons';
import { listSaveFileBackups, restoreSaveFileBackup, deleteSaveFileBackup, type SaveBackupInfo } from '../utils/advanced-features-api';

const { Text } = Typography;

interface SaveBackupManagerProps {
  visible: boolean;
  onClose: () => void;
  onSaveEditorOpen?: () => void;
}

export default function SaveBackupManager({ visible, onClose, onSaveEditorOpen }: SaveBackupManagerProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [backups, setBackups] = useState<SaveBackupInfo[]>([]);
  const [totalSize, setTotalSize] = useState(0);
  const [restoring, setRestoring] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);

  const loadBackups = useCallback(async () => {
    setLoading(true);
    try {
      const result = await listSaveFileBackups();
      setBackups(result.backups);
      setTotalSize(result.total_size_mb);
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (visible) {
      loadBackups();
    }
  }, [visible, loadBackups]);

  const handleRestore = useCallback(async (backupPath: string) => {
    setRestoring(backupPath);
    try {
      const result = await restoreSaveFileBackup(backupPath);
      if (result.success) {
        message.success(t('saveBackup.restoreSuccess', '存档已恢复') + ': ' + result.message);
      } else {
        message.error(result.message);
      }
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setRestoring(null);
    }
  }, [t]);

  const handleDelete = useCallback(async (backupPath: string) => {
    setDeleting(backupPath);
    try {
      const result = await deleteSaveFileBackup(backupPath);
      if (result.success) {
        message.success(t('saveBackup.deleteSuccess', '备份已删除'));
        loadBackups();
      } else {
        message.error(result.message);
      }
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setDeleting(null);
    }
  }, [t, loadBackups]);

  const openBackupFolder = useCallback(async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const { revealItemInDir } = await import('@tauri-apps/plugin-opener');
      const dir = await invoke<string>('get_save_backup_dir_cmd');
      if (dir) await revealItemInDir(dir);
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    }
  }, []);

  return (
    <Modal
      title={
        <Space>
          <SaveOutlined />
          {t('saveBackup.title', '存档备份')}
        </Space>
      }
      open={visible}
      onCancel={onClose}
      footer={[
        <Button key="openFolder" icon={<FolderOpenOutlined />} onClick={openBackupFolder}>
          {t('saveBackup.openFolder', '打开备份目录')}
        </Button>,
        <Button key="saveEditor" icon={<SaveOutlined />} onClick={() => {
          onClose();
          onSaveEditorOpen?.();
        }}>
          {t('saveBackup.goToSaveEditor', '去存档编辑器')}
        </Button>,
        <Button key="close" type="primary" onClick={onClose}>
          {t('common.close', '关闭')}
        </Button>,
      ]}
      width={800}
    >
      <div style={{ marginBottom: 12, padding: 12, background: '#f5f5f5', borderRadius: 4 }}>
        <Space split={<span style={{ color: '#ccc' }}>•</span>}>
          <Text strong>{t('saveBackup.total', '总计')}: {backups.length}</Text>
          <Text>{t('saveBackup.totalSize', '总大小')}: {totalSize.toFixed(2)} MB</Text>
        </Space>
      </div>

      {loading ? (
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin />
        </div>
      ) : backups.length === 0 ? (
        <Empty
          description={
            <div>
              <div>{t('saveBackup.empty', '暂无存档备份')}</div>
              <div style={{ fontSize: 12, color: '#999', marginTop: 8 }}>
                {t('saveBackup.emptyHint', '使用存档编辑器修改存档时，会自动在此创建备份')}
              </div>
            </div>
          }
        />
      ) : (
        <List
          dataSource={backups}
          renderItem={(item) => {
            const isAuto = item.source === 'save_editor';
            const character = item.character_name || t('saveBackup.unknown', '未知角色');
            const farm = item.farm_name ? ` · ${item.farm_name}` : '';
            return (
              <List.Item
                key={item.backup_path}
                actions={[
                  <Popconfirm
                    key="restore"
                    title={t('saveBackup.confirmRestore', '确定要恢复此备份吗？当前存档将被覆盖。')}
                    okText={t('common.confirm', '确定')}
                    cancelText={t('common.cancel', '取消')}
                    onConfirm={() => handleRestore(item.backup_path)}
                  >
                    <Button
                      type="link"
                      icon={<RollbackOutlined />}
                      loading={restoring === item.backup_path}
                    >
                      {t('saveBackup.restore', '恢复')}
                    </Button>
                  </Popconfirm>,
                  <Popconfirm
                    key="delete"
                    title={t('saveBackup.confirmDelete', '确定要删除此备份吗？')}
                    okText={t('common.confirm', '确定')}
                    cancelText={t('common.cancel', '取消')}
                    onConfirm={() => handleDelete(item.backup_path)}
                  >
                    <Button
                      type="link"
                      danger
                      icon={<DeleteOutlined />}
                      loading={deleting === item.backup_path}
                    >
                      {t('common.delete', '删除')}
                    </Button>
                  </Popconfirm>,
                ]}
              >
                <List.Item.Meta
                  avatar={<HistoryOutlined style={{ fontSize: 24, color: '#1890ff' }} />}
                  title={
                    <Space>
                      <Text strong>{character}</Text>
                      <Text type="secondary">{farm}</Text>
                      <Tag color={isAuto ? 'blue' : 'green'}>
                        {isAuto ? t('saveBackup.sourceAuto', '自动') : t('saveBackup.sourceManual', '手动')}
                      </Tag>
                    </Space>
                  }
                  description={
                    <div>
                      <div style={{ fontSize: 12, color: '#666' }}>
                        <Tooltip title={item.backup_path}>
                          <span>{item.backup_name}</span>
                        </Tooltip>
                      </div>
                      <div style={{ fontSize: 12, color: '#999', marginTop: 4 }}>
                        {new Date(item.created_at).toLocaleString()} · {item.size_mb.toFixed(2)} MB
                        {item.note && <span style={{ marginLeft: 8 }}>· {item.note}</span>}
                      </div>
                    </div>
                  }
                />
              </List.Item>
            );
          }}
        />
      )}
    </Modal>
  );
}
