import { useState } from 'react';
import { Modal, Button, Switch, message } from 'antd';
import { FolderOpenOutlined, DatabaseOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';

interface ModBackupConfirmModalProps {
  visible: boolean;
  modName: string;
  modUniqueId: string;
  modVersion: string;
  _defaultBackupDir: string;
  onCancel: () => void;
  onConfirm: (customBackupDir: string | null) => void;
  onSkipBackup: () => void;
}

export default function ModBackupConfirmModal({
  visible,
  modName,
  modUniqueId,
  modVersion,
  onCancel,
  onConfirm,
  onSkipBackup,
}: ModBackupConfirmModalProps) {
  const { t } = useTranslation();
  const [useCustomDir, setUseCustomDir] = useState(false);
  const [customDir, setCustomDir] = useState('');
  const [selectingDir, setSelectingDir] = useState(false);

  const handleSelectDir = async () => {
    setSelectingDir(true);
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      if (selected) {
        setCustomDir(selected as string);
      }
    } finally {
      setSelectingDir(false);
    }
  };

  const handleConfirm = () => {
    if (useCustomDir && !customDir) {
      message.warning(t('app.modBackup.selectCustomDir'));
      return;
    }
    onConfirm(useCustomDir ? customDir : null);
  };

  const handleClose = () => {
    setUseCustomDir(false);
    setCustomDir('');
    setSelectingDir(false);
    onCancel();
  };

  return (
    <Modal
      open={visible}
      onCancel={handleClose}
      footer={null}
      width={480}
      centered
    >
      <div style={{ padding: '8px 0' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 16 }}>
          <DatabaseOutlined style={{ fontSize: 28, color: '#1890ff' }} />
          <div>
            <div style={{ fontSize: 16, fontWeight: 600, color: '#f0e6d3' }}>{t('app.modBackup.title')}</div>
            <div style={{ fontSize: 12, color: '#8a7d6b' }}>{t('app.modBackup.desc')}</div>
          </div>
        </div>

        <div style={{ padding: '12px 16px', background: 'var(--svl-bg-card)', borderRadius: 8, marginBottom: 16 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
            <span style={{ color: '#a09880', fontSize: 12 }}>{t('app.modBackup.modName')}</span>
            <span style={{ color: 'var(--svl-text-primary)', fontSize: 12 }}>{modName}</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
            <span style={{ color: '#a09880', fontSize: 12 }}>Unique ID</span>
            <span style={{ color: 'var(--svl-text-primary)', fontSize: 12, fontFamily: 'monospace' }}>{modUniqueId}</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: '#a09880', fontSize: 12 }}>{t('app.modBackup.version')}</span>
            <span style={{ color: 'var(--svl-text-primary)', fontSize: 12 }}>{modVersion}</span>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 12 }}>
          <span style={{ color: '#a09880', fontSize: 13 }}>{t('app.modBackup.useCustomDir')}</span>
          <Switch checked={useCustomDir} onChange={setUseCustomDir} size="small" />
        </div>

        {useCustomDir && (
          <div style={{ marginBottom: 16 }}>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                type="text"
                readOnly
                value={customDir || t('app.modBackup.selectDirPlaceholder')}
                style={{
                  flex: 1,
                  padding: '8px 12px',
                  background: 'var(--svl-bg-primary)',
                  border: '1px solid var(--svl-border)',
                  borderRadius: 6,
                  color: 'var(--svl-text-primary)',
                  fontSize: 12,
                  fontFamily: 'monospace',
                }}
              />
              <Button
                icon={<FolderOpenOutlined />}
                onClick={handleSelectDir}
                loading={selectingDir}
                size="small"
              >
                {t('app.modBackup.browse')}
              </Button>
            </div>
          </div>
        )}

        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end', marginTop: 16 }}>
          <Button onClick={onSkipBackup} style={{ color: 'var(--svl-text-muted)' }}>
            {t('app.modBackup.skip')}
          </Button>
          <Button onClick={handleClose}>{t('app.common.cancel')}</Button>
          <Button type="primary" onClick={handleConfirm} style={{ background: 'var(--svl-primary)', borderColor: 'var(--svl-primary)' }}>
            <DatabaseOutlined />
            {t('app.modBackup.confirmBackup')}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
