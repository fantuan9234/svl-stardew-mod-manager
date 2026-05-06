import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Alert, Button, Space } from 'antd';
import { SettingOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';

export default function ApiKeyReminder() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    const apiKey = localStorage.getItem('svl-nexus-api-key');
    const dismissed = localStorage.getItem('svl-api-key-reminder-dismissed');
    
    if (!apiKey && dismissed !== 'true') {
      setVisible(true);
    }
  }, []);

  const handleDismiss = () => {
    localStorage.setItem('svl-api-key-reminder-dismissed', 'true');
    setVisible(false);
  };

  const handleGoToSettings = () => {
    navigate('/settings');
  };

  if (!visible) {
    return null;
  }

  return (
    <Alert
      message={t('app.apiKeyReminder.title')}
      description={
        <div>
          <p>{t('app.apiKeyReminder.description')}</p>
          <ul style={{ margin: '8px 0', paddingLeft: '20px' }}>
            <li>{t('app.apiKeyReminder.feature1')}</li>
            <li>{t('app.apiKeyReminder.feature2')}</li>
            <li>{t('app.apiKeyReminder.feature3')}</li>
          </ul>
          <Space>
            <Button
              type="primary"
              icon={<SettingOutlined />}
              onClick={handleGoToSettings}
              size="small"
            >
              {t('app.apiKeyReminder.goToSettings')}
            </Button>
            <Button
              onClick={handleDismiss}
              size="small"
            >
              {t('app.apiKeyReminder.dismiss')}
            </Button>
          </Space>
        </div>
      }
      type="warning"
      showIcon
      closable
      onClose={handleDismiss}
      style={{ marginBottom: 16 }}
    />
  );
}
