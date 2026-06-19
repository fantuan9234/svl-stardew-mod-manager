import { useState, useRef, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Upload, Button, message, Modal, Tag, List } from 'antd';
import { InboxOutlined, FolderOpenOutlined, CheckCircleOutlined, WarningOutlined, LoadingOutlined, ExclamationCircleOutlined, PlusOutlined } from '@ant-design/icons';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  installMod,
  installModFromFolder,
  checkModDependencies,
  checkInstallSourceSafety,
  type InstallResult,
  type ModDependencyCheck,
  type InstallSourceSafety,
} from '../utils/tauri-api';

const { Dragger } = Upload;

interface DropZoneProps {
  modsPath: string;
  onInstallSuccess: () => void;
}

interface InstallProgress {
  fileName: string;
  status: 'checking' | 'installing' | 'success' | 'error';
  message: string;
}

export default function DropZone({ modsPath, onInstallSuccess }: DropZoneProps) {
  const { t } = useTranslation();
  const [installing, setInstalling] = useState(false);
  const [progressList, setProgressList] = useState<InstallProgress[]>([]);
  const [showProgress, setShowProgress] = useState(false);
  const [depCheckResult, setDepCheckResult] = useState<ModDependencyCheck | null>(null);
  const [pendingFile, setPendingFile] = useState<string | null>(null);
  const [showDepModal, setShowDepModal] = useState(false);
  const [isDragOver, setIsDragOver] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const progressRef = useRef<InstallProgress[]>([]);
  const installingRef = useRef(false);
  const processFileRef = useRef<(filePath: string) => Promise<void>>(async () => {});
  const executeInstallRef = useRef<(filePaths: string[]) => Promise<void>>(async () => {});

  useEffect(() => {
    let disposed = false;
    const unlisten = getCurrentWindow().onDragDropEvent((event) => {
      if (disposed) return;
      if (event.payload.type === 'over') {
        setIsDragOver(true);
        if (!expanded) setExpanded(true);
      } else if (event.payload.type === 'leave') {
        setIsDragOver(false);
      } else if (event.payload.type === 'drop') {
        setIsDragOver(false);
        if (installingRef.current) return;
        if (!modsPath) {
          message.error(t('app.modInstall.noModsPath'));
          return;
        }
        const paths: string[] = event.payload.paths;
        const archives = paths.filter(p =>
          p.match(/\.(zip|7z|rar)$/i)
        );
        const folders = paths.filter(p =>
          !p.match(/\.(zip|7z|rar)$/i)
        );
        if (archives.length > 0) {
          for (const f of archives) {
            processFileRef.current(f);
          }
        }
        if (folders.length > 0) {
          executeInstallRef.current(folders);
        }
        if (archives.length === 0 && folders.length === 0) {
          message.warning(t('app.modInstall.unsupportedFormat'));
        }
      }
    });

    return () => {
      disposed = true;
      unlisten.then(fn => fn()).catch(() => {});
    };
  }, [modsPath]);

  const updateProgress = (index: number, update: Partial<InstallProgress>) => {
    setProgressList(prev => {
      const next = [...prev];
      next[index] = { ...next[index], ...update };
      progressRef.current = next;
      return next;
    });
  };

  const doInstall = async (filePath: string, index: number, variant: string | null = null, nexusDescription: string | null = null) => {
    const isFolder = !filePath.match(/\.(zip|7z|rar)$/i);

    if (isFolder) {
      const proceed = await confirmRiskySource(filePath);
      if (!proceed) {
        updateProgress(index, { status: 'error', message: '用户取消安装（源路径不安全）' });
        return;
      }
    }

    updateProgress(index, { status: 'installing', message: t('app.modInstall.installing') });

    try {
      let result: InstallResult;
      if (isFolder) {
        result = await installModFromFolder(filePath, modsPath);
      } else {
        result = await installMod(filePath, modsPath, null, variant, nexusDescription);
      }

      updateProgress(index, {
        status: result.success ? 'success' : 'error',
        message: result.message,
      });
    } catch (err: any) {
      updateProgress(index, {
        status: 'error',
        message: err?.toString() || t('app.modInstall.installFailed'),
      });
    }
  };

  const confirmRiskySource = async (filePath: string): Promise<boolean> => {
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

  const processFile = async (filePath: string) => {
    if (!modsPath) {
      message.error(t('app.modInstall.noModsPath'));
      return;
    }

    const isArchive = filePath.match(/\.(zip|7z)$/i);
    if (isArchive) {
      try {
        const depResult = await checkModDependencies(filePath, modsPath);
        const hasRequiredMissing = depResult.missing_dependencies.some(d => d.is_required);
        if (hasRequiredMissing) {
          setDepCheckResult(depResult);
          setPendingFile(filePath);
          setShowDepModal(true);
          return;
        }
      } catch {
        // dependency check failed, continue install anyway
      }
    }

    await executeInstall([filePath]);
  };

  const executeInstall = async (filePaths: string[], variant: string | null = null, nexusDescription: string | null = null) => {
    if (!modsPath) {
      message.error(t('app.modInstall.noModsPath'));
      return;
    }

    installingRef.current = true;
    setInstalling(true);
    setShowProgress(true);

    const initial: InstallProgress[] = filePaths.map(f => ({
      fileName: f.split(/[/\\]/).pop() || f,
      status: 'checking',
      message: t('app.modInstall.checkDeps'),
    }));
    progressRef.current = initial;
    setProgressList(initial);

    for (let i = 0; i < filePaths.length; i++) {
      updateProgress(i, { status: 'installing', message: t('app.modInstall.installing') });
      await doInstall(filePaths[i], i, variant, nexusDescription);
    }

    installingRef.current = false;
    setInstalling(false);

    const hasError = progressRef.current.some(p => p.status === 'error');
    if (!hasError && filePaths.length > 0) {
      message.success(t('app.modInstall.installSuccess'));
      await new Promise(resolve => setTimeout(resolve, 1500));
      setShowProgress(false);
      setProgressList([]);
      progressRef.current = [];
    }

    onInstallSuccess();
  };

  processFileRef.current = processFile;
  executeInstallRef.current = executeInstall;

  const handleDraggerClick = async () => {
    if (installing) return;
    if (!modsPath) {
      message.error(t('app.modInstall.noModsPath'));
      return;
    }

    const selected = await open({
      multiple: true,
      filters: [{
        name: t('app.modArchives'),
        extensions: ['zip', '7z'],
      }],
    });

    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected];
      for (const f of files) {
        await processFile(f);
      }
    }
  };

  const handleFolderSelect = async () => {
    if (installing) return;
    if (!modsPath) {
      message.error(t('app.modInstall.noModsPath'));
      return;
    }

    const selected = await open({
      directory: true,
      multiple: true,
    });

    if (selected) {
      const files = Array.isArray(selected) ? selected : [selected];
      await executeInstall(files);
    }
  };

  const handleDepContinue = async () => {
    setShowDepModal(false);
    const file = pendingFile;
    setPendingFile(null);
    setDepCheckResult(null);
    if (file) {
      await executeInstall([file]);
    }
  };

  const handleDepCancel = () => {
    setShowDepModal(false);
    setPendingFile(null);
    setDepCheckResult(null);
  };

  const handleExpandOrPick = async () => {
    if (installing) return;
    if (!modsPath) {
      message.error(t('app.modInstall.noModsPath'));
      return;
    }
    setExpanded(true);
  };

  const getStatusIcon = (status: InstallProgress['status']) => {
    switch (status) {
      case 'checking':
        return <LoadingOutlined spin style={{ color: 'var(--svl-primary-light)' }} />;
      case 'installing':
        return <LoadingOutlined spin style={{ color: 'var(--svl-warning)' }} />;
      case 'success':
        return <CheckCircleOutlined style={{ color: 'var(--svl-success)' }} />;
      case 'error':
        return <WarningOutlined style={{ color: 'var(--svl-error)' }} />;
    }
  };

  return (
    <>
      {!expanded ? (
        <div
          className={`svl-install-banner ${isDragOver ? 'svl-install-banner--dragover' : ''}`}
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            handleExpandOrPick();
          }}
        >
          <PlusOutlined className="svl-install-banner-icon" />
          <span className="svl-install-banner-text">{t('app.pages.modManager.installMod')}</span>
          <span className="svl-install-banner-hint">{t('app.pages.modManager.installBannerHint', '拖放 ZIP / 7z 文件或点击展开')}</span>
        </div>
      ) : (
        <>
          <div
            className={`svl-dropzone-wrapper ${isDragOver ? 'svl-dropzone-wrapper--dragover' : ''}`}
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              handleDraggerClick();
            }}
          >
            <Dragger
              multiple
              showUploadList={false}
              accept=".zip,.7z"
              customRequest={({ onSuccess }) => {
                if (onSuccess) onSuccess('ok');
              }}
              className={`svl-dropzone ${isDragOver ? 'svl-dropzone--dragover' : ''}`}
              disabled={installing}
              openFileDialogOnClick={false}
            >
              <p className="ant-upload-drag-icon">
                <InboxOutlined />
              </p>
              <p className="ant-upload-text">
                {installing
                  ? t('app.modInstall.installing')
                  : t('app.pages.modManager.dropzoneTitle')}
              </p>
              <p className="ant-upload-hint">
                {t('app.pages.modManager.dropzoneDesc')}
              </p>
            </Dragger>
            <button
              className="svl-collapse-btn"
              onClick={(e) => {
                e.stopPropagation();
                setExpanded(false);
              }}
              title={t('app.pages.modManager.collapseDropzone')}
            >
              ▲
            </button>
          </div>

          <div className="svl-dropzone-folder-btn">
            <Button
              icon={<FolderOpenOutlined />}
              onClick={handleFolderSelect}
              disabled={installing}
              block
            >
              {t('app.pages.modManager.importFromFolder')}
            </Button>
          </div>
        </>
      )}

      {showProgress && progressList.length > 0 && (
        <div className="svl-install-progress">
          <List
            size="small"
            dataSource={progressList}
            renderItem={(item, index) => (
              <List.Item key={index}>
                <List.Item.Meta
                  avatar={getStatusIcon(item.status)}
                  title={item.fileName}
                  description={item.message}
                />
              </List.Item>
            )}
          />
        </div>
      )}

      <Modal
        open={showDepModal}
        title={t('app.modInstall.dependencyCheck')}
        onOk={handleDepContinue}
        onCancel={handleDepCancel}
        okText={t('app.modInstall.continue')}
        cancelText={t('app.common.cancel')}
      >
        {depCheckResult && (
          <div>
            <p>
              <WarningOutlined style={{ color: 'var(--svl-warning)', marginRight: 8 }} />
              {t('app.modInstall.missingDeps', { count: depCheckResult.missing_dependencies.length })}
            </p>
            <ul style={{ paddingLeft: 20 }}>
              {depCheckResult.missing_dependencies.map((dep, i) => (
                <li key={i} style={{ marginBottom: 4 }}>
                  <code>{dep.unique_id}</code>
                  {dep.minimum_version && (
                    <Tag style={{ marginLeft: 8 }}>{dep.minimum_version}+</Tag>
                  )}
                  <Tag
                    color={dep.is_required ? 'red' : 'blue'}
                    style={{ marginLeft: 8 }}
                  >
                    {dep.is_required ? t('app.modInstall.missingRequired') : t('app.optional')}
                  </Tag>
                </li>
              ))}
            </ul>
          </div>
        )}
      </Modal>
    </>
  );
}
