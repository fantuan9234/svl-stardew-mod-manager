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

interface ParsedLogError {
  mod_name: string;
  error_type: string;
  raw_line: string;
  solution: string;
  severity: string;
  missing_dep_id?: string;
  missing_dep_name?: string;
}

interface ParseSmapiLogResult {
  errors: ParsedLogError[];
  log_path: string;
  has_errors: boolean;
  log_not_found: boolean;
  smapi_not_installed: boolean;
}

export default function LogViewer() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<ParseSmapiLogResult | null>(null);
  const [downloading, setDownloading] = useState<string | null>(null);

  const checkLog = async () => {
    setLoading(true);
    try {
      const res = await invoke<ParseSmapiLogResult>('parse_smapi_log', { logPath: null });
      setResult(res);
    } catch {
      setResult(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    checkLog();
  }, []);

  const handleDownload = async (err: ParsedLogError) => {
    setDownloading(err.mod_name);
    try {
      const uniqueId = err.missing_dep_id || err.mod_name;
      const result = await invoke<NexusLinkResult>('get_nexus_link', {
        uniqueId: uniqueId,
        modName: err.mod_name
      });
      await openUrl(result.url);
    } catch {
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
      case 'VersionMismatch':
      case 'DllLoadFailed':
      case 'ModuleError':
        return 'error';
      case 'UpdateAvailable':
        return 'warning';
      default:
        return 'default';
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'error':
        return 'error';
      case 'warn':
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

  if (!result) {
    return (
      <div style={{ padding: '24px' }}>
        <Alert
          message={t('app.log.noErrors')}
          type="info"
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

  if (result.smapi_not_installed) {
    return (
      <div style={{ padding: '24px' }}>
        <Alert
          message={t('app.log.smapiNotInstalled', 'SMAPI 未安装')}
          type="warning"
          showIcon
          style={{ marginBottom: 16 }}
        />
        <Button icon={<ReloadOutlined />} onClick={checkLog}>
          {t('app.log.refresh')}
        </Button>
      </div>
    );
  }

  if (result.log_not_found || !result.has_errors) {
    return (
      <div style={{ padding: '24px' }}>
        <Alert
          message={result.log_not_found
            ? t('app.log.logNotFound', '未找到 SMAPI 日志文件，请先运行一次游戏')
            : t('app.log.noErrors')}
          type={result.log_not_found ? 'info' : 'success'}
          showIcon
          style={{ marginBottom: 16 }}
        />
        <Space>
          <Button icon={<ReloadOutlined />} onClick={checkLog}>
            {t('app.log.refresh')}
          </Button>
          <Button icon={<FolderOpenOutlined />} onClick={handleOpenLogFolder}>
            {t('app.log.openLogFolder')}
          </Button>
        </Space>
      </div>
    );
  }

  const updateErrors = result.errors.filter(e => e.error_type === 'UpdateAvailable');
  const otherErrors = result.errors.filter(e => e.error_type !== 'UpdateAvailable');

  const realErrorCount = otherErrors.length;
  const infoCount = updateErrors.length;

  return (
    <div style={{ padding: '24px' }}>
      <Alert
        message={realErrorCount > 0
          ? t('app.log.errorsDetected', { count: realErrorCount })
          : infoCount > 0
            ? t('app.log.infoDetected', { count: infoCount })
            : t('app.log.noErrors')
        }
        type={realErrorCount > 0 ? 'error' : infoCount > 0 ? 'warning' : 'success'}
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
        {otherErrors.map((err, idx) => (
          <Alert
            key={idx}
            message={
              <Space>
                <strong>{err.mod_name}</strong>
                <Tag color={getErrorTypeColor(err.error_type)}>
                  {t(`app.log.errorTypes.${err.error_type}`, err.error_type)}
                </Tag>
                <Tag color={getSeverityColor(err.severity)}>
                  {err.severity}
                </Tag>
              </Space>
            }
            description={
              <div>
                <div style={{ marginBottom: 8, color: 'var(--svl-text-muted)', fontSize: 12 }}>
                  {err.raw_line}
                </div>
                <div style={{ marginBottom: 8, whiteSpace: 'pre-line' }}>{err.solution}</div>
                {err.error_type === 'MissingDependency' && (
                  <Button
                    size="small"
                    icon={downloading === err.mod_name ? <Spin size="small" /> : <DownloadOutlined />}
                    loading={downloading === err.mod_name}
                    onClick={() => handleDownload(err)}
                  >
                    {t('app.log.downloadNexus')}
                  </Button>
                )}
              </div>
            }
            type={err.severity === 'error' ? 'error' : 'warning'}
            showIcon
          />
        ))}

        {updateErrors.length > 0 && (
          <Alert
            type="info"
            showIcon
            message={
              <Space>
                <Tag color="warning">
                  {t(`app.log.errorTypes.UpdateAvailable`, 'UpdateAvailable')}
                </Tag>
                <strong>{updateErrors.length} {t('app.log.errorTypes.UpdateAvailable', 'UpdateAvailable')}</strong>
              </Space>
            }
            description={
              <div>
                <p style={{ marginBottom: 8, lineHeight: 1.6 }}>
                  {updateErrors.map((e, i) => (
                    <span key={i}>
                      {e.mod_name && e.mod_name !== 'Unknown' ? <strong>{e.mod_name}</strong> : e.solution}
                      {i < updateErrors.length - 1 && '、'}
                    </span>
                  ))}
                </p>
              </div>
            }
          />
        )}


      </Space>
    </div>
  );
}
