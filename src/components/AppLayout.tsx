import { useState, useEffect, useRef } from 'react';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { MinusOutlined, BorderOutlined, CloseOutlined, SwitcherOutlined } from '@ant-design/icons';
import { Badge } from 'antd';
import chickenImg from '../assets/chicken.png';

import { SyncOutlined, FolderOutlined, SaveOutlined, CoffeeOutlined } from '@ant-design/icons';

const navItems = [
  { key: '/mod-manager', icon: '', label: 'app.nav.mods' },
  { key: '/profiles', icon: <FolderOutlined />, label: 'app.nav.profiles' },
  { key: '/saves', icon: <SaveOutlined />, label: 'app.nav.saves' },
  { key: '/sync', icon: <SyncOutlined />, label: 'app.nav.sync' },
  { key: '/settings', icon: '⚙️', label: 'app.nav.settings' },
  { key: '/donate', icon: <CoffeeOutlined />, label: 'sidebar.donate' },
  { key: '/log-viewer', icon: '', label: '', hidden: true },
];

export default function AppLayout() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const [isMaximized, setIsMaximized] = useState(false);
  const [errorCount, setErrorCount] = useState(0);
  const logCheckUnlistenRef = useRef<(() => void) | null>(null);

  const checkLog = async () => {
    try {
      const result = await invoke<any>('check_smapi_log');
      if (result.has_error && result.error_count > 0) {
        setErrorCount(result.error_count);
      } else {
        setErrorCount(0);
      }
    } catch {}
  };

  useEffect(() => {
    const window = getCurrentWindow();
    window.isMaximized().then(setIsMaximized).catch(() => {});

    const unlisten = window.onResized(async () => {
      try {
        const maximized = await window.isMaximized();
        setIsMaximized(maximized);
      } catch {}
    });

    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, []);

  useEffect(() => {
    checkLog();
    const interval = setInterval(checkLog, 30000);
    return () => clearInterval(interval);
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
    getCurrentWindow().minimize();
  };

  const handleToggleMaximize = () => {
    getCurrentWindow().toggleMaximize();
  };

  const handleClose = () => {
    getCurrentWindow().close();
  };

  const handleLogClick = () => {
    navigate('/log-viewer');
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
            <div className="svl-logo-icon">SVL</div>
          </div>

          <nav className="svl-nav">
            {navItems.filter(item => !item.hidden).map((item) => {
              const isActive = location.pathname === item.key;
              return (
                <div
                  key={item.key}
                  className={`svl-nav-item ${isActive ? 'active' : ''}`}
                  onClick={() => navigate(item.key)}
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
              alt="chicken"
              className="svl-chicken"
            />
          </div>
        </aside>

        <main className="svl-main">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
