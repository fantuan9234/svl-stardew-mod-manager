import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Alert, Modal, Button, Spin, Empty, Tag, Space, Progress, message } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { CheckCircleOutlined, CloseCircleOutlined, ToolOutlined, DownloadOutlined } from '@ant-design/icons';

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

interface FixDetail {
  mod_name: string;
  error_type: string;
  action: string;
  success: boolean;
  message: string;
}

interface FixResult {
  total: number;
  fixed: number;
  failed: number;
  details: FixDetail[];
}

interface LogParserProps {
  isOpen: boolean;
  onClose: () => void;
  smapiInstalled?: boolean;
  onFixComplete?: () => void;
}

const ERROR_TYPE_KEYS: Record<string, string> = {
  MissingDependency: 'app.logParser.errorTypeMissingDep',
  MissingDll: 'app.logParser.errorTypeMissingDll',
  FailedLoading: 'app.logParser.errorTypeFailedLoading',
  UpdateAvailable: 'app.logParser.errorTypeUpdateAvailable',
  ModuleError: 'app.logParser.errorTypeModuleError',
  VersionMismatch: 'app.logParser.errorTypeVersionMismatch',
  DllLoadFailed: 'app.logParser.errorTypeDllLoadFailed',
  UnknownError: 'app.logParser.errorTypeUnknown',
  BrokenMod: 'app.logParser.errorTypeBrokenMod',
  AbandonedMod: 'app.logParser.errorTypeAbandonedMod',
  ObsoleteMod: 'app.logParser.errorTypeObsoleteMod',
  NeedsWorkaround: 'app.logParser.errorTypeNeedsWorkaround',
};

const SEVERITY_COLOR_MAP: Record<string, 'error' | 'warning' | 'info'> = {
  Error: 'error',
  Warning: 'warning',
  Info: 'info',
};

const SEVERITY_TAG_CLASS: Record<string, string> = {
  Error: 'svl-tag-error',
  Warning: 'svl-tag-warning',
  Info: 'svl-tag-info',
};

const FIXABLE_ERROR_TYPES = ['MissingDependency', 'BrokenMod', 'AbandonedMod', 'ObsoleteMod'];

export default function LogParser({ isOpen, onClose, smapiInstalled, onFixComplete }: LogParserProps) {
  const { t } = useTranslation();
  const [result, setResult] = useState<ParseSmapiLogResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [fixing, setFixing] = useState(false);
  const [fixProgress, setFixProgress] = useState<number>(0);
  const [fixCurrentMod, setFixCurrentMod] = useState<string>('');
  const [fixResultVisible, setFixResultVisible] = useState(false);
  const [fixResult, setFixResult] = useState<FixResult | null>(null);
  const [fixingIndex, setFixingIndex] = useState<number>(-1);

  const fetchLog = useCallback(async () => {
    setLoading(true);
    try {
      const res = await invoke<ParseSmapiLogResult>('parse_smapi_log', {
        logPath: null,
      });
      setResult(res);
    } catch (err) {
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

  useEffect(() => {
    if (!isOpen) return;

    const unlisten = listen('fix-progress', (event) => {
      const data = event.payload as any;
      if (data.mod_name) {
        setFixCurrentMod(data.mod_name);
      }
      if (data.status === 'success' || data.status === 'failed') {
        setFixProgress((prev) => prev + 1);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isOpen]);

  const handleRefresh = () => {
    setResult(null);
    fetchLog();
  };

  const handleOpenLogFolder = async () => {
    if (!result?.log_path) return;
    try {
      await revealItemInDir(result.log_path);
    } catch (err) {
    }
  };

  const handleFixAll = async () => {
    if (!result || result.errors.length === 0) return;

    const fixableErrors = result.errors.filter((e) => FIXABLE_ERROR_TYPES.includes(e.error_type));
    if (fixableErrors.length === 0) {
      message.warning(t('app.logParser.noFixableErrors'));
      return;
    }

    const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!apiKey) {
      message.error(t('app.logParser.needApiKey'));
      return;
    }

    setFixing(true);
    setFixProgress(0);
    setFixCurrentMod('');

    try {
      const fixRes = await invoke<FixResult>('fix_all_log_errors', {
        errors: fixableErrors,
        apiKey,
      });
      setFixResult(fixRes);
      setFixResultVisible(true);
      if (onFixComplete) {
        onFixComplete();
      }
    } catch (err) {
      message.error(String(err));
    } finally {
      setFixing(false);
    }
  };

  const handleFixSingle = async (error: ParsedLogError, index: number) => {
    const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!apiKey) {
      message.error(t('app.logParser.needApiKey'));
      return;
    }

    setFixingIndex(index);
    try {
      const detail = await invoke<FixDetail>('fix_single_log_error', {
        error,
        apiKey,
      });
      message.success(detail.message);
      handleRefresh();
    } catch (err) {
      message.error(String(err));
    } finally {
      setFixingIndex(-1);
    }
  };

  if (!isOpen) return null;

  const getTypeLabel = (errorType: string) => {
    return t(ERROR_TYPE_KEYS[errorType] || ERROR_TYPE_KEYS.UnknownError);
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
    const fixableCount = result.errors.filter(e => FIXABLE_ERROR_TYPES.includes(e.error_type)).length;

    return (
      <div className="svl-log-parser-errors">
        <div className="svl-log-parser-summary">
          <span>{t('app.logParser.errorCount', { count: result.errors.length })}</span>
          {errorCount > 0 && <Tag className="svl-tag-error" style={{ marginLeft: 8 }}>{errorCount} {t('app.logParser.errors')}</Tag>}
          {warningCount > 0 && <Tag className="svl-tag-warning" style={{ marginLeft: 4 }}>{warningCount} {t('app.logParser.warnings')}</Tag>}
          {fixableCount > 0 && (
            <Tag color="blue" style={{ marginLeft: 4 }}>{fixableCount} {t('app.logParser.fixable')}</Tag>
          )}
        </div>

        {fixing && (
          <Alert
            type="info"
            showIcon
            message={t('app.logParser.fixing', { mod: fixCurrentMod })}
            description={
              <Progress
                percent={Math.round((fixProgress / result.errors.length) * 100)}
                status="active"
              />
            }
            style={{ marginBottom: 12 }}
          />
        )}

        {(() => {
          const updateErrors = result.errors.filter(e => e.error_type === 'UpdateAvailable');
          const otherErrors = result.errors.filter(e => e.error_type !== 'UpdateAvailable');

          return (
            <>
              {otherErrors.map((error, index) => {
                const alertType = getAlertType(error.severity);
                const typeLabel = getTypeLabel(error.error_type);
                const isFixable = FIXABLE_ERROR_TYPES.includes(error.error_type);
                return (
                  <Alert
                    key={index}
                    type={alertType}
                    showIcon
                    message={
                      <span>
                        <Tag className={SEVERITY_TAG_CLASS[error.severity] || 'svl-tag-default'} style={{ marginRight: 6 }}>
                          {typeLabel}
                        </Tag>
                        {error.mod_name && error.mod_name !== 'Unknown' && (
                          <strong>{error.mod_name}</strong>
                        )}
                        {isFixable && (
                          <Tag color="blue" style={{ marginLeft: 6, fontSize: 11 }}>
                            {t('app.logParser.fixable')}
                          </Tag>
                        )}
                        {isFixable && (
                          <Button
                            type="link"
                            size="small"
                            icon={<DownloadOutlined />}
                            loading={fixingIndex === index}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleFixSingle(error, index);
                            }}
                            style={{ marginLeft: 4, padding: '0 4px', fontSize: 12 }}
                          >
                            {fixingIndex === index ? t('app.logParser.downloading') : t('app.logParser.downloadFix')}
                          </Button>
                        )}
                      </span>
                    }
                    description={
                      <div className="svl-log-error-detail">
                        <p className="svl-log-error-solution" style={{ whiteSpace: 'pre-line', lineHeight: 1.8 }}>{error.solution}</p>
                        <details className="svl-log-raw-details">
                          <summary>{t('app.logParser.rawLog')}</summary>
                          <pre className="svl-log-raw-code">{error.raw_line}</pre>
                        </details>
                      </div>
                    }
                  />
                );
              })}

              {updateErrors.length > 0 && (
                <Alert
                  type="info"
                  showIcon
                  message={
                    <span>
                      <Tag className="svl-tag-info" style={{ marginRight: 6 }}>
                        {getTypeLabel('UpdateAvailable')}
                      </Tag>
                      <strong>{t('app.logParser.errorTypeUpdateAvailable')} ({updateErrors.length})</strong>
                    </span>
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
                      <details className="svl-log-raw-details">
                        <summary>{t('app.logParser.rawLog')}</summary>
                        {updateErrors.map((e, i) => (
                          <pre key={i} className="svl-log-raw-code">{e.raw_line}</pre>
                        ))}
                      </details>
                    </div>
                  }
                />
              )}
            </>
          );
        })()}
      </div>
    );
  };

  const fixableCount = result?.errors.filter(e => FIXABLE_ERROR_TYPES.includes(e.error_type)).length || 0;

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
              disabled={loading || fixing}
            >
              {loading ? t('app.logParser.analyzing') : t('app.logParser.refreshButton')}
            </button>
            {fixableCount > 0 && (
              <Button
                type="primary"
                icon={<ToolOutlined />}
                onClick={handleFixAll}
                loading={fixing}
                disabled={fixing}
                style={{ marginLeft: 8 }}
              >
                {fixing ? t('app.logParser.fixing', { mod: fixCurrentMod }) : t('app.logParser.fixAll', { count: fixableCount })}
              </Button>
            )}
            {result?.log_path && result.log_path.includes('Mods') && (
              <>
                <button
                  className="svl-log-parser-open-btn"
                  onClick={handleOpenLogFolder}
                  disabled={fixing}
                >
                  {t('app.logParser.openModsFolder')}
                </button>
              </>
            )}
          </div>

          {renderContent()}
        </div>

        <Modal
          title={t('app.logParser.fixResultTitle')}
          open={fixResultVisible}
          onCancel={() => {
            setFixResultVisible(false);
            handleRefresh();
          }}
          footer={[
            <Button key="refresh" type="primary" onClick={() => {
              setFixResultVisible(false);
              handleRefresh();
            }}>
              {t('app.logParser.refreshAndClose')}
            </Button>,
          ]}
          width={600}
        >
          {fixResult && (
            <div>
              <Space style={{ marginBottom: 16 }}>
                <Tag color="success">{t('app.logParser.fixSuccess', { count: fixResult.fixed })}</Tag>
                {fixResult.failed > 0 && <Tag color="error">{t('app.logParser.fixFailed', { count: fixResult.failed })}</Tag>}
              </Space>
              <div style={{ maxHeight: 300, overflow: 'auto' }}>
                {fixResult.details.map((detail, i) => (
                  <div key={i} style={{ marginBottom: 8, padding: 8, background: '#1a1a1a', borderRadius: 4 }}>
                    <Space>
                      {detail.success ? (
                        <CheckCircleOutlined style={{ color: '#52c41a' }} />
                      ) : (
                        <CloseCircleOutlined style={{ color: '#ff4d4f' }} />
                      )}
                      <strong>{detail.mod_name}</strong>
                      <Tag style={{ fontSize: 11 }}>{detail.action}</Tag>
                    </Space>
                    <p style={{ margin: '4px 0 0', fontSize: 12, color: '#999', whiteSpace: 'pre-line' }}>
                      {detail.message}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          )}
        </Modal>
      </div>
    </div>
  );
}
