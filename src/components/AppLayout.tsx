import { useState, useEffect, useRef, lazy, Suspense, startTransition } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { MinusOutlined, BorderOutlined, CloseOutlined, SwitcherOutlined, LoadingOutlined } from '@ant-design/icons';
import { Badge, Modal, Button, Typography, Progress, Tag, message, Spin } from 'antd';
import { CloudDownloadOutlined, CheckCircleOutlined, SyncOutlined, FolderOutlined, SaveOutlined, CoffeeOutlined, SearchOutlined, GlobalOutlined, ToolOutlined } from '@ant-design/icons';
import chickenImg from '../assets/chicken.png';
import HomeModal from './HomeModal';
import { openUrl } from '../utils/openUrl';
import { downloadAppUpdateFromServer, AppUpdateInfo, AppUpdateProgress } from '../utils/tauri-api';

const ModManager = lazy(() => import('../pages/ModManager'));
const NexusModBrowser = lazy(() => import('../pages/NexusModBrowser'));
const ProfilesPage = lazy(() => import('../pages/ProfilesPage'));
const SavesManager = lazy(() => import('../pages/SavesManager'));
const SyncPage = lazy(() => import('../pages/SyncPage'));
const Settings = lazy(() => import('../pages/Settings'));
const DonatePage = lazy(() => import('../pages/DonatePage'));
const LogViewer = lazy(() => import('../pages/LogViewer'));
const Toolbox = lazy(() => import('../pages/Toolbox'));

function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function getTauriWindow() {
  if (!isTauriEnvironment()) return null;
  const { getCurrentWindow } = await import('@tauri-apps/api/window');
  return getCurrentWindow();
}

const pageMap: Record<string, React.LazyExoticComponent<React.ComponentType<any>>> = {
  '/mod-manager': ModManager,
  '/nexus-browser': NexusModBrowser,
  '/profiles': ProfilesPage,
  '/saves': SavesManager,
  '/sync': SyncPage,
  '/settings': Settings,
  '/donate': DonatePage,
  '/log-viewer': LogViewer,
  '/toolbox': Toolbox,
};

const navItems = [
  { key: '/mod-manager', icon: '', label: 'app.nav.mods' },
  { key: '/nexus-browser', icon: <SearchOutlined />, label: 'app.nav.nexusBrowser' },
  { key: '/profiles', icon: <FolderOutlined />, label: 'app.nav.profiles' },
  { key: '/saves', icon: <SaveOutlined />, label: 'app.nav.saves' },
  { key: '/sync', icon: <SyncOutlined />, label: 'app.nav.sync' },
  { key: '/toolbox', icon: <ToolOutlined />, label: 'app.nav.toolbox' },
  { key: '/settings', icon: '⚙️', label: 'app.nav.settings' },
  { key: '/donate', icon: <CoffeeOutlined />, label: 'sidebar.donate' },
  { key: '/log-viewer', icon: '', label: '', hidden: true },
];

const { Title, Text, Paragraph } = Typography;

export default function AppLayout() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const [isMaximized, setIsMaximized] = useState(false);
  const [errorCount, setErrorCount] = useState(0);
  const logCheckUnlistenRef = useRef<(() => void) | null>(null);

  const [forceUpdateInfo, setForceUpdateInfo] = useState<AppUpdateInfo | null>(null);
  const [forceDownloading, setForceDownloading] = useState(false);
  const [forceProgress, setForceProgress] = useState(0);
  const [forceDownloadedBytes, setForceDownloadedBytes] = useState(0);
  const [forceTotalBytes, setForceTotalBytes] = useState(0);
  const [forceInstalled, setForceInstalled] = useState(false);
  const [forceInstallerPath, setForceInstallerPath] = useState<string | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<AppUpdateInfo>('app-update-available', (event) => {
      const info = event.payload;
      if (info.force_update) {
        setForceUpdateInfo(info);
      } else {
        message.info(t('features.serverUpdater.newVersionAvailable', { version: info.latest_version }));
      }
    }).then(fn => { unlisten = fn; });

    let progressUnlisten: (() => void) | null = null;
    listen<AppUpdateProgress>('app-update-progress', (event) => {
      setForceDownloadedBytes(event.payload.downloaded);
      setForceTotalBytes(event.payload.total);
      setForceProgress(Math.min(Math.round(event.payload.percent), 100));
    }).then(fn => { progressUnlisten = fn; });

    return () => {
      unlisten?.();
      progressUnlisten?.();
    };
  }, []);

  useEffect(() => {
    if (location.pathname === '/') {
      startTransition(() => navigate('/mod-manager', { replace: true }));
    }
  }, [location.pathname, navigate]);

  const checkLog = async () => {
    try {
      const result = await invoke<any>('parse_smapi_log', { logPath: null });
      if (result.has_errors && result.errors.length > 0) {
        setErrorCount(result.errors.length);
      } else {
        setErrorCount(0);
      }
    } catch {}
  };

  useEffect(() => {
    let resizeUnlisten: (() => void) | null = null;
    getTauriWindow().then((window) => {
      if (!window) return;
      window.isMaximized().then(setIsMaximized).catch(() => {});

      window.onResized(async () => {
        try {
          const maximized = await window.isMaximized();
          setIsMaximized(maximized);
        } catch {}
      }).then((unlisten) => {
        resizeUnlisten = unlisten;
      });
    }).catch(() => {});

    return () => {
      resizeUnlisten?.();
    };
  }, []);

  useEffect(() => {
    checkLog();
    const interval = setInterval(checkLog, 30000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;
    listen('game-exit-errors', (event) => {
      const payload = event.payload as { has_errors: boolean; error_count: number; errors: string[] };
      if (payload.has_errors && payload.error_count > 0) {
        setErrorCount(payload.error_count);
        message.warning(t('app.log.gameExitErrors', { count: payload.error_count }));
      }
    }).then(fn => { unlistenFn = fn; });
    return () => { unlistenFn?.(); };
  }, []);

  useEffect(() => {
    let unlistenFn: (() => void) | null = null;

    const setupListener = async () => {
      try {
        if (logCheckUnlistenRef.current) {
          logCheckUnlistenRef.current();
          logCheckUnlistenRef.current = null;
        }

        const unlisten = await listen('mod-install-progress', (event) => {
          const payload = event.payload as { step: string; mod_name?: string; message?: string };
          if (payload.step === 'done') {
            setTimeout(() => {
              checkLog();
            }, 1500);
          }
        });

        unlistenFn = () => {
          unlisten();
        };
        logCheckUnlistenRef.current = unlistenFn;
      } catch {}
    };

    setupListener();

    return () => {
      if (logCheckUnlistenRef.current) {
        logCheckUnlistenRef.current();
        logCheckUnlistenRef.current = null;
      }
    };
  }, []);

  const handleMinimize = () => {
    getTauriWindow().then((window) => {
      window?.minimize().catch(() => {});
    }).catch(() => {});
  };

  const handleToggleMaximize = () => {
    getTauriWindow().then((window) => {
      window?.toggleMaximize().catch(() => {});
    }).catch(() => {});
  };

  const handleClose = () => {
    getTauriWindow().then((window) => {
      window?.close().catch(() => {});
    }).catch(() => {});
  };

  const handleLogClick = () => {
    startTransition(() => navigate('/log-viewer'));
  };

  const handleForceDownload = async () => {
    if (!forceUpdateInfo) return;
    setForceDownloading(true);
    setForceProgress(0);
    setForceDownloadedBytes(0);
    setForceTotalBytes(0);
    try {
      const result = await downloadAppUpdateFromServer(forceUpdateInfo.download_url);
      if (result.success) {
        setForceDownloading(false);
        setForceInstalled(true);
        if (result.file_path) {
          setForceInstallerPath(result.file_path);
        }
      } else {
        setForceDownloading(false);
        message.error(result.message || t('features.updater.downloadFailed'), 5);
      }
    } catch (err: any) {
      console.error('Force download failed:', err);
      setForceDownloading(false);
      const errMsg = err?.message || err?.toString() || t('features.updater.downloadFailed');
      message.error(errMsg, 5);
    }
  };

  const handleForceRestart = async () => {
    if (!forceInstallerPath) return;
    try {
      await invoke('run_installer', { path: forceInstallerPath });
    } catch {
      message.error('Failed to start installer');
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  return (
    <div style={{ display: 'flex', height: '100vh', width: '100vw', overflow: 'hidden', flexDirection: 'column' }}>
      <div className="svl-custom-titlebar" data-tauri-drag-region>
        <div className="svl-titlebar-logo" data-tauri-drag-region>
          SVL
        </div>
        <div className="svl-titlebar-controls">
          <button className="svl-titlebar-btn" onClick={handleMinimize} title={t('app.window.minimize')}>
            <MinusOutlined />
          </button>
          <button className="svl-titlebar-btn" onClick={handleToggleMaximize} title={isMaximized ? t('app.window.restore') : t('app.window.maximize')}>
            {isMaximized ? <SwitcherOutlined /> : <BorderOutlined />}
          </button>
          <button className="svl-titlebar-btn svl-titlebar-btn-close" onClick={handleClose} title={t('app.window.close')}>
            <CloseOutlined />
          </button>
        </div>
      </div>

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        <aside className="svl-sidebar">
          <div className="svl-logo">
            <img
              src="/images/stardew-farm-screenshot.jpg"
              alt={t('app.altFarm')}
              className="svl-logo-image"
              style={{
                width: '180px',
                height: '100px',
                objectFit: 'cover',
                borderRadius: '12px',
                imageRendering: 'auto',
                filter: 'drop-shadow(0 2px 4px rgba(0,0,0,0.3))',
                marginBottom: '8px',
              }}
            />
            <div className="svl-logo-text">{t('app.brandName')}</div>
          </div>

          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 8,
              padding: '10px 14px',
              borderRadius: '8px',
              cursor: 'pointer',
              color: '#c49a3b',
              fontWeight: 600,
              fontSize: 14,
              background: 'rgba(196, 154, 59, 0.08)',
              border: '1px solid rgba(196, 154, 59, 0.25)',
              transition: 'all 0.2s ease',
              userSelect: 'none',
              margin: '0 16px 8px',
            }}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLDivElement).style.background = 'rgba(196, 154, 59, 0.15)';
              (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(196, 154, 59, 0.45)';
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLDivElement).style.background = 'rgba(196, 154, 59, 0.08)';
              (e.currentTarget as HTMLDivElement).style.borderColor = 'rgba(196, 154, 59, 0.25)';
            }}
            onClick={() => openUrl('https://svlmod.cn')}
          >
            <GlobalOutlined style={{ fontSize: 16 }} />
            <span>访问官网</span>
          </div>

          <nav className="svl-nav">
            {navItems.filter(item => !item.hidden).map((item) => {
              const isActive = location.pathname === item.key;
              return (
                <div
                  key={item.key}
                  className={`svl-nav-item ${isActive ? 'active' : ''}`}
                  onClick={() => startTransition(() => navigate(item.key))}
                >
                  <span className="svl-nav-icon">{item.icon}</span>
                  <span>{t(item.label)}</span>
                </div>
              );
            })}
            {errorCount > 0 && (
              <div
                className="svl-nav-item svl-nav-item-error"
                onClick={handleLogClick}
              >
                <Badge count={errorCount} style={{ backgroundColor: 'var(--svl-error)' }}>
                  <span style={{ color: 'var(--svl-error)' }}>⚠️</span>
                </Badge>
                <span style={{ color: 'var(--svl-error)', fontSize: 12 }}>{t('app.log.badgeText', { count: errorCount })}</span>
              </div>
            )}
          </nav>

          <div className="svl-sidebar-footer">
            <img
              src={chickenImg}
              alt={t('app.altChicken')}
              className="svl-chicken"
            />
          </div>
        </aside>

        <main className="svl-main">
          <Suspense fallback={
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
              <Spin indicator={<LoadingOutlined style={{ fontSize: 32 }} spin />} />
            </div>
          }>
            {(() => {
              const PageComponent = pageMap[location.pathname];
              return PageComponent ? <PageComponent /> : null;
            })()}
          </Suspense>
        </main>
      </div>
      <HomeModal />

      <Modal
        title={null}
        open={!!forceUpdateInfo}
        closable={false}
        maskClosable={false}
        keyboard={false}
        footer={null}
        centered
        width={520}
        style={{
          background: '#1a1510',
          border: '1px solid #4a3d2e',
          borderRadius: 12,
        }}
        styles={{
          body: {
            background: '#1a1510',
            color: '#f0e6d3',
          },
          mask: {
            backgroundColor: 'rgba(0, 0, 0, 0.7)',
          },
        }}
      >
        <div style={{ textAlign: 'center', padding: '16px 0' }}>
          <CloudDownloadOutlined style={{ fontSize: 48, color: '#1890ff', marginBottom: 16 }} />
          <Title level={3} style={{ marginBottom: 8, color: '#f0e6d3' }}>
            {t('features.serverUpdater.forceUpdateTitle')}
          </Title>
          <Text style={{ color: '#8a7d6b' }}>
            {t('features.serverUpdater.forceUpdateDesc')}
          </Text>
        </div>

        {forceUpdateInfo && (
          <div style={{ marginTop: 20, padding: 16, background: '#2d2418', borderRadius: 8 }}>
            <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap', marginBottom: 12 }}>
              <Tag color="blue">{t('features.updater.currentVersion')}: {forceUpdateInfo.current_version}</Tag>
              <Tag color="green">{t('features.updater.latestVersion')}: {forceUpdateInfo.latest_version}</Tag>
              <Tag color="red">{t('features.serverUpdater.forceUpdate')}</Tag>
            </div>

            {forceUpdateInfo.release_notes && (
              <div>
                <Text strong style={{ color: '#c4b89a' }}>{t('features.updater.releaseNotes')}:</Text>
                <Paragraph style={{ marginTop: 8, whiteSpace: 'pre-wrap', fontSize: 13, color: '#a09880' }}>
                  {forceUpdateInfo.release_notes}
                </Paragraph>
              </div>
            )}

            {forceDownloading && (
              <div style={{ marginTop: 16 }}>
                <Progress percent={forceProgress} status="active" strokeColor={{ '0%': '#8b6914', '100%': '#c49a3b' }} />
                {forceTotalBytes > 0 && (
                  <Text style={{ fontSize: 12, color: '#8a7d6b' }}>
                    {formatBytes(forceDownloadedBytes)} / {formatBytes(forceTotalBytes)}
                  </Text>
                )}
              </div>
            )}

            {forceInstalled && (
              <div style={{ marginTop: 16, textAlign: 'center' }}>
                <CheckCircleOutlined style={{ color: '#52c41a', fontSize: 20, marginRight: 8 }} />
                <Text style={{ color: '#52c41a' }}>{t('features.serverUpdater.downloadCompleteRestart')}</Text>
              </div>
            )}

            <div style={{ marginTop: 20, textAlign: 'center' }}>
              {!forceDownloading && !forceInstalled && (
                <Button type="primary" size="large" onClick={handleForceDownload} style={{ background: '#8b6914', borderColor: '#8b6914' }}>
                  <CloudDownloadOutlined />
                  {t('features.updater.downloadButton')}
                </Button>
              )}
              {forceInstalled && (
                <Button type="primary" size="large" danger onClick={handleForceRestart}>
                  {t('features.updater.restartButton')}
                </Button>
              )}
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
