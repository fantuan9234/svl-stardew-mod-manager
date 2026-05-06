import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Button, Alert, Spin, Space, Tag, message } from 'antd';
import { ReloadOutlined, FolderOpenOutlined, DownloadOutlined } from '@ant-design/icons';
import { openUrl } from '../utils/openUrl';

interface NexusLinkResult {
  url: string;
  method: string;
  mod_id: string | null;
}

interface SmapiLogError {
  mod_name: string;
  error_type: string;
  original_line: string;
  solution: string;
}

interface CheckSmapiLogResult {
  has_error: boolean;
  errors: SmapiLogError[];
  error_count: number;
}

export default function LogViewer() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<CheckSmapiLogResult | null>(null);
  const [downloading, setDownloading] = useState<string | null>(null);

  const checkLog = async () => {
    setLoading(true);
    try {
      const res = await invoke<CheckSmapiLogResult>('check_smapi_log');
      setResult(res);
    } catch {
      setResult({ has_error: false, errors: [], error_count: 0 });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkLog();
  }, []);

  const handleDownload = async (modName: string) => {
    setDownloading(modName);
    try {
      const result = await invoke<NexusLinkResult>('get_nexus_link', { 
        uniqueId: modName,
        modName: modName
      });
      await openUrl(result.url);
    } catch (err: any) {
      console.error('[LogViewer] Failed to get download URL:', err);
      message.error(t('app.modCard.openLinkFailed'));
    } finally {
      setDownloading(null);
    }
  };

  const handleOpenLogFolder = async () => {
    try {
      const appData = await invoke<string>('get_appdata_path');
      const logPath = `${appData}\\StardewValley\\ErrorLogs`;
      await invoke('open_path', { path: logPath });
    } catch {
      message.error(t('app.log.openLogFolder'));
    }
  };

  const getErrorTypeColor = (type: string) => {
    switch (type) {
      case 'MissingDependency':
      case 'MissingDll':
      case 'FailedLoading':
        return 'error';
      case 'UpdateAvailable':
        return 'warning';
      default:
        return 'default';
    }
  };

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', padding: '60px 0' }}>
        <Spin size="large" />
      </div>
    );
  }

  if (!result || result.error_count === 0) {
    return (
      <div style={{ padding: '24px' }}>
        <Alert
          message={t('app.log.noErrors')}
          type="success"
          showIcon
          style={{ marginBottom: 16 }}
        />
        <Space>
          <Button icon={<ReloadOutlined />} onClick={checkLog}>
            {t('app.log.refresh')}
          </Button>
        </Space>
      </div>
    );
  }

  return (
    <div style={{ padding: '24px' }}>
      <Alert
        message={t('app.log.errorsDetected', { count: result.error_count })}
        type="error"
        showIcon
        style={{ marginBottom: 16 }}
      />

      <div style={{ marginBottom: 16 }}>
        <Space>
          <Button icon={<ReloadOutlined />} onClick={checkLog}>
            {t('app.log.refresh')}
          </Button>
          <Button icon={<FolderOpenOutlined />} onClick={handleOpenLogFolder}>
            {t('app.log.openLogFolder')}
          </Button>
        </Space>
      </div>

      <Space direction="vertical" style={{ width: '100%' }} size="middle">
        {result.errors.map((err, idx) => (
          <Alert
            key={idx}
            message={
              <Space>
                <strong>{err.mod_name}</strong>
                <Tag color={getErrorTypeColor(err.error_type)}>
                  {t(`app.log.errorTypes.${err.error_type}`, err.error_type)}
                </Tag>
              </Space>
            }
            description={
              <div>
                <div style={{ marginBottom: 8, color: 'var(--svl-text-muted)', fontSize: 12 }}>
                  {err.original_line}
                </div>
                <div style={{ marginBottom: 8 }}>{err.solution}</div>
                {err.error_type === 'MissingDependency' && (
                  <Button
                    size="small"
                    icon={downloading === err.mod_name ? <Spin size="small" /> : <DownloadOutlined />}
                    loading={downloading === err.mod_name}
                    onClick={() => handleDownload(err.mod_name)}
                  >
                    {t('app.log.downloadNexus')}
                  </Button>
                )}
              </div>
            }
            type="error"
            showIcon
          />
        ))}
      </Space>
    </div>
  );
}
