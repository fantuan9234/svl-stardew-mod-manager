import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Space, Spin, Empty, Typography, Input, message, Modal } from 'antd';
import { ArrowLeftOutlined, PlusOutlined, RollbackOutlined, DeleteOutlined, ExclamationCircleOutlined } from '@ant-design/icons';
import {
  listSnapshots,
  createSnapshot,
  restoreSnapshot,
  deleteSnapshot,
  detectGamePath,
  type ModSnapshotInfo,
  type ModSnapshotList,
} from '../utils/tauri-api';

const { Text, Title } = Typography;

const SnapshotIconSvg = ({ color, size = 20 }: { color: string; size?: number }) => (
  <svg width={size} height={size} viewBox="0 0 32 32" fill="none">
    <rect x="4" y="7" width="24" height="18" rx="3" fill={color} opacity="0.1" stroke={color} strokeWidth="1.5"/>
    <path d="M4 12h24" stroke={color} strokeWidth="1" opacity="0.25"/>
    <circle cx="9" cy="10" r="1.3" fill={color} opacity="0.5"/>
    <circle cx="13" cy="10" r="1.3" fill={color} opacity="0.35"/>
    <circle cx="17" cy="10" r="1.3" fill={color} opacity="0.2"/>
    <path d="M8 22l4.5-5.5 3.5 3.5 4-4L24 22H8z" fill={color} opacity="0.3"/>
    <circle cx="23" cy="14" r="1.5" fill={color} opacity="0.4"/>
  </svg>
);

export default function SnapshotManager({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [snapshots, setSnapshots] = useState<ModSnapshotInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [snapshotName, setSnapshotName] = useState('');

  const loadSnapshots = async () => {
    setLoading(true);
    try {
      const result: ModSnapshotList = await listSnapshots();
      setSnapshots(result.snapshots);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.snapshotLoadFailed'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSnapshots();
  }, []);

  const handleCreate = async () => {
    const name = snapshotName.trim();
    if (!name) {
      message.warning(t('app.toolbox.snapshotNeedName'));
      return;
    }
    setCreating(true);
    try {
      const pathInfo = await detectGamePath();
      const gamePath = pathInfo.detected_path;
      if (!gamePath) {
        message.warning(t('app.toolbox.snapshotNeedGamePath'));
        return;
      }
      await createSnapshot(gamePath + '/Mods', name);
      message.success(t('app.toolbox.snapshotCreateSuccess'));
      setSnapshotName('');
      await loadSnapshots();
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.snapshotCreateFailed'));
    } finally {
      setCreating(false);
    }
  };

  const handleRestore = async (snapshot: ModSnapshotInfo) => {
    Modal.confirm({
      title: t('app.toolbox.snapshotRestoreConfirm'),
      icon: <ExclamationCircleOutlined />,
      content: (
        <div>
          <p>{t('app.toolbox.snapshotRestoreWarning')}</p>
          <p><strong>{snapshot.snapshot_name}</strong></p>
          <p style={{ color: 'var(--svl-text-muted)', fontSize: 12 }}>{snapshot.created_at}</p>
        </div>
      ),
      okText: t('app.toolbox.snapshotRestore'),
      okType: 'primary',
      cancelText: t('common.cancel'),
      onOk: async () => {
        setRestoring(true);
        try {
          const pathInfo = await detectGamePath();
          const gamePath = pathInfo.detected_path;
          if (!gamePath) {
            message.warning(t('app.toolbox.snapshotNeedGamePath'));
            return;
          }
          await restoreSnapshot(snapshot.snapshot_name, gamePath + '/Mods');
          message.success(t('app.toolbox.snapshotRestoreSuccess'));
        } catch (e: any) {
          message.error(e?.toString() || t('app.toolbox.snapshotRestoreFailed'));
        } finally {
          setRestoring(false);
        }
      },
    });
  };

  const handleDelete = async (snapshot: ModSnapshotInfo) => {
    Modal.confirm({
      title: t('app.toolbox.snapshotDeleteConfirm'),
      icon: <ExclamationCircleOutlined />,
      content: (
        <div>
          <p>{t('app.toolbox.snapshotDeleteWarning')}</p>
          <p><strong>{snapshot.snapshot_name}</strong></p>
        </div>
      ),
      okText: t('common.delete'),
      okType: 'danger',
      cancelText: t('common.cancel'),
      onOk: async () => {
        try {
          await deleteSnapshot(snapshot.snapshot_name);
          message.success(t('app.toolbox.snapshotDeleteSuccess'));
          await loadSnapshots();
        } catch (e: any) {
          message.error(e?.toString() || t('app.toolbox.snapshotDeleteFailed'));
        }
      },
    });
  };

  return (
    <div style={{ padding: '24px 28px', maxWidth: 1200, margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 24 }}>
        <button
          onClick={onBack}
          style={{
            width: 36, height: 36, borderRadius: 10,
            border: '1px solid rgba(139,115,85,0.2)',
            background: 'rgba(61,50,37,0.5)',
            color: 'var(--svl-text-secondary)',
            cursor: 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            transition: 'all 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'rgba(61,50,37,0.8)';
            e.currentTarget.style.borderColor = 'rgba(139,115,85,0.4)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'rgba(61,50,37,0.5)';
            e.currentTarget.style.borderColor = 'rgba(139,115,85,0.2)';
          }}
        >
          <ArrowLeftOutlined />
        </button>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <SnapshotIconSvg color="#6b9ec4" size={22} />
          <Title level={4} style={{ margin: 0, fontWeight: 600 }}>{t('app.toolbox.snapshotTitle')}</Title>
        </div>
      </div>

      <div style={{
        display: 'flex',
        gap: 12,
        marginBottom: 24,
        padding: '16px 20px',
        borderRadius: 14,
        background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
        border: '1px solid rgba(139,115,85,0.12)',
        alignItems: 'center',
        flexWrap: 'wrap',
      }}>
        <Input
          placeholder={t('app.toolbox.snapshotNamePlaceholder')}
          value={snapshotName}
          onChange={(e) => setSnapshotName(e.target.value)}
          onPressEnter={handleCreate}
          style={{
            flex: 1,
            minWidth: 200,
            maxWidth: 400,
            borderRadius: 10,
            background: 'rgba(45,36,24,0.4)',
            borderColor: 'rgba(139,115,85,0.2)',
          }}
        />
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={handleCreate}
          loading={creating}
          style={{
            background: 'linear-gradient(135deg, #6b9ec4, #7db0d4)',
            border: 'none',
            borderRadius: 10,
            height: 36,
            fontWeight: 500,
          }}
        >
          {t('app.toolbox.snapshotCreate')}
        </Button>
      </div>

      {loading && (
        <div style={{ textAlign: 'center', padding: '80px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 16, color: 'var(--svl-text-muted)', fontSize: 14 }}>
            {t('app.toolbox.snapshotLoading')}
          </div>
        </div>
      )}

      {!loading && snapshots.length === 0 && (
        <Empty description={t('app.toolbox.snapshotEmpty')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}

      {!loading && snapshots.length > 0 && (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 16 }}>
          {snapshots.map((snapshot) => (
            <div
              key={snapshot.snapshot_name}
              style={{
                borderRadius: 14,
                padding: '18px 20px',
                background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
                border: '1px solid rgba(139,115,85,0.12)',
                transition: 'all 0.2s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.borderColor = 'rgba(107,158,196,0.25)';
                e.currentTarget.style.boxShadow = '0 4px 16px rgba(0,0,0,0.2)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.borderColor = 'rgba(139,115,85,0.12)';
                e.currentTarget.style.boxShadow = 'none';
              }}
            >
              <div style={{ display: 'flex', alignItems: 'flex-start', gap: 12, marginBottom: 14 }}>
                <div style={{
                  width: 40, height: 40, borderRadius: 10,
                  background: 'linear-gradient(135deg, rgba(107,158,196,0.18), rgba(107,158,196,0.06))',
                  border: '1px solid rgba(107,158,196,0.2)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  flexShrink: 0,
                }}>
                  <SnapshotIconSvg color="#6b9ec4" size={20} />
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <Text strong style={{ fontSize: 15, display: 'block', marginBottom: 2 }}>{snapshot.snapshot_name}</Text>
                  <Text type="secondary" style={{ fontSize: 12 }}>{snapshot.created_at}</Text>
                </div>
              </div>

              <div style={{
                display: 'flex',
                gap: 8,
                padding: '10px 12px',
                borderRadius: 10,
                background: 'rgba(45,36,24,0.4)',
                marginBottom: 14,
              }}>
                <div style={{ flex: 1, textAlign: 'center' }}>
                  <div style={{ fontSize: 18, fontWeight: 700, color: '#6b9ec4' }}>{snapshot.mod_count}</div>
                  <Text type="secondary" style={{ fontSize: 11 }}>{t('app.toolbox.snapshotModCount')}</Text>
                </div>
                <div style={{ width: 1, background: 'var(--svl-bg-hover)' }} />
                <div style={{ flex: 1, textAlign: 'center' }}>
                  <div style={{ fontSize: 18, fontWeight: 700, color: 'var(--svl-primary-light)' }}>{snapshot.size_mb} MB</div>
                  <Text type="secondary" style={{ fontSize: 11 }}>{t('app.toolbox.snapshotSize')}</Text>
                </div>
              </div>

              <Space style={{ width: '100%', justifyContent: 'flex-end' }}>
                <Button
                  size="small"
                  icon={<RollbackOutlined />}
                  onClick={() => handleRestore(snapshot)}
                  loading={restoring}
                  style={{
                    borderRadius: 8,
                    borderColor: 'rgba(107,158,196,0.3)',
                    color: '#6b9ec4',
                  }}
                >
                  {t('app.toolbox.snapshotRestore')}
                </Button>
                <Button
                  size="small"
                  danger
                  icon={<DeleteOutlined />}
                  onClick={() => handleDelete(snapshot)}
                  style={{ borderRadius: 8 }}
                >
                  {t('common.delete')}
                </Button>
              </Space>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
