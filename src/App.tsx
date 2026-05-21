import { useEffect, useState, useRef } from 'react';
import { ConfigProvider, theme } from 'antd';
import { useTheme } from './hooks/useTheme';
import { getNexusStatus, setNexusStatus, verifyNexusConnection } from './hooks/useNexusStatus';
import AppLayout from './components/AppLayout';
import appIcon from './assets/donate/app-icon.png';

type SplashPhase = 'showing' | 'hiding' | 'done';

function App() {
  const { theme: currentTheme } = useTheme();
  const [splashPhase, setSplashPhase] = useState<SplashPhase>('showing');
  const [mainAnimated, setMainAnimated] = useState(false);
  const splashTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    splashTimerRef.current = setTimeout(() => {
      setSplashPhase('hiding');
      setTimeout(() => {
        setSplashPhase('done');
        setMainAnimated(true);
      }, 400);
    }, 1500);

    return () => {
      if (splashTimerRef.current) clearTimeout(splashTimerRef.current);
    };
  }, []);

  useEffect(() => {
    const status = getNexusStatus();
    if (!status.hasApiKey) {
      const apiKey = localStorage.getItem('svl-nexus-api-key');
      if (apiKey) {
        const lastChecked = status.lastChecked;
        const fiveMinutes = 5 * 60 * 1000;
        if (lastChecked && Date.now() - lastChecked < fiveMinutes) {
          setNexusStatus({ hasApiKey: true });
          return;
        }
        setTimeout(() => verifyNexusConnection(apiKey), 2000);
      }
    }
  }, []);

  const isEyeCare = currentTheme === 'eyeCare';
  const algorithm = theme.darkAlgorithm;
  const primaryColor = isEyeCare ? '#5b8a72' : '#8b6914';
  const primaryHover = isEyeCare ? '#6b9b82' : '#a67c1a';
  const bgCard = isEyeCare ? '#232a28' : '#3d3225';
  const bgCardHover = isEyeCare ? '#2a3330' : '#4a3d2e';
  const borderColor = isEyeCare ? '#2f3a35' : '#4a3d2e';
  const textColor = isEyeCare ? '#d4ddd8' : '#f0e6d3';
  const textPlaceholder = isEyeCare ? '#7a8f82' : '#8a7d6b';

  return (
    <ConfigProvider
      theme={{
        algorithm,
        token: {
          colorPrimary: primaryColor,
          colorPrimaryHover: primaryHover,
          borderRadius: 8,
          colorBgContainer: bgCard,
          colorBorder: borderColor,
          colorText: textColor,
        },
        components: {
          Table: {
            colorBgContainer: bgCard,
            headerBg: isEyeCare ? '#2a3330' : '#2d2418',
            rowHoverBg: bgCardHover,
          },
          Modal: {
            contentBg: bgCard,
            headerBg: bgCard,
          },
          Card: {
            colorBgContainer: bgCard,
          },
          Tabs: {
            inkBarColor: primaryColor,
            itemActiveColor: primaryHover,
            itemSelectedColor: primaryHover,
            itemHoverColor: primaryColor,
          },
          Select: {
            colorBgContainer: bgCard,
            colorBgElevated: bgCard,
            colorBorder: borderColor,
            colorText: textColor,
            colorTextPlaceholder: textPlaceholder,
            optionSelectedBg: bgCardHover,
            optionActiveBg: bgCardHover,
          },
          Dropdown: {
            colorBgElevated: bgCard,
            colorBorder: borderColor,
            colorText: textColor,
          },
          Input: {
            colorBgContainer: bgCard,
            colorBorder: borderColor,
            colorText: textColor,
            colorTextPlaceholder: textPlaceholder,
          },
          Button: {
            colorBgContainer: bgCard,
            colorBorder: borderColor,
            colorText: textColor,
          },
          Radio: {
            colorBgContainer: bgCard,
            colorText: textColor,
          },
          Checkbox: {
            colorBgContainer: bgCard,
            colorText: textColor,
          },
          Popover: {
            colorBgElevated: bgCard,
            colorBorder: borderColor,
            colorText: textColor,
          },
          Tooltip: {
            colorBgElevated: bgCard,
            colorText: textColor,
          },
          List: {
            colorBgContainer: bgCard,
            colorBorder: borderColor,
            colorText: textColor,
          },
          Empty: {
            colorText: textColor,
            colorTextDescription: textPlaceholder,
          },
          Menu: {
            itemBg: 'transparent',
            itemColor: textColor,
            itemSelectedBg: 'transparent',
            itemSelectedColor: primaryColor,
            itemHoverBg: 'transparent',
            itemHoverColor: primaryHover,
          },
        },
      }}
    >
      {splashPhase !== 'done' && (
        <div className={`svl-splash-overlay${splashPhase === 'hiding' ? ' svl-splash-hiding' : ''}`}>
          <div className="svl-splash-logo-container">
            <img
              src={appIcon}
              alt="SVL"
              className="svl-splash-logo-img"
            />
          </div>
          <div className="svl-splash-title">SVL</div>
          <div className="svl-splash-subtitle">Mod Manager</div>
          <div className="svl-splash-loading">
            <div className="svl-splash-dot" />
            <div className="svl-splash-dot" />
            <div className="svl-splash-dot" />
          </div>
        </div>
      )}
      <div className={mainAnimated ? 'svl-main-shake-enter' : ''} style={{ opacity: splashPhase === 'done' ? undefined : 0 }}>
        <AppLayout />
      </div>
    </ConfigProvider>
  );
}

export default App;
