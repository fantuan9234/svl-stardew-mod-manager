import { useState, useRef, useEffect } from 'react';
import { Typography, Button, Card, message, Progress, Space, Tag, Radio } from 'antd';
import { CloudDownloadOutlined, CheckCircleOutlined, LoadingOutlined, ReloadOutlined, GlobalOutlined, SafetyCertificateOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { check } from '@tauri-apps/plugin-updater';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import {
  checkAppUpdateFromServer,
  downloadAppUpdateFromServer,
  getUpdateServerUrl,
  getCurrentAppVersion,
  AppUpdateInfo,
  AppUpdateProgress,
} from '../utils/tauri-api';

const { Title, Text, Paragraph } = Typography;

type UpdateMode = 'server' | 'tauri';

export default function UpdateChecker() {
  const { t } = useTranslation();
  const [mode, setMode] = useState<UpdateMode>('server');
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState(0);
  const [downloadedBytes, setDownloadedBytes] = useState(0);
  const [totalBytes, setTotalBytes] = useState(0);
  const [installed, setInstalled] = useState(false);
  const [serverUrl, setServerUrl] = useState('');
  const [currentVersion, setCurrentVersion] = useState('');

  const [tauriUpdateInfo, setTauriUpdateInfo] = useState<{
    version: string;
    notes?: string;
    currentVersion: string;
    date?: string;
  } | null>(null);

  const [serverUpdateInfo, setServerUpdateInfo] = useState<AppUpdateInfo | null>(null);

  const cumulativeRef = useRef(0);
  const totalRef = useRef(0);

  useEffect(() => {
    getUpdateServerUrl().then(setServerUrl).catch(() => {});
    getCurrentAppVersion().then(setCurrentVersion).catch(() => {});
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    if (mode === 'server') {
      listen<AppUpdateProgress>('app-update-progress', (event) => {
        setDownloadedBytes(event.payload.downloaded);
        setTotalBytes(event.payload.total);
        setProgress(Math.min(Math.round(event.payload.percent), 100));
      }).then(fn => { unlisten = fn; });
    }
    return () => { unlisten?.(); };
  }, [mode]);

  const handleCheck = async () => {
    setChecking(true);
    setInstalled(false);
    setProgress(0);
    setDownloadedBytes(0);
    setTotalBytes(0);

    if (mode === 'tauri') {
      setTauriUpdateInfo(null);
      try {
        const update = await check();
        if (update) {
          setTauriUpdateInfo({
            version: update.version,
            notes: update.body,
            currentVersion: update.currentVersion,
            date: formatUpdaterDate(update.date),
          });
        } else {
          message.success(t('features.updater.upToDate', { version: currentVersion || '1.0.2' }));
        }
      } catch (err) {
        console.error('Tauri update check failed:', err);
        message.error(t('features.updater.checkFailed'));
      } finally {
        setChecking(false);
      }
    } else {
      setServerUpdateInfo(null);
      try {
        const info = await checkAppUpdateFromServer();
        setServerUpdateInfo(info);
        if (!info.has_update) {
          message.success(t('features.updater.upToDate', { version: info.current_version }));
        } else if (info.force_update) {
          message.warning(t('features.serverUpdater.forceUpdate'));
        }
      } catch (err) {
        console.error('Server update check failed:', err);
        message.error(t('features.serverUpdater.serverUnavailable'));
      } finally {
        setChecking(false);
      }
    }
  };

  const handleTauriDownloadInstall = async () => {
    setDownloading(true);
    setProgress(0);
    cumulativeRef.current = 0;
    totalRef.current = 0;
    setDownloadedBytes(0);
    setTotalBytes(0);
    try {
      const update = await check();
      if (!update) {
        message.error(t('features.updater.checkFailed'));
        setDownloading(false);
        return;
      }

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            totalRef.current = event.data.contentLength || 0;
            setTotalBytes(event.data.contentLength || 0);
            break;
          case 'Progress':
            cumulativeRef.current += event.data.chunkLength || 0;
            setDownloadedBytes(cumulativeRef.current);
            if (totalRef.current > 0) {
              setProgress(Math.min(Math.round((cumulativeRef.current / totalRef.current) * 100), 100));
            }
            break;
          case 'Finished':
            setProgress(100);
            setDownloading(false);
            setInstalling(true);
            break;
        }
      });

      setInstalling(false);
      setInstalled(true);
      message.success(t('features.updater.restartRequired'));
    } catch (err) {
      console.error('Download install failed:', err);
      setDownloading(false);
      setInstalling(false);
      message.error(t('features.updater.downloadFailed'));
    }
  };

  const handleServerDownloadInstall = async () => {
    if (!serverUpdateInfo) return;
    setDownloading(true);
    setProgress(0);
    setDownloadedBytes(0);
    setTotalBytes(0);
    try {
      const result = await downloadAppUpdateFromServer(serverUpdateInfo.download_url);
      if (result.success) {
        setDownloading(false);
        setInstalled(true);
        if (result.file_path) {
          try {
            await invoke('run_installer', { path: result.file_path });
          } catch (err: any) {
            console.error('Run installer failed:', err);
            message.error(t('features.updater.downloadFailed'));
          }
        } else {
          message.success(t('features.serverUpdater.downloadCompleteRestart'));
        }
      } else {
        setDownloading(false);
        message.error(result.message);
      }
    } catch (err: any) {
      console.error('Server download failed:', err);
      setDownloading(false);
      message.error(t('features.updater.downloadFailed'));
    }
  };

  const handleRestart = async () => {
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  };

  const isServerMode = mode === 'server';
  const hasUpdate = isServerMode ? serverUpdateInfo?.has_update : !!tauriUpdateInfo;

  return (
    <Card style={{ marginTop: 24 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <Title level={4} style={{ margin: 0 }}>
            <CloudDownloadOutlined style={{ marginRight: 8 }} />
            {t('features.updater.title')}
          </Title>
          <Text type="secondary">{t('features.updater.description')}</Text>
        </div>
        <Button
          icon={checking ? <LoadingOutlined /> : <ReloadOutlined />}
          loading={checking}
          onClick={handleCheck}
          disabled={downloading || installing}
        >
          {checking ? t('features.updater.checking') : t('features.updater.checkButton')}
        </Button>
      </div>

      <div style={{ marginTop: 16, display: 'flex', alignItems: 'center', gap: 12 }}>
        <Text type="secondary" style={{ fontSize: 12 }}>
          {t('features.serverUpdater.updateSource')}:
        </Text>
        <Radio.Group
          value={mode}
          onChange={(e) => {
            setMode(e.target.value);
            setTauriUpdateInfo(null);
            setServerUpdateInfo(null);
            setInstalled(false);
          }}
          size="small"
        >
          <Radio.Button value="server">
            <GlobalOutlined style={{ marginRight: 4 }} />
            {t('features.serverUpdater.serverMode')}
          </Radio.Button>
          <Radio.Button value="tauri">
            <SafetyCertificateOutlined style={{ marginRight: 4 }} />
            {t('features.serverUpdater.tauriMode')}
          </Radio.Button>
        </Radio.Group>
        {currentVersion && (
          <Tag style={{ marginLeft: 'auto' }}>v{currentVersion}</Tag>
        )}
      </div>

      {isServerMode && (
        <div style={{ marginTop: 16, padding: 16, background: '#fafafa', borderRadius: 8 }}>
          <Space>
            <GlobalOutlined style={{ color: '#1890ff' }} />
            <Text strong>{t('features.serverUpdater.serverAddress')}:</Text>
            <Tag color="blue" style={{ fontSize: 13, padding: '4px 10px' }}>{serverUrl}</Tag>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {t('features.serverUpdater.checkEndpoint', { endpoint: '/api/update/check?version=' })}
            </Text>
          </Space>
        </div>
      )}

      {isServerMode && hasUpdate && serverUpdateInfo && (
        <div style={{ marginTop: 20, padding: 16, background: '#fafafa', borderRadius: 8 }}>
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <div style={{ display: 'flex', gap: 16, alignItems: 'center', flexWrap: 'wrap' }}>
              <Tag color="blue">{t('features.updater.currentVersion')}: {serverUpdateInfo.current_version}</Tag>
              <Tag color="green">{t('features.updater.latestVersion')}: {serverUpdateInfo.latest_version}</Tag>
              {serverUpdateInfo.release_date && (
                <Text type="secondary">{t('features.updater.releaseDate')}: {serverUpdateInfo.release_date}</Text>
              )}
              {serverUpdateInfo.force_update && (
                <Tag color="red">{t('features.serverUpdater.forceUpdate')}</Tag>
              )}
            </div>

            {serverUpdateInfo.release_notes && (
              <div>
                <Text strong>{t('features.updater.releaseNotes')}:</Text>
                <Paragraph style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>
                  {serverUpdateInfo.release_notes}
                </Paragraph>
              </div>
            )}

            {downloading && (
              <div>
                <Text>{t('features.updater.downloading')}</Text>
                <Progress percent={progress} status="active" style={{ marginTop: 8 }} />
                {totalBytes > 0 && (
                  <Text type="secondary">
                    {t('features.updater.downloadProgress', {
                      downloaded: formatBytes(downloadedBytes),
                      total: formatBytes(totalBytes),
                    })}
                  </Text>
                )}
              </div>
            )}

            {installed && (
              <div>
                <CheckCircleOutlined style={{ color: '#52c41a', marginRight: 8 }} />
                {t('features.serverUpdater.downloadCompleteRestart')}
              </div>
            )}

            {!downloading && !installed && (
              <Button type="primary" onClick={handleServerDownloadInstall}>
                <CloudDownloadOutlined />
                {t('features.updater.downloadButton')}
              </Button>
            )}

            {installed && (
              <Button type="primary" danger onClick={handleRestart}>
                {t('features.updater.restartButton')}
              </Button>
            )}
          </Space>
        </div>
      )}

      {!isServerMode && tauriUpdateInfo && (
        <div style={{ marginTop: 20, padding: 16, background: '#fafafa', borderRadius: 8 }}>
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <div style={{ display: 'flex', gap: 16, alignItems: 'center', flexWrap: 'wrap' }}>
              <Tag color="blue">{t('features.updater.currentVersion')}: {tauriUpdateInfo.currentVersion}</Tag>
              <Tag color="green">{t('features.updater.latestVersion')}: {tauriUpdateInfo.version}</Tag>
              {tauriUpdateInfo.date && <Text type="secondary">{t('features.updater.releaseDate')}: {tauriUpdateInfo.date}</Text>}
            </div>

            {tauriUpdateInfo.notes && (
              <div>
                <Text strong>{t('features.updater.releaseNotes')}:</Text>
                <Paragraph style={{ marginTop: 8, whiteSpace: 'pre-wrap' }}>
                  {tauriUpdateInfo.notes}
                </Paragraph>
              </div>
            )}

            {downloading && (
              <div>
                <Text>{t('features.updater.downloading')}</Text>
                <Progress percent={progress} status="active" style={{ marginTop: 8 }} />
                {totalBytes > 0 && (
                  <Text type="secondary">
                    {t('features.updater.downloadProgress', {
                      downloaded: formatBytes(downloadedBytes),
                      total: formatBytes(totalBytes),
                    })}
                  </Text>
                )}
              </div>
            )}

            {installing && (
              <div>
                <LoadingOutlined style={{ marginRight: 8 }} />
                {t('features.updater.installing')}
              </div>
            )}

            {installed && (
              <div>
                <CheckCircleOutlined style={{ color: '#52c41a', marginRight: 8 }} />
                {t('features.updater.restartRequired')}
              </div>
            )}

            {!downloading && !installing && !installed && (
              <Button type="primary" onClick={handleTauriDownloadInstall}>
                <CloudDownloadOutlined />
                {t('features.updater.downloadButton')}
              </Button>
            )}

            {installed && (
              <Button type="primary" danger onClick={handleRestart}>
                {t('features.updater.restartButton')}
              </Button>
            )}
          </Space>
        </div>
      )}
    </Card>
  );
}

function formatUpdaterDate(date: string | Date | undefined): string | undefined {
  if (date == null) return undefined;
  if (typeof date === 'string') return date.split('T')[0];
  return date.toISOString().split('T')[0];
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}
