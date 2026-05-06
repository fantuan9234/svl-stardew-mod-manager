import { useEffect } from 'react';
import { ConfigProvider, theme } from 'antd';
import { useTheme } from './hooks/useTheme';
import { getNexusStatus, verifyNexusConnection } from './hooks/useNexusStatus';
import AppLayout from './components/AppLayout';

function App() {
  const { theme: currentTheme } = useTheme();

  useEffect(() => {
    const status = getNexusStatus();
    if (!status.hasApiKey) {
      const apiKey = localStorage.getItem('svl-nexus-api-key');
      if (apiKey) {
        verifyNexusConnection(apiKey);
      }
    }
  }, []);

  const isEyeCare = currentTheme === 'eyeCare';
  const algorithm = theme.darkAlgorithm;

  return (
    <ConfigProvider
      theme={{
        algorithm,
        token: {
          colorPrimary: isEyeCare ? '#5b8a72' : '#7c3aed',
          borderRadius: 8,
        },
        components: {
          Table: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            headerBg: isEyeCare ? '#2a3330' : '#1c2333',
            rowHoverBg: isEyeCare ? '#2a3330' : '#1c2333',
          },
          Modal: {
            contentBg: isEyeCare ? '#232a28' : '#161b22',
            headerBg: isEyeCare ? '#232a28' : '#161b22',
          },
          Card: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
          },
          Tabs: {
            inkBarColor: isEyeCare ? '#5b8a72' : '#7c3aed',
            itemActiveColor: isEyeCare ? '#7db89a' : '#a78bfa',
            itemSelectedColor: isEyeCare ? '#7db89a' : '#a78bfa',
            itemHoverColor: isEyeCare ? '#6b9b82' : '#8b5cf6',
          },
          Select: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            colorBgElevated: isEyeCare ? '#232a28' : '#161b22',
            colorBorder: isEyeCare ? '#2f3a35' : '#21262d',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
            colorTextPlaceholder: isEyeCare ? '#7a8f82' : '#64748b',
            optionSelectedBg: isEyeCare ? '#2a3330' : '#1c2333',
            optionActiveBg: isEyeCare ? '#2a3330' : '#1c2333',
          },
          Dropdown: {
            colorBgElevated: isEyeCare ? '#232a28' : '#161b22',
            colorBorder: isEyeCare ? '#2f3a35' : '#21262d',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          Input: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            colorBorder: isEyeCare ? '#2f3a35' : '#21262d',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
            colorTextPlaceholder: isEyeCare ? '#7a8f82' : '#64748b',
          },
          Button: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            colorBorder: isEyeCare ? '#2f3a35' : '#21262d',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          Radio: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          Checkbox: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          Popover: {
            colorBgElevated: isEyeCare ? '#232a28' : '#161b22',
            colorBorder: isEyeCare ? '#2f3a35' : '#21262d',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          Tooltip: {
            colorBgElevated: isEyeCare ? '#232a28' : '#161b22',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          List: {
            colorBgContainer: isEyeCare ? '#232a28' : '#161b22',
            colorBorder: isEyeCare ? '#2f3a35' : '#21262d',
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
          },
          Empty: {
            colorText: isEyeCare ? '#d4ddd8' : '#e2e8f0',
            colorTextDescription: isEyeCare ? '#7a8f82' : '#64748b',
          },
        },
      }}
    >
      <AppLayout />
    </ConfigProvider>
  );
}

export default App;
