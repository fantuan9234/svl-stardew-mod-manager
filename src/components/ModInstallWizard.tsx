import { useState } from 'react';
import { Modal, Button, Steps, Tag, message } from 'antd';
import { UploadOutlined, FolderOpenOutlined, CheckCircleOutlined, WarningOutlined, CloseOutlined, LoadingOutlined, ExclamationCircleOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import {
  installMod,
  installModFromFolder,
  checkModDependencies,
  checkInstallSourceSafety,
  type InstallResult,
  type ModDependencyCheck,
  type InstallSourceSafety,
} from '../utils/tauri-api';
import ModBackupConfirmModal from './ModBackupConfirmModal';

interface ModInstallWizardProps {
  visible: boolean;
  onClose: () => void;
  modsPath: string;
  onInstallComplete: () => void;
  existingMods: Array<{ unique_id: string; name: string; version: string; folder_path: string }>;
  gamePath?: string;
}

interface InstallStep {
  file: string;
  name: string;
  status: 'pending' | 'installing' | 'success' | 'error';
  message: string;
  uniqueId?: string;
  existingModIndex?: number;
}

export default function ModInstallWizard({ visible, onClose, modsPath, onInstallComplete, existingMods }: ModInstallWizardProps) {
  const { t } = useTranslation();
  const [currentStep, setCurrentStep] = useState(0);
  const [, setSelectedFiles] = useState<string[]>([]);
  const [installSteps, setInstallSteps] = useState<InstallStep[]>([]);
  const [installing, setInstalling] = useState(false);
  const [dependencyCheck, setDependencyCheck] = useState<ModDependencyCheck | null>(null);
  const [checkingDeps, setCheckingDeps] = useState(false);

  // Backup confirmation state
  const [showBackupConfirm, setShowBackupConfirm] = useState(false);
  const [backupModInfo, setBackupModInfo] = useState<{ modPath: string; name: string; uniqueId: string; version: string } | null>(null);
  const [backupQueue, setBackupQueue] = useState<Array<{ stepIndex: number; modPath: string; name: string; uniqueId: string; version: string }>>([]);

  const handleSelectFiles = async () => {
    const selected = await open({
      multiple: true,
      filters: [{
        name: t('app.modArchives'),
        extensions: ['zip', '7z'],
      }],
    });

    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected];
      setSelectedFiles(files);
      await runDependencyCheck(files);
    }
  };

  const handleSelectFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: true,
    });

    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected];
      setSelectedFiles(files);
      const steps: InstallStep[] = files.map(f => ({
        file: f,
        name: f.split(/[/\\]/).pop() || f,
        status: 'pending',
        message: '',
      }));
      setInstallSteps(steps);
      setCurrentStep(2);
    }
  };

  const runDependencyCheck = async (files: string[]) => {
    setCheckingDeps(true);
    setCurrentStep(1);

    const steps: InstallStep[] = [];
    let lastDepResult: ModDependencyCheck | null = null;

    for (const file of files) {
      const fileName = file.split(/[/\\]/).pop() || file;
      const isArchive = file.match(/\.(zip|7z)$/i);

      if (isArchive) {
        try {
          const result = await checkModDependencies(file, modsPath);
          const uniqueId = (result as any).unique_id || '';
          const existingIdx = uniqueId ? existingMods.findIndex(m => m.unique_id === uniqueId) : -1;

          steps.push({
            file,
            name: result.mod_name || fileName,
            status: 'pending',
            message: '',
            uniqueId: uniqueId || undefined,
            existingModIndex: existingIdx >= 0 ? existingIdx : undefined,
          });
          lastDepResult = result;
        } catch {
          steps.push({
            file,
            name: fileName,
            status: 'pending',
            message: '',
          });
        }
      } else {
        steps.push({
          file,
          name: fileName,
          status: 'pending',
          message: '',
        });
      }
    }

    setInstallSteps(steps);
    setDependencyCheck(lastDepResult);
    setCheckingDeps(false);

    const hasRequiredMissing = lastDepResult?.missing_dependencies?.some(d => d.is_required);
    if (hasRequiredMissing) {
      // stay at step 1, show dependency warning
    } else {
      setCurrentStep(2);
    }
  };

  // Process backup queue: backup current mod, then move to next
  const processBackupQueue = async (customDir: string | null) => {
    const queue = [...backupQueue];
    if (queue.length === 0) {
      setShowBackupConfirm(false);
      setBackupModInfo(null);
      setBackupQueue([]);
      await doInstall();
      return;
    }

    for (let i = 0; i < queue.length; i++) {
      const current = queue[i];
      try {
        await invoke('backup_mod_before_update', {
          modPath: current.modPath,
          customBackupDir: customDir || null,
        });
      } catch (err) {
        console.error('Backup failed:', err);
        message.warning(t('app.modBackup.backupFailedContinue'));
      }
    }

    setShowBackupConfirm(false);
    setBackupModInfo(null);
    setBackupQueue([]);
    await doInstall();
  };

  // Check for existing mods and build backup queue
  const checkAndBuildBackupQueue = async () => {
    const backups: Array<{ stepIndex: number; modPath: string; name: string; uniqueId: string; version: string }> = [];

    for (let i = 0; i < installSteps.length; i++) {
      const step = installSteps[i];
      if (step.existingModIndex !== undefined) {
        const existing = existingMods[step.existingModIndex];
        backups.push({
          stepIndex: i,
          modPath: existing.folder_path,
          name: existing.name,
          uniqueId: existing.unique_id,
          version: existing.version,
        });
      }
    }

    if (backups.length > 0) {
      setBackupQueue(backups);
      setBackupModInfo({
        modPath: backups[0].modPath,
        name: backups[0].name,
        uniqueId: backups[0].uniqueId,
        version: backups[0].version,
      });
      setShowBackupConfirm(true);
      return true;
    }
    return false;
  };

  const handleStartInstall = async () => {
    const hasBackup = await checkAndBuildBackupQueue();
    if (hasBackup) return;
    await doInstall();
  };

  const doInstall = async () => {
    setInstalling(true);
    setCurrentStep(2);
    const updatedSteps = [...installSteps];

    for (let i = 0; i < updatedSteps.length; i++) {
      const isFolder = !updatedSteps[i].file.match(/\.(zip|7z)$/i);
      const proceed = await confirmRiskySource(updatedSteps[i].file, isFolder);
      if (!proceed) {
        updatedSteps[i] = {
          ...updatedSteps[i],
          status: 'error',
          message: '用户取消安装（源路径不安全）',
        };
        setInstallSteps([...updatedSteps]);
        continue;
      }

      updatedSteps[i] = { ...updatedSteps[i], status: 'installing' };
      setInstallSteps([...updatedSteps]);

      try {
        let result: InstallResult;

        if (isFolder) {
          result = await installModFromFolder(updatedSteps[i].file, modsPath);
        } else {
          result = await installMod(updatedSteps[i].file, modsPath);
        }

        updatedSteps[i] = {
          ...updatedSteps[i],
          status: result.success ? 'success' : 'error',
          message: result.message,
        };
      } catch (err: any) {
        updatedSteps[i] = {
          ...updatedSteps[i],
          status: 'error',
          message: err?.toString() || t('app.modInstall.installFailed'),
        };
      }

      setInstallSteps([...updatedSteps]);
    }

    setInstalling(false);
    setCurrentStep(3);

    const hasError = updatedSteps.some(s => s.status === 'error');
    if (!hasError) {
      message.success(t('app.modInstall.installSuccess'));
    }
    onInstallComplete();
    await new Promise(resolve => setTimeout(resolve, hasError ? 800 : 1500));
    handleClose();
  };

  const confirmRiskySource = async (filePath: string, isFolder: boolean): Promise<boolean> => {
    if (!isFolder) return true;
    let safety: InstallSourceSafety | null = null;
    try {
      safety = await checkInstallSourceSafety(filePath, modsPath);
    } catch {
      return true;
    }
    if (safety.safe) return true;
    if (safety.risk === 'missing' || safety.risk === 'not_dir') {
      message.error(safety.reason);
      return false;
    }
    return new Promise<boolean>(resolve => {
      const displayName = safety.conflicting_mod_name || filePath.split(/[/\\]/).pop() || filePath;
      Modal.confirm({
        title: (
          <span>
            <ExclamationCircleOutlined style={{ color: 'var(--svl-warning)', marginRight: 8 }} />
            源文件夹位于 Mods 目录内部
          </span>
        ),
        content: (
          <div>
            <p style={{ marginBottom: 8 }}>{safety.reason}</p>
            <p style={{ marginBottom: 8 }}>
              <strong>源文件夹：</strong> <code>{displayName}</code>
            </p>
            <p style={{ marginBottom: 8 }}>
              <strong>Mods 目录：</strong> <code>{modsPath}</code>
            </p>
            {safety.conflicting_mod_name && (
              <p style={{ color: 'var(--svl-warning)', marginBottom: 8 }}>
                目标 Mods 文件夹中已存在同名文件夹 <code>{safety.conflicting_mod_name}</code>，
                安装过程会用新版替换旧版（系统已做备份，旧版会保留到 <code>.{safety.conflicting_mod_name}.svl_backup</code>，安装成功后自动清理）。
              </p>
            )}
            <p style={{ marginBottom: 0, color: 'var(--svl-text-muted)' }}>
              如果你只是想重新启用这个 MOD 而不是安装新版本，请直接关闭此窗口。
            </p>
          </div>
        ),
        okText: '我已确认，继续安装',
        cancelText: '取消',
        okButtonProps: { danger: true },
        onOk: () => resolve(true),
        onCancel: () => resolve(false),
      });
    });
  };

  const handleClose = () => {
    setSelectedFiles([]);
    setInstallSteps([]);
    setDependencyCheck(null);
    setCheckingDeps(false);
    setCurrentStep(0);
    setShowBackupConfirm(false);
    setBackupQueue([]);
    setBackupModInfo(null);
    onClose();
  };

  const hasRequiredMissing = dependencyCheck?.missing_dependencies?.some(d => d.is_required);

  return (
    <>
      <Modal
        open={visible}
        onCancel={handleClose}
        footer={null}
        width={600}
        centered
        className="svl-mod-install-wizard"
      >
        <div className="svl-wizard-header">
          <h2>{t('app.modInstall.title')}</h2>
          <Button type="text" icon={<CloseOutlined />} onClick={handleClose} />
        </div>

        <Steps
          current={currentStep}
          className="svl-wizard-steps"
          items={[
            { title: t('app.modInstall.selectFiles') },
            { title: t('app.modInstall.checkDeps') },
            { title: t('app.modInstall.installing') },
            { title: t('app.modInstall.complete') },
          ]}
        />

        <div className="svl-wizard-content">
          {currentStep === 0 && (
            <div className="svl-wizard-select">
              <p>{t('app.modInstall.selectDesc')}</p>
              <div className="svl-wizard-actions">
                <Button
                  type="primary"
                  icon={<UploadOutlined />}
                  onClick={handleSelectFiles}
                  block
                  size="large"
                >
                  {t('app.modInstall.selectArchive')}
                </Button>
                <Button
                  icon={<FolderOpenOutlined />}
                  onClick={handleSelectFolder}
                  block
                  size="large"
                >
                  {t('app.modInstall.selectFolder')}
                </Button>
              </div>
            </div>
          )}

          {currentStep === 1 && checkingDeps && (
            <div className="svl-wizard-checking">
              <div style={{ textAlign: 'center', padding: '24px 0' }}>
                <LoadingOutlined spin style={{ fontSize: 32, color: 'var(--svl-primary)' }} />
                <p style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>
                  {t('app.modInstall.checkingDeps')}
                </p>
              </div>
            </div>
          )}

          {currentStep === 1 && !checkingDeps && dependencyCheck && (
            <div className="svl-wizard-deps">
              <h3>{t('app.modInstall.dependencyCheck')}</h3>
              <div className="svl-dep-result">
                <div className="svl-dep-mod-name">
                  {dependencyCheck.mod_name}
                  <Tag>v{dependencyCheck.version}</Tag>
                </div>

                {dependencyCheck.missing_dependencies?.length > 0 ? (
                  <div className="svl-missing-deps">
                    <p>
                      <WarningOutlined style={{ color: 'var(--svl-warning)', marginRight: 8 }} />
                      {t('app.modInstall.missingDeps', { count: dependencyCheck.missing_dependencies.length })}
                    </p>
                    <ul>
                      {dependencyCheck.missing_dependencies.map((dep, i) => (
                        <li key={i}>
                          <code>{dep.unique_id}</code>
                          {dep.minimum_version && <Tag>{dep.minimum_version}+</Tag>}
                          <Tag color={dep.is_required ? 'red' : 'blue'}>
                            {dep.is_required ? t('app.modInstall.missingRequired') : t('app.optional')}
                          </Tag>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p style={{ color: 'var(--svl-success)' }}>
                    <CheckCircleOutlined style={{ marginRight: 8 }} />
                    {t('app.modInstall.noMissingDeps')}
                  </p>
                )}
              </div>

              <div className="svl-wizard-actions">
                <Button onClick={handleClose}>
                  {t('app.common.cancel')}
                </Button>
                <Button
                  type="primary"
                  onClick={handleStartInstall}
                  disabled={!!hasRequiredMissing}
                >
                  {hasRequiredMissing ? t('app.modInstall.missingRequired') : t('app.modInstall.startInstall')}
                </Button>
              </div>
            </div>
          )}

          {currentStep === 2 && (
            <div className="svl-wizard-install">
              <div className="svl-install-steps">
                {installSteps.map((step, i) => (
                  <div key={i} className={`svl-install-step ${step.status}`}>
                    <span className="svl-install-step-name">{step.name}</span>
                    {step.status === 'pending' && (
                      <span className="svl-install-step-status">—</span>
                    )}
                    {step.status === 'installing' && (
                      <span className="svl-install-step-status">
                        <LoadingOutlined spin />
                      </span>
                    )}
                    {step.status === 'success' && (
                      <span className="svl-install-step-status success">
                        <CheckCircleOutlined />
                      </span>
                    )}
                    {step.status === 'error' && (
                      <span className="svl-install-step-status error" title={step.message}>
                        <WarningOutlined />
                      </span>
                    )}
                  </div>
                ))}
              </div>

              {!installing && installSteps.every(s => s.status !== 'installing') && (
                <div className="svl-wizard-actions">
                  <Button onClick={handleClose}>
                    {t('app.common.cancel')}
                  </Button>
                  <Button
                    type="primary"
                    onClick={handleStartInstall}
                    loading={installing}
                  >
                    {t('app.modInstall.startInstall')}
                  </Button>
                </div>
              )}
            </div>
          )}

          {currentStep === 3 && (
            <div className="svl-wizard-complete">
              <CheckCircleOutlined className="svl-complete-icon" />
              <h3>{t('app.modInstall.installComplete')}</h3>
              <div className="svl-install-summary">
                {installSteps.map((step, i) => (
                  <div key={i} className={`svl-summary-item ${step.status}`}>
                    <span>{step.name}</span>
                    {step.status === 'success' && <Tag className="svl-tag-success">{t('app.modInstall.success')}</Tag>}
                    {step.status === 'error' && <Tag className="svl-tag-error">{t('app.modInstall.failed')}</Tag>}
                  </div>
                ))}
              </div>
              <div className="svl-wizard-actions">
                <Button type="primary" onClick={handleClose}>
                  {t('app.modInstall.close')}
                </Button>
              </div>
            </div>
          )}
        </div>
      </Modal>

      {/* Backup confirmation modal */}
      {backupModInfo && (
        <ModBackupConfirmModal
          visible={showBackupConfirm}
          modName={backupModInfo.name}
          modUniqueId={backupModInfo.uniqueId}
          modVersion={backupModInfo.version}
          _defaultBackupDir={modsPath || ''}
          onCancel={() => { setShowBackupConfirm(false); setBackupQueue([]); setBackupModInfo(null); doInstall(); }}
          onConfirm={async (dir) => { await processBackupQueue(dir); }}
          onSkipBackup={async () => { setShowBackupConfirm(false); setBackupQueue([]); setBackupModInfo(null); await doInstall(); }}
        />
      )}
    </>
  );
}
