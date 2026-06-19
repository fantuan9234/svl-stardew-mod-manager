import { useState, useEffect, useRef, lazy, Suspense, startTransition, memo, useCallback } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { MinusOutlined, BorderOutlined, CloseOutlined, SwitcherOutlined, LoadingOutlined } from '@ant-design/icons';
import { Badge, Modal, Button, Typography, Tag, message, Spin } from 'antd';
import { CloudDownloadOutlined, SyncOutlined, FolderOutlined, SaveOutlined, CoffeeOutlined, SearchOutlined, ToolOutlined, AppstoreOutlined, GlobalOutlined } from '@ant-design/icons';
import chickenImg from '../assets/chicken.png';
import sunIcon from '../assets/sv-sun-icon.png';
import moonIcon from '../assets/sv-moon-icon.png';
import HomeModal from './HomeModal';
import { openUrl } from '../utils/openUrl';
import { AppUpdateInfo, getCurrentAppVersion } from '../utils/tauri-api';
import { useTheme } from '../hooks/useTheme';
import { useSplashDone } from '../hooks/useSplashDone';
import { PageActiveProvider } from '../hooks/usePageActive';

const ModManager = lazy(() => import('../pages/ModManager'));
const NexusModBrowser = lazy(() => import('../pages/NexusModBrowser'));
const ProfilesPage = lazy(() => import('../pages/ProfilesPage'));
const SavesManager = lazy(() => import('../pages/SavesManager'));
const SyncPage = lazy(() => import('../pages/SyncPage'));
const Settings = lazy(() => import('../pages/Settings'));
const DonatePage = lazy(() => import('../pages/DonatePage'));
const LogViewer = lazy(() => import('../pages/LogViewer'));
const Toolbox = lazy(() => import('../pages/Toolbox'));

const lazyLoaders: Record<string, () => Promise<any>> = {
  '/mod-manager': () => import('../pages/ModManager'),
  '/nexus-browser': () => import('../pages/NexusModBrowser'),
  '/profiles': () => import('../pages/ProfilesPage'),
  '/saves': () => import('../pages/SavesManager'),
  '/sync': () => import('../pages/SyncPage'),
  '/settings': () => import('../pages/Settings'),
  '/donate': () => import('../pages/DonatePage'),
  '/log-viewer': () => import('../pages/LogViewer'),
  '/toolbox': () => import('../pages/Toolbox'),
};

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
  { key: '/mod-manager', icon: <AppstoreOutlined />, label: 'app.nav.mods' },
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

function lerpColor(a: number[], b: number[], t: number): string {
  const r = Math.round(a[0] + (b[0] - a[0]) * t);
  const g = Math.round(a[1] + (b[1] - a[1]) * t);
  const bl = Math.round(a[2] + (b[2] - a[2]) * t);
  return `rgb(${r},${g},${bl})`;
}

function hexToRgb(hex: string): number[] {
  const h = hex.replace('#', '');
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
}

interface SkyFrame {
  hour: number;
  colors: string[];
  starOpacity: number;
  cloudOpacity: number;
  isDay: boolean;
  particleOpacity: number;
  logoColor: string;
  iconFilter: string;
  orbGlow: string;
}

const SKY_FRAMES: SkyFrame[] = [
  { hour: 0,    colors: ['#080c24', '#0c1230', '#101838', '#141e40'], starOpacity: 1, cloudOpacity: 0, isDay: false, particleOpacity: 0, logoColor: '#b0b0e8', iconFilter: 'brightness(0.85) drop-shadow(0 0 6px rgba(200,200,255,0.6))', orbGlow: 'radial-gradient(circle, rgba(100,100,200,0.2) 0%, transparent 70%)' },
  { hour: 4,    colors: ['#0c1030', '#181e48', '#282858', '#382858'], starOpacity: 0.9, cloudOpacity: 0, isDay: false, particleOpacity: 0, logoColor: '#b0b0e8', iconFilter: 'brightness(0.85) drop-shadow(0 0 6px rgba(200,200,255,0.6))', orbGlow: 'radial-gradient(circle, rgba(100,100,200,0.2) 0%, transparent 70%)' },
  { hour: 5,    colors: ['#1a1848', '#382868', '#683878', '#c06050'], starOpacity: 0.4, cloudOpacity: 0, isDay: false, particleOpacity: 0.5, logoColor: '#F4A460', iconFilter: 'drop-shadow(0 0 10px rgba(255,160,80,0.6)) brightness(1.1)', orbGlow: 'radial-gradient(circle, rgba(255,160,80,0.3) 0%, transparent 70%)' },
  { hour: 5.5,  colors: ['#2a3878', '#5a5898', '#b86868', '#f0a050'], starOpacity: 0.1, cloudOpacity: 0.1, isDay: true, particleOpacity: 0.8, logoColor: '#F4A460', iconFilter: 'drop-shadow(0 0 10px rgba(255,160,80,0.6)) brightness(1.1)', orbGlow: 'radial-gradient(circle, rgba(255,160,80,0.3) 0%, transparent 70%)' },
  { hour: 6.5,  colors: ['#4a88c8', '#7ab8e0', '#b8dce8', '#f0d098'], starOpacity: 0, cloudOpacity: 0.4, isDay: true, particleOpacity: 0.3, logoColor: '#E65100', iconFilter: 'drop-shadow(0 2px 4px rgba(255,152,0,0.4))', orbGlow: 'radial-gradient(circle, rgba(255,200,50,0.25) 0%, transparent 70%)' },
  { hour: 8,    colors: ['#5898d8', '#88c8e8', '#b8e0f0', '#f0e0b8'], starOpacity: 0, cloudOpacity: 0.8, isDay: true, particleOpacity: 0, logoColor: '#E65100', iconFilter: 'drop-shadow(0 2px 4px rgba(255,152,0,0.4))', orbGlow: 'radial-gradient(circle, rgba(255,200,50,0.25) 0%, transparent 70%)' },
  { hour: 12,   colors: ['#4888d0', '#78b8e0', '#a8d8f0', '#e0d0a8'], starOpacity: 0, cloudOpacity: 1, isDay: true, particleOpacity: 0, logoColor: '#E65100', iconFilter: 'drop-shadow(0 2px 4px rgba(255,152,0,0.4))', orbGlow: 'radial-gradient(circle, rgba(255,200,50,0.25) 0%, transparent 70%)' },
  { hour: 15,   colors: ['#5088c0', '#80b0d0', '#b0c8d8', '#e0c0a0'], starOpacity: 0, cloudOpacity: 0.9, isDay: true, particleOpacity: 0, logoColor: '#E65100', iconFilter: 'drop-shadow(0 2px 4px rgba(255,152,0,0.4))', orbGlow: 'radial-gradient(circle, rgba(255,200,50,0.25) 0%, transparent 70%)' },
  { hour: 17,   colors: ['#4860a0', '#8868a0', '#c87060', '#e89050'], starOpacity: 0, cloudOpacity: 0.4, isDay: true, particleOpacity: 0.5, logoColor: '#D4725C', iconFilter: 'drop-shadow(0 0 10px rgba(255,120,60,0.5)) brightness(0.95)', orbGlow: 'radial-gradient(circle, rgba(255,120,60,0.3) 0%, transparent 70%)' },
  { hour: 18,   colors: ['#2a1850', '#5a2870', '#983858', '#d06040'], starOpacity: 0.2, cloudOpacity: 0.1, isDay: false, particleOpacity: 0.8, logoColor: '#D4725C', iconFilter: 'drop-shadow(0 0 10px rgba(255,120,60,0.5)) brightness(0.95)', orbGlow: 'radial-gradient(circle, rgba(255,120,60,0.3) 0%, transparent 70%)' },
  { hour: 19,   colors: ['#141040', '#282050', '#382858', '#483050'], starOpacity: 0.6, cloudOpacity: 0, isDay: false, particleOpacity: 0.4, logoColor: '#b0b0e8', iconFilter: 'brightness(0.85) drop-shadow(0 0 6px rgba(200,200,255,0.6))', orbGlow: 'radial-gradient(circle, rgba(100,100,200,0.2) 0%, transparent 70%)' },
  { hour: 20,   colors: ['#0c1030', '#141840', '#1c2048', '#242850'], starOpacity: 0.9, cloudOpacity: 0, isDay: false, particleOpacity: 0, logoColor: '#b0b0e8', iconFilter: 'brightness(0.85) drop-shadow(0 0 6px rgba(200,200,255,0.6))', orbGlow: 'radial-gradient(circle, rgba(100,100,200,0.2) 0%, transparent 70%)' },
  { hour: 24,   colors: ['#080c24', '#0c1230', '#101838', '#141e40'], starOpacity: 1, cloudOpacity: 0, isDay: false, particleOpacity: 0, logoColor: '#b0b0e8', iconFilter: 'brightness(0.85) drop-shadow(0 0 6px rgba(200,200,255,0.6))', orbGlow: 'radial-gradient(circle, rgba(100,100,200,0.2) 0%, transparent 70%)' },
];

function computeSkyState(hour: number) {
  const h = hour % 24;
  let prev = SKY_FRAMES[0];
  let next = SKY_FRAMES[1];
  for (let i = 0; i < SKY_FRAMES.length - 1; i++) {
    if (h >= SKY_FRAMES[i].hour && h < SKY_FRAMES[i + 1].hour) {
      prev = SKY_FRAMES[i];
      next = SKY_FRAMES[i + 1];
      break;
    }
  }
  const range = next.hour - prev.hour;
  const t = range > 0 ? (h - prev.hour) / range : 0;
  const pColors = prev.colors.map(hexToRgb);
  const nColors = next.colors.map(hexToRgb);
  const gradient = pColors.map((c, i) => lerpColor(c, nColors[i], t));
  const pLogo = hexToRgb(prev.logoColor);
  const nLogo = hexToRgb(next.logoColor);
  return {
    gradient: `linear-gradient(180deg, ${gradient[0]} 0%, ${gradient[1]} 30%, ${gradient[2]} 60%, ${gradient[3]} 100%)`,
    starOpacity: prev.starOpacity + (next.starOpacity - prev.starOpacity) * t,
    cloudOpacity: prev.cloudOpacity + (next.cloudOpacity - prev.cloudOpacity) * t,
    isDay: t < 0.5 ? prev.isDay : next.isDay,
    particleOpacity: prev.particleOpacity + (next.particleOpacity - prev.particleOpacity) * t,
    logoColor: lerpColor(pLogo, nLogo, t),
    iconFilter: t < 0.5 ? prev.iconFilter : next.iconFilter,
    orbGlow: t < 0.5 ? prev.orbGlow : next.orbGlow,
  };
}

function DayNightIcon() {
  const getDecimalHour = () => {
    const now = new Date();
    return now.getHours() + now.getMinutes() / 60 + now.getSeconds() / 3600;
  };

  const [skyState, setSkyState] = useState(() => computeSkyState(getDecimalHour()));
  const [bouncing, setBouncing] = useState(false);
  const [sparkles, setSparkles] = useState(false);

  useEffect(() => {
    const update = () => setSkyState(computeSkyState(getDecimalHour()));
    update();
    const interval = setInterval(update, 60000);
    return () => clearInterval(interval);
  }, []);

  const handleClick = () => {
    setBouncing(true);
    setSparkles(true);
    setTimeout(() => setBouncing(false), 600);
    setTimeout(() => setSparkles(false), 800);
  };

  const showStars = skyState.starOpacity > 0.05;
  const showClouds = skyState.cloudOpacity > 0.05;
  const showParticles = skyState.particleOpacity > 0.05;

  return (
    <div
      className={`svl-daynight-scene${bouncing ? ' svl-bounce' : ''}${sparkles ? ' svl-sparkle' : ''}`}
      style={{ background: skyState.gradient }}
      onClick={handleClick}
    >
      {showClouds && (
        <div className="svl-day-clouds" style={{ opacity: skyState.cloudOpacity }}>
          <span className="svl-cloud svl-cloud-1" />
          <span className="svl-cloud svl-cloud-2" />
          <span className="svl-cloud svl-cloud-3" />
        </div>
      )}
      <div className="svl-daynight-orb" style={{ background: skyState.orbGlow }}>
        <img
          className="svl-daynight-icon"
          src={skyState.isDay ? sunIcon : moonIcon}
          alt=""
          draggable={false}
          style={{ filter: skyState.iconFilter }}
        />
      </div>
      {showStars && (
        <div className="svl-night-stars" style={{ opacity: skyState.starOpacity }}>
          <span className="svl-star svl-star-1" />
          <span className="svl-star svl-star-2" />
          <span className="svl-star svl-star-3" />
          <span className="svl-star svl-star-4" />
          <span className="svl-star svl-star-5" />
        </div>
      )}
      {showParticles && (
        <div className="svl-transition-particles" style={{ opacity: skyState.particleOpacity }}>
          <span className="svl-particle svl-particle-1" />
          <span className="svl-particle svl-particle-2" />
          <span className="svl-particle svl-particle-3" />
        </div>
      )}
      {sparkles && (
        <div className="svl-click-sparkles">
          <span className="svl-sparkle-dot svl-sparkle-dot-1" />
          <span className="svl-sparkle-dot svl-sparkle-dot-2" />
          <span className="svl-sparkle-dot svl-sparkle-dot-3" />
          <span className="svl-sparkle-dot svl-sparkle-dot-4" />
          <span className="svl-sparkle-dot svl-sparkle-dot-5" />
          <span className="svl-sparkle-dot svl-sparkle-dot-6" />
        </div>
      )}
      <div className="svl-logo-text" style={{ color: skyState.logoColor }}>SVL</div>
    </div>
  );
}

const Sidebar = memo(function Sidebar({
  activePath,
  errorCount,
  customChickenImage,
  sidebarLogoMode,
  customSidebarImage,
  appVersion,
  onNavigate,
  onLogClick,
}: {
  activePath: string;
  errorCount: number;
  customChickenImage: string | null;
  sidebarLogoMode: string;
  customSidebarImage: string;
  appVersion: string;
  onNavigate: (path: string) => void;
  onLogClick: () => void;
}) {
  const { t } = useTranslation();

  const logoContent = (() => {
    if (sidebarLogoMode === 'farm') {
      return (
        <div className="svl-sidebar-farm-logo">
          <img src="/images/stardew-farm-screenshot.jpg" alt="" className="svl-sidebar-farm-img" />
          <div className="svl-sidebar-farm-overlay" />
          <div className="svl-logo-text" style={{ color: 'var(--svl-text-primary)' }}>SVL</div>
        </div>
      );
    }
    if (sidebarLogoMode === 'custom' && customSidebarImage) {
      return (
        <div className="svl-sidebar-farm-logo">
          <img src={customSidebarImage} alt="" className="svl-sidebar-farm-img" />
          <div className="svl-sidebar-farm-overlay" />
          <div className="svl-logo-text" style={{ color: 'var(--svl-text-primary)' }}>SVL</div>
        </div>
      );
    }
    return <DayNightIcon />;
  })();

  return (
    <aside className="svl-sidebar">
      <div className="svl-logo">
        {logoContent}
      </div>

      <nav className="svl-nav">
        {navItems.filter(item => !item.hidden).map((item) => {
          const isActive = activePath === item.key;
          return (
            <div
              key={item.key}
              className={`svl-nav-item ${isActive ? 'active' : ''}`}
              onClick={() => onNavigate(item.key)}
            >
              <span className="svl-nav-icon">{item.icon}</span>
              <span>{t(item.label)}</span>
            </div>
          );
        })}
        {errorCount > 0 && (
          <div
            className="svl-nav-item svl-nav-item-error"
            onClick={onLogClick}
          >
            <Badge count={errorCount} style={{ backgroundColor: 'var(--svl-error)' }}>
              <span style={{ color: 'var(--svl-error)' }}>⚠️</span>
            </Badge>
            <span style={{ color: 'var(--svl-error)', fontSize: 12 }}>{t('app.log.badgeText', { count: errorCount })}</span>
          </div>
        )}

        <div
          className="svl-nav-item svl-nav-item-website"
          onClick={() => openUrl('https://svlmod.cn')}
        >
          <span className="svl-nav-icon"><GlobalOutlined /></span>
          <span>{t('app.nav.website') || '访问官网'}</span>
        </div>
      </nav>

      <div className="svl-sidebar-footer">
        <img
          src={customChickenImage || chickenImg}
          alt={t('app.altChicken')}
          className="svl-chicken"
        />
        {appVersion && (
          <div style={{ textAlign: 'center', fontSize: 11, color: 'var(--svl-text-muted)', marginTop: 4 }}>
            v{appVersion}
          </div>
        )}
      </div>
    </aside>
  );
});

const PageWrapper = memo(function PageWrapper({
  path,
  isActive,
  children,
}: {
  path: string;
  isActive: boolean;
  children: React.ReactNode;
}) {
  return (
    <div key={path} style={{ display: isActive ? 'contents' : 'none' }}>
      {children}
    </div>
  );
});

export default function AppLayout() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const { customColors } = useTheme();
  const splashDone = useSplashDone();
  const [isMaximized, setIsMaximized] = useState(false);
  const [errorCount, setErrorCount] = useState(0);
  const [appVersion, setAppVersion] = useState('');
  const logCheckUnlistenRef = useRef<(() => void) | null>(null);

  const [forceUpdateInfo, setForceUpdateInfo] = useState<AppUpdateInfo | null>(null);
  const pendingUpdateInfoRef = useRef<AppUpdateInfo | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<AppUpdateInfo>('app-update-available', (event) => {
      const info = event.payload;
      pendingUpdateInfoRef.current = info;
    }).then(fn => { unlisten = fn; });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!splashDone) return;
    const info = pendingUpdateInfoRef.current;
    if (!info) return;
    pendingUpdateInfoRef.current = null;
    if (info.force_update) {
      setForceUpdateInfo(info);
    } else {
      message.info({
        content: t('features.serverUpdater.newVersionAvailable', { version: info.latest_version }),
        duration: 5,
        onClick: () => {
          startTransition(() => navigate('/settings', { state: { updateInfo: info } }));
        },
      });
    }
  }, [splashDone, navigate, t]);

  useEffect(() => {
    if (location.pathname === '/') {
      startTransition(() => navigate('/mod-manager', { replace: true }));
    }
  }, [location.pathname, navigate]);

  useEffect(() => {
    getCurrentAppVersion().then(v => setAppVersion(v)).catch(() => {});
  }, []);

  useEffect(() => {
    const loaders = Object.values(lazyLoaders);
    loaders.forEach((loader, i) => {
      setTimeout(() => loader().catch(() => {}), i * 300);
    });
  }, []);

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
    const interval = setInterval(checkLog, 120000);
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
            try {
              window.dispatchEvent(new CustomEvent('svl:mod-list-refresh'));
            } catch {}
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

  const handleLogClick = useCallback(() => {
    startTransition(() => navigate('/log-viewer'));
  }, [navigate]);

  const handleNavClick = useCallback((path: string) => {
    startTransition(() => navigate(path));
  }, [navigate]);

  const handleForceDownload = () => {
    if (!forceUpdateInfo) return;
    const info = forceUpdateInfo;
    setForceUpdateInfo(null);
    startTransition(() => navigate('/settings', { state: { updateInfo: info } }));
  };

  const renderedPagesRef = useRef<Set<string>>(new Set());
  const [, setRenderTick] = useState(0);
  const prevPathRef = useRef<string>('');

  useEffect(() => {
    if (location.pathname !== '/') {
      const isNew = !renderedPagesRef.current.has(location.pathname);
      renderedPagesRef.current.add(location.pathname);
      if (prevPathRef.current && prevPathRef.current !== '/' && prevPathRef.current !== location.pathname) {
        renderedPagesRef.current.add(prevPathRef.current);
      }
      prevPathRef.current = location.pathname;
      if (isNew) {
        setRenderTick(t => t + 1);
      }
    }
  }, [location.pathname]);

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
        <Sidebar
          activePath={location.pathname}
          errorCount={errorCount}
          customChickenImage={customColors.customChickenImage}
          sidebarLogoMode={customColors.sidebarLogoMode}
          customSidebarImage={customColors.customSidebarImage}
          appVersion={appVersion}
          onNavigate={handleNavClick}
          onLogClick={handleLogClick}
        />

        <main className="svl-main">
          <PageActiveProvider value={location.pathname}>
          <Suspense fallback={
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
              <Spin indicator={<LoadingOutlined style={{ fontSize: 32 }} spin />} />
            </div>
          }>
            {Array.from(renderedPagesRef.current).map((path) => {
              const PageComponent = pageMap[path];
              if (!PageComponent) return null;
              const isActive = path === location.pathname;
              return (
                <PageWrapper key={path} path={path} isActive={isActive}>
                  <PageComponent />
                </PageWrapper>
              );
            })}
            {(() => {
              if (renderedPagesRef.current.has(location.pathname)) return null;
              const PageComponent = pageMap[location.pathname];
              return PageComponent ? <PageComponent /> : null;
            })()}
          </Suspense>
          </PageActiveProvider>
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
          background: 'var(--svl-bg-primary)',
          border: '1px solid var(--svl-border)',
          borderRadius: 12,
        }}
        styles={{
          body: {
            background: 'var(--svl-bg-primary)',
            color: 'var(--svl-text-primary)',
          },
          mask: {
            backgroundColor: 'rgba(0, 0, 0, 0.7)',
          },
        }}
      >
        <div style={{ textAlign: 'center', padding: '16px 0' }}>
          <CloudDownloadOutlined style={{ fontSize: 48, color: '#1890ff', marginBottom: 16 }} />
          <Title level={3} style={{ marginBottom: 8, color: 'var(--svl-text-primary)' }}>
            {t('features.serverUpdater.forceUpdateTitle')}
          </Title>
          <Text style={{ color: 'var(--svl-text-muted)' }}>
            {t('features.serverUpdater.forceUpdateDesc')}
          </Text>
        </div>

        {forceUpdateInfo && (
          <div style={{ marginTop: 20, padding: 16, background: 'var(--svl-bg-secondary)', borderRadius: 8 }}>
            <div style={{ display: 'flex', gap: 12, alignItems: 'center', flexWrap: 'wrap', marginBottom: 12 }}>
              <Tag color="blue">{t('features.updater.currentVersion')}: {forceUpdateInfo.current_version}</Tag>
              <Tag color="green">{t('features.updater.latestVersion')}: {forceUpdateInfo.latest_version}</Tag>
              <Tag color="red">{t('features.serverUpdater.forceUpdate')}</Tag>
            </div>

            {forceUpdateInfo.release_notes && (
              <div>
                <Text strong style={{ color: 'var(--svl-text-secondary)' }}>{t('features.updater.releaseNotes')}:</Text>
                <Paragraph style={{ marginTop: 8, whiteSpace: 'pre-wrap', fontSize: 13, color: '#a09880' }}>
                  {forceUpdateInfo.release_notes}
                </Paragraph>
              </div>
            )}

            <div style={{ marginTop: 20, textAlign: 'center' }}>
              <Button type="primary" size="large" onClick={handleForceDownload} style={{ background: 'var(--svl-primary)', borderColor: 'var(--svl-primary)' }}>
                <CloudDownloadOutlined />
                {t('features.updater.downloadButton')}
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </div>
  );
}
