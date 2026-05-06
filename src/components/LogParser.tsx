import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Alert, Modal, Button, Spin, Empty, Tag } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { revealItemInDir } from '@tauri-apps/plugin-opener';

interface ParsedLogError {
  mod_name: string;
  error_type: string;
  raw_line: string;
  solution: string;
  severity: string;
}

interface ParseSmapiLogResult {
  errors: ParsedLogError[];
  log_path: string;
  has_errors: boolean;
  log_not_found: boolean;
  smapi_not_installed: boolean;
}

interface LogParserProps {
  isOpen: boolean;
  onClose: () => void;
  smapiInstalled?: boolean;
}

const ERROR_TYPE_MAP: Record<string, { label: string }> = {
  MissingDependency: { label: '缺少前置' },
  MissingDll: { label: 'DLL缺失' },
  FailedLoading: { label: '加载失败' },
  UpdateAvailable: { label: '可更新' },
  ModuleError: { label: '模块错误' },
  UnknownError: { label: '未知错误' },
};

const SEVERITY_COLOR_MAP: Record<string, 'error' | 'warning' | 'info'> = {
  Error: 'error',
  Warning: 'warning',
  Info: 'info',
};

const SEVERITY_TAG_COLOR: Record<string, string> = {
  Error: 'error',
  Warning: 'warning',
  Info: 'processing',
};

export default function LogParser({ isOpen, onClose, smapiInstalled }: LogParserProps) {
  const { t } = useTranslation();
  const [result, setResult] = useState<ParseSmapiLogResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [fullLog, setFullLog] = useState<string | null>(null);
  const [fullLogLoading, setFullLogLoading] = useState(false);
  const [fullLogVisible, setFullLogVisible] = useState(false);

  const fetchLog = useCallback(async () => {
    setLoading(true);
    try {
      const res = await invoke<ParseSmapiLogResult>('parse_smapi_log', {
        logPath: null,
      });
      console.log('[LogParser] parse_smapi_log result:', res);
      setResult(res);
    } catch (err) {
      console.error('[LogParser] parse_smapi_log failed:', err);
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      fetchLog();
    }
  }, [isOpen, fetchLog]);

  const handleRefresh = () => {
    setResult(null);
    setFullLog(null);
    fetchLog();
  };

  const handleOpenFullLog = async () => {
    if (!result?.log_path) return;

    setFullLogLoading(true);
    setFullLogVisible(true);
    try {
      const content = await invoke<string>('read_log_file', {
        filePath: result.log_path,
      });
      setFullLog(content);
    } catch (err) {
      console.error('[LogParser] read_log_file failed:', err);
      setFullLog(String(err));
    } finally {
      setFullLogLoading(false);
    }
  };

  const handleOpenLogFolder = async () => {
    if (!result?.log_path) return;
    try {
      await revealItemInDir(result.log_path);
    } catch (err) {
      console.error('[LogParser] revealItemInDir failed:', err);
    }
  };

  if (!isOpen) return null;

  const getTypeLabel = (errorType: string) => {
    return ERROR_TYPE_MAP[errorType]?.label || ERROR_TYPE_MAP.UnknownError.label;
  };

  const getAlertType = (severity: string) => {
    return SEVERITY_COLOR_MAP[severity] || 'warning';
  };

  const renderContent = () => {
    if (loading) {
      return (
        <div className="svl-log-parser-loading">
          <Spin size="large" />
          <p>{t('app.logParser.analyzing')}</p>
        </div>
      );
    }

    if (!result) {
      return (
        <div className="svl-log-parser-empty">
          <Empty description={t('app.logParser.loadFailed')} />
        </div>
      );
    }

    if (result.smapi_not_installed || smapiInstalled === false) {
      return (
        <Alert
          type="warning"
          showIcon
          message={t('app.logParser.smapiNotInstalled')}
          description={t('app.logParser.smapiNotInstalledDesc')}
        />
      );
    }

    if (result.log_not_found) {
      return (
        <Alert
          type="success"
          showIcon
          message={t('app.logParser.noLogFile')}
          description={t('app.logParser.noLogFileDesc')}
        />
      );
    }

    if (!result.has_errors) {
      return (
        <Alert
          type="success"
          showIcon
          message={t('app.logParser.noErrors')}
          description={t('app.logParser.noErrorsDesc')}
        />
      );
    }

    const errorCount = result.errors.filter(e => e.severity === 'Error').length;
    const warningCount = result.errors.filter(e => e.severity === 'Warning').length;

    return (
      <div className="svl-log-parser-errors">
        <div className="svl-log-parser-summary">
          <span>{t('app.logParser.errorCount', { count: result.errors.length })}</span>
          {errorCount > 0 && <Tag color="error" style={{ marginLeft: 8 }}>{errorCount} {t('app.logParser.errors')}</Tag>}
          {warningCount > 0 && <Tag color="warning" style={{ marginLeft: 4 }}>{warningCount} {t('app.logParser.warnings')}</Tag>}
        </div>
        {result.errors.map((error, index) => {
          const alertType = getAlertType(error.severity);
          const typeLabel = getTypeLabel(error.error_type);
          return (
            <Alert
              key={index}
              type={alertType}
              showIcon
              message={
                <span>
                  <Tag color={SEVERITY_TAG_COLOR[error.severity] || 'default'} style={{ marginRight: 6 }}>
                    {typeLabel}
                  </Tag>
                  {error.mod_name && error.mod_name !== 'Unknown' && (
                    <strong>{error.mod_name}</strong>
                  )}
                </span>
              }
              description={
                <div className="svl-log-error-detail">
                  <p className="svl-log-error-solution">{error.solution}</p>
                  <details className="svl-log-raw-details">
                    <summary>{t('app.logParser.rawLog')}</summary>
                    <pre className="svl-log-raw-code">{error.raw_line}</pre>
                  </details>
                </div>
              }
            />
          );
        })}
      </div>
    );
  };

  return (
    <div className="svl-log-parser-overlay" onClick={onClose}>
      <div className="svl-log-parser-modal" onClick={(e) => e.stopPropagation()}>
        <div className="svl-log-parser-header">
          <h2>{t('app.logParser.title')}</h2>
          <button className="svl-log-parser-close" onClick={onClose}>✕</button>
        </div>

        <div className="svl-log-parser-content">
          <div className="svl-log-parser-actions">
            <button
              className="svl-log-parser-analyze-btn"
              onClick={handleRefresh}
              disabled={loading}
            >
              {loading ? t('app.logParser.analyzing') : t('app.logParser.refreshButton')}
            </button>
            {result?.log_path && (
              <>
                <button
                  className="svl-log-parser-open-btn"
                  onClick={handleOpenFullLog}
                >
                  {t('app.logParser.openFullLog')}
                </button>
                <button
                  className="svl-log-parser-open-btn"
                  onClick={handleOpenLogFolder}
                >
                  {t('app.logParser.openLogFolder')}
                </button>
              </>
            )}
          </div>

          {renderContent()}
        </div>

        <Modal
          title={t('app.logParser.fullLogTitle')}
          open={fullLogVisible}
          onCancel={() => setFullLogVisible(false)}
          footer={[
            <Button key="close" onClick={() => setFullLogVisible(false)}>
              {t('app.logParser.close')}
            </Button>,
          ]}
          width={800}
          className="svl-log-full-modal"
        >
          {fullLogLoading ? (
            <div style={{ textAlign: 'center', padding: 40 }}>
              <Spin />
            </div>
          ) : (
            <pre className="svl-log-full-content">{fullLog}</pre>
          )}
        </Modal>
      </div>
    </div>
  );
}
