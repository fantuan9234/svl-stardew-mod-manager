import { useEffect, useMemo } from 'react';
import { ConfigProvider, theme } from 'antd';
import { useTheme, ThemeProvider } from './hooks/useTheme';
import { getNexusStatus, setNexusStatus, verifyNexusConnection } from './hooks/useNexusStatus';
import { SplashProvider } from './hooks/useSplashDone';
import AppLayout from './components/AppLayout';

function App() {
  return (
    <ThemeProvider>
      <AppInner />
    </ThemeProvider>
  );
}

function AppInner() {
  const { getAntdThemeConfig } = useTheme();

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

  const cfg = getAntdThemeConfig();

  const antdTheme = useMemo(() => ({
    algorithm: theme.darkAlgorithm,
    token: {
      colorPrimary: cfg.primaryColor,
      colorPrimaryHover: cfg.primaryHover,
      borderRadius: 8,
      controlHeight: 36,
      colorBgContainer: cfg.bgCard,
      colorBorder: cfg.borderColor,
      colorText: cfg.textColor,
    },
    components: {
      Table: {
        colorBgContainer: cfg.bgCard,
        headerBg: cfg.headerBg,
        rowHoverBg: cfg.bgCardHover,
      },
      Modal: {
        contentBg: cfg.bgCard,
        headerBg: cfg.bgCard,
      },
      Card: {
        colorBgContainer: cfg.bgCard,
      },
      Tabs: {
        inkBarColor: cfg.primaryColor,
        itemActiveColor: cfg.primaryHover,
        itemSelectedColor: cfg.primaryHover,
        itemHoverColor: cfg.primaryColor,
      },
      Select: {
        colorBgContainer: cfg.bgCard,
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextPlaceholder: cfg.textPlaceholder,
        optionSelectedBg: cfg.bgCardHover,
        optionActiveBg: cfg.bgCardHover,
      },
      Dropdown: {
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
      },
      Input: {
        colorBgContainer: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextPlaceholder: cfg.textPlaceholder,
      },
      Button: {
        colorBgContainer: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
      },
      Radio: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
      },
      Checkbox: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
      },
      Switch: {
        colorPrimary: cfg.primaryColor,
        colorPrimaryHover: cfg.primaryHover,
      },
      Tag: {
        colorBgContainer: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
      },
      Alert: {
        colorBgContainer: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextHeading: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
      },
      Badge: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
      },
      Progress: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
      },
      Timeline: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
        colorTextHeading: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
      },
      Descriptions: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
        colorTextSecondary: cfg.textPlaceholder,
        colorSplit: cfg.borderColor,
      },
      Collapse: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
        colorTextHeading: cfg.textColor,
        colorBorder: cfg.borderColor,
      },
      Transfer: {
        colorBgContainer: cfg.bgCard,
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextDisabled: cfg.textPlaceholder,
      },
      Tree: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
        colorTextDisabled: cfg.textPlaceholder,
      },
      Steps: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
        colorSplit: cfg.borderColor,
      },
      Segmented: {
        colorBgContainer: cfg.bgCard,
        itemSelectedBg: cfg.bgCardHover,
        itemHoverBg: cfg.bgCardHover,
        colorText: cfg.textColor,
        colorTextLabel: cfg.textColor,
      },
      Popover: {
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
      },
      Tooltip: {
        colorBgElevated: cfg.bgCard,
        colorText: cfg.textColor,
      },
      List: {
        colorBgContainer: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
      },
      Empty: {
        colorText: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
      },
      Menu: {
        itemBg: 'transparent',
        itemColor: cfg.textColor,
        itemSelectedBg: 'transparent',
        itemSelectedColor: cfg.primaryColor,
        itemHoverBg: 'transparent',
        itemHoverColor: cfg.primaryHover,
      },
      DatePicker: {
        colorBgContainer: cfg.bgCard,
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextPlaceholder: cfg.textPlaceholder,
        colorTextDisabled: cfg.textPlaceholder,
        cellHoverBg: cfg.bgCardHover,
        cellActiveWithRangeBg: cfg.bgCardHover,
        cellHoverWithRangeBg: cfg.bgCardHover,
      },
      Pagination: {
        colorBgContainer: cfg.bgCard,
        colorBgTextHover: cfg.bgCardHover,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextDisabled: cfg.textPlaceholder,
      },
      Spin: {
        colorPrimary: cfg.primaryColor,
      },
      Skeleton: {
        colorBgContainer: cfg.bgCard,
        color: cfg.borderColor,
      },
      Notification: {
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextHeading: cfg.textColor,
      },
      Message: {
        colorBgElevated: cfg.bgCard,
        colorText: cfg.textColor,
      },
      Upload: {
        colorBgContainer: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
        colorTextDisabled: cfg.textPlaceholder,
      },
      Slider: {
        colorBgContainer: cfg.bgCard,
        colorPrimary: cfg.primaryColor,
        colorBorder: cfg.borderColor,
      },
      Rate: {
        colorBgContainer: cfg.bgCard,
        colorFillContent: cfg.borderColor,
      },
      Avatar: {
        colorBgContainer: cfg.bgCard,
        colorText: cfg.textColor,
      },
      Breadcrumb: {
        colorText: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
        separatorColor: cfg.borderColor,
      },
      Drawer: {
        colorBgElevated: cfg.bgCard,
        colorText: cfg.textColor,
        colorBorder: cfg.borderColor,
      },
      Tour: {
        colorBgElevated: cfg.bgCard,
        colorText: cfg.textColor,
        colorBorder: cfg.borderColor,
      },
      FloatButton: {
        colorBgElevated: cfg.bgCard,
        colorText: cfg.textColor,
      },
      QRCode: {
        colorBgContainer: cfg.bgCard,
      },
      Statistic: {
        colorText: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
      },
      Image: {
        colorBgContainer: cfg.bgCard,
        colorBgMask: 'rgba(0, 0, 0, 0.7)',
      },
      Mentions: {
        colorBgContainer: cfg.bgCard,
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
      },
      ColorPicker: {
        colorBgContainer: cfg.bgCard,
        colorBgElevated: cfg.bgCard,
        colorBorder: cfg.borderColor,
        colorText: cfg.textColor,
      },
      Form: {
        colorText: cfg.textColor,
        colorTextDescription: cfg.textPlaceholder,
      },
    },
  }), [cfg]);

  return (
    <ConfigProvider theme={antdTheme}>
      <SplashProvider splashDone={true}>
        <AppLayout />
      </SplashProvider>
    </ConfigProvider>
  );
}

export default App;
