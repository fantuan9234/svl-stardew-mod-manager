import { useState } from 'react';
import { Modal, Button, Tag, Spin, message, Progress } from 'antd';
import { DownloadOutlined, CheckCircleOutlined, CloseCircleOutlined, ExclamationCircleOutlined, LinkOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import {
  scanAllMissingDependencies,
  autoInstallMissingDependency,
  type MissingDependency,
  type DependencyScanResult,
} from '../utils/tauri-api';

interface Props {
  open: boolean;
  onClose: () => void;
  modsPath: string;
  apiKey: string;
  onInstallComplete: () => void;
}

type DepInstallStatus = 'pending' | 'installing' | 'success' | 'failed' | 'no_nexus_id';

interface DepWithStatus extends MissingDependency {
  status: DepInstallStatus;
  statusMessage?: string;
}

export default function DependencyResolver({ open, onClose, modsPath, apiKey, onInstallComplete }: Props) {
  const { t } = useTranslation();
  const [scanning, setScanning] = useState(false);
  const [scanResult, setScanResult] = useState<DependencyScanResult | null>(null);
  const [deps, setDeps] = useState<DepWithStatus[]>([]);
  const [batchInstalling, setBatchInstalling] = useState(false);
  const [progress, setProgress] = useState({ current: 0, total: 0 });

  const handleScan = async () => {
    if (!modsPath) {
      message.error(t('app.depResolver.noModsPath'));
      return;
    }
    setScanning(true);
    setScanResult(null);
    setDeps([]);
    try {
      const result = await scanAllMissingDependencies(modsPath);
      setScanResult(result);
      setDeps(result.missing_dependencies.map(d => ({
        ...d,
        status: d.nexus_mod_id ? 'pending' : 'no_nexus_id',
      })));
    } catch (e: any) {
      message.error(t('app.depResolver.scanFailed', { error: String(e) }));
    } finally {
      setScanning(false);
    }
  };

  const handleInstallOne = async (index: number) => {
    const dep = deps[index];
    if (!dep.nexus_mod_id) {
      message.warning(t('app.depResolver.noNexusId', { name: dep.display_name }));
      return;
    }
    if (!apiKey) {
      message.error(t('app.depResolver.noApiKey'));
      return;
    }

    setDeps(prev => prev.map((d, i) => i === index ? { ...d, status: 'installing' } : d));

    try {
      const result = await autoInstallMissingDependency(
        dep.unique_id,
        dep.nexus_mod_id,
        modsPath,
        apiKey,
      );
      setDeps(prev => prev.map((d, i) => i === index ? {
        ...d,
        status: result.success ? 'success' : 'failed',
        statusMessage: result.message,
      } : d));
      if (result.success) {
        onInstallComplete();
      }
    } catch (e: any) {
      setDeps(prev => prev.map((d, i) => i === index ? {
        ...d,
        status: 'failed',
        statusMessage: String(e),
      } : d));
    }
  };

  const handleBatchInstall = async () => {
    const installable = deps.filter(d => d.status === 'pending' && d.nexus_mod_id);
    if (installable.length === 0) {
      message.info(t('app.depResolver.noInstallable'));
      return;
    }
    if (!apiKey) {
      message.error(t('app.depResolver.noApiKey'));
      return;
    }

    setBatchInstalling(true);
    setProgress({ current: 0, total: installable.length });

    for (let i = 0; i < deps.length; i++) {
      if (deps[i].status !== 'pending' || !deps[i].nexus_mod_id) continue;

      setDeps(prev => prev.map((d, j) => j === i ? { ...d, status: 'installing' } : d));

      try {
        const result = await autoInstallMissingDependency(
          deps[i].unique_id,
          deps[i].nexus_mod_id,
          modsPath,
          apiKey,
        );
        setDeps(prev => prev.map((d, j) => j === i ? {
          ...d,
          status: result.success ? 'success' : 'failed',
          statusMessage: result.message,
        } : d));
        if (result.success) {
          onInstallComplete();
        }
      } catch (e: any) {
        setDeps(prev => prev.map((d, j) => j === i ? {
          ...d,
          status: 'failed',
          statusMessage: String(e),
        } : d));
      }

      setProgress(prev => ({ ...prev, current: prev.current + 1 }));
    }

    setBatchInstalling(false);
    message.success(t('app.depResolver.batchComplete'));
  };

  const getStatusIcon = (status: DepInstallStatus) => {
    switch (status) {
      case 'pending': return <ExclamationCircleOutlined style={{ color: '#faad14' }} />;
      case 'installing': return <Spin size="small" />;
      case 'success': return <CheckCircleOutlined style={{ color: '#52c41a' }} />;
      case 'failed': return <CloseCircleOutlined style={{ color: '#ff4d4f' }} />;
      case 'no_nexus_id': return <ExclamationCircleOutlined style={{ color: '#999' }} />;
    }
  };

  const getStatusTag = (dep: DepWithStatus) => {
    switch (dep.status) {
      case 'pending': return <Tag color="warning">{t('app.depResolver.pending')}</Tag>;
      case 'installing': return <Tag color="processing">{t('app.depResolver.installing')}</Tag>;
      case 'success': return <Tag color="success">{t('app.depResolver.success')}</Tag>;
      case 'failed': return <Tag color="error">{t('app.depResolver.failed')}</Tag>;
      case 'no_nexus_id': return <Tag color="default">{t('app.depResolver.manual')}</Tag>;
    }
  };

  const requiredDeps = deps;
  const installedCount = deps.filter(d => d.status === 'success').length;

  return (
    <Modal
      title={t('app.depResolver.title')}
      open={open}
      onCancel={onClose}
      width={720}
      footer={[
        <Button key="close" onClick={onClose}>{t('app.depResolver.close')}</Button>,
        !scanResult && !scanning && (
          <Button key="scan" type="primary" loading={scanning} onClick={handleScan}>
            {t('app.depResolver.scan')}
          </Button>
        ),
        scanResult && !scanning && (
          <Button key="rescan" type="default" onClick={handleScan}>
            {t('app.depResolver.rescan')}
          </Button>
        ),
        scanResult && scanResult.total_missing > 0 && (
          <Button
            key="batch"
            type="primary"
            icon={<DownloadOutlined />}
            loading={batchInstalling}
            onClick={handleBatchInstall}
            disabled={deps.filter(d => d.status === 'pending' && d.nexus_mod_id).length === 0}
          >
            {t('app.depResolver.batchInstall', { count: deps.filter(d => d.status === 'pending' && d.nexus_mod_id).length })}
          </Button>
        ),
      ].filter(Boolean)}
    >
      {!scanResult && !scanning && (
        <div className="svl-dep-resolver-empty">
          <ExclamationCircleOutlined style={{ fontSize: 48, color: '#faad14', marginBottom: 16 }} />
          <p>{t('app.depResolver.scanHint')}</p>
          <Button type="primary" loading={scanning} onClick={handleScan}>
            {t('app.depResolver.scan')}
          </Button>
        </div>
      )}

      {scanning && (
        <div className="svl-dep-resolver-loading">
          <Spin size="large" />
          <p>{t('app.depResolver.scanning')}</p>
        </div>
      )}

      {scanResult && !scanning && (
        <div className="svl-dep-resolver-result">
          <div className="svl-dep-resolver-summary">
            <span>{t('app.depResolver.summary', {
              total: scanResult.total_installed,
              missing: scanResult.total_missing,
              installed: installedCount,
            })}</span>
            {batchInstalling && (
              <Progress
                percent={Math.round((progress.current / progress.total) * 100)}
                size="small"
                style={{ width: 200, marginLeft: 16 }}
              />
            )}
          </div>

          {scanResult.total_missing === 0 ? (
            <div className="svl-dep-resolver-all-good">
              <CheckCircleOutlined style={{ fontSize: 48, color: '#52c41a', marginBottom: 16 }} />
              <p>{t('app.depResolver.allSatisfied')}</p>
            </div>
          ) : (
            <>
              {requiredDeps.length > 0 && (
                <div className="svl-dep-section">
                  <h4 className="svl-dep-section-title">
                    {t('app.depResolver.requiredDeps')}
                    <Tag color="error" style={{ marginLeft: 8 }}>
                      {requiredDeps.filter(d => d.status !== 'success').length} {t('app.depResolver.missing')}
                    </Tag>
                  </h4>
                  <div className="svl-dep-list">
                    {requiredDeps.map((dep) => {
                      const realIndex = deps.indexOf(dep);
                      return (
                        <div key={dep.unique_id} className={`svl-dep-item svl-dep-${dep.status}`}>
                          <div className="svl-dep-item-left">
                            {getStatusIcon(dep.status)}
                            <div className="svl-dep-item-info">
                              <span className="svl-dep-name">{dep.display_name}</span>
                              <span className="svl-dep-id">{dep.unique_id}</span>
                              {dep.minimum_version && (
                                <Tag style={{ marginLeft: 4 }}>v{dep.minimum_version}+</Tag>
                              )}
                              {dep.required_by.length > 0 && (
                                <span className="svl-dep-required-by">
                                  {t('app.depResolver.requiredBy', { mods: dep.required_by.join(', ') })}
                                </span>
                              )}
                            </div>
                          </div>
                          <div className="svl-dep-item-right">
                            {getStatusTag(dep)}
                            {dep.status === 'pending' && dep.nexus_mod_id && (
                              <Button
                                size="small"
                                type="primary"
                                icon={<DownloadOutlined />}
                                onClick={() => handleInstallOne(realIndex)}
                              >
                                {t('app.depResolver.install')}
                              </Button>
                            )}
                            {dep.nexus_url && (
                              <a
                                href={dep.nexus_url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="svl-dep-nexus-link"
                              >
                                <LinkOutlined />
                              </a>
                            )}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

            </>
          )}
        </div>
      )}
    </Modal>
  );
}
