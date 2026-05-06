import { useState } from 'react';
import { Modal, Button, Steps, Tag, message } from 'antd';
import { UploadOutlined, FolderOpenOutlined, CheckCircleOutlined, WarningOutlined, CloseOutlined, LoadingOutlined } from '@ant-design/icons';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/plugin-dialog';
import {
  installMod,
  installModFromFolder,
  checkModDependencies,
  type InstallResult,
  type ModDependencyCheck,
} from '../utils/tauri-api';

interface ModInstallWizardProps {
  visible: boolean;
  onClose: () => void;
  modsPath: string;
  onInstallComplete: () => void;
}

interface InstallStep {
  file: string;
  name: string;
  status: 'pending' | 'installing' | 'success' | 'error';
  message: string;
}

export default function ModInstallWizard({ visible, onClose, modsPath, onInstallComplete }: ModInstallWizardProps) {
  const { t } = useTranslation();
  const [currentStep, setCurrentStep] = useState(0);
  const [, setSelectedFiles] = useState<string[]>([]);
  const [installSteps, setInstallSteps] = useState<InstallStep[]>([]);
  const [installing, setInstalling] = useState(false);
  const [dependencyCheck, setDependencyCheck] = useState<ModDependencyCheck | null>(null);
  const [checkingDeps, setCheckingDeps] = useState(false);

  const handleSelectFiles = async () => {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'MOD Archives',
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
          steps.push({
            file,
            name: result.mod_name || fileName,
            status: 'pending',
            message: '',
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
      // auto-advance to step 2 (install)
      setCurrentStep(2);
    }
  };

  const handleInstall = async () => {
    setInstalling(true);
    setCurrentStep(2);
    const updatedSteps = [...installSteps];

    for (let i = 0; i < updatedSteps.length; i++) {
      updatedSteps[i] = { ...updatedSteps[i], status: 'installing' };
      setInstallSteps([...updatedSteps]);

      try {
        const isFolder = !updatedSteps[i].file.match(/\.(zip|7z)$/i);
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

    console.log('[ModInstallWizard] install complete, calling onInstallComplete, modsPath:', modsPath);
    await new Promise(resolve => setTimeout(resolve, 800));
    onInstallComplete();
  };

  const handleClose = () => {
    setSelectedFiles([]);
    setInstallSteps([]);
    setDependencyCheck(null);
    setCheckingDeps(false);
    setCurrentStep(0);
    onClose();
  };

  const hasRequiredMissing = dependencyCheck?.missing_dependencies?.some(d => d.is_required);

  return (
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
                          {dep.is_required ? t('app.modInstall.missingRequired') : 'Optional'}
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
                onClick={handleInstall}
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
                  onClick={handleInstall}
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
                  {step.status === 'success' && <Tag color="success">{t('app.modInstall.success')}</Tag>}
                  {step.status === 'error' && <Tag color="error">{t('app.modInstall.failed')}</Tag>}
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
  );
}
