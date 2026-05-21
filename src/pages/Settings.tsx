import { Typography, Button, Space, Radio, Divider } from 'antd';
import { useTranslation } from 'react-i18next';
import i18n from '../i18n';
import { useTheme } from '../hooks/useTheme';
import NexusApiConfig from '../components/NexusApiConfig';


const { Title, Text } = Typography;

export default function Settings() {
  const { t } = useTranslation();
  const { theme, switchTheme } = useTheme();

  const handleLanguageChange = (lang: string) => {
    i18n.changeLanguage(lang);
    localStorage.setItem('svl-language', lang);
  };

  return (
    <div style={{ padding: 24 }}>
      <Title level={2}>{t('app.pages.settings.title')}</Title>

      <div style={{ marginTop: 24 }}>
        <Text strong style={{ display: 'block', marginBottom: 12 }}>
          {t('app.theme.title')}
        </Text>
        <Radio.Group
          value={theme}
          onChange={(e) => switchTheme(e.target.value)}
          buttonStyle="solid"
        >
          <Radio.Button value="colorful">
            {t('app.theme.colorful')}
          </Radio.Button>
          <Radio.Button value="eyeCare">
            {t('app.theme.eyeCare')}
          </Radio.Button>
        </Radio.Group>
      </div>

      <div style={{ marginTop: 24 }}>
        <Text strong style={{ display: 'block', marginBottom: 12 }}>
          {t('app.language.switch')}
        </Text>
        <Space>
          <Button
            type={i18n.language === 'zh' ? 'primary' : 'default'}
            onClick={() => handleLanguageChange('zh')}
          >
            中文
          </Button>
          <Button
            type={i18n.language === 'en' ? 'primary' : 'default'}
            onClick={() => handleLanguageChange('en')}
          >
            English
          </Button>
        </Space>
      </div>

      <Divider style={{ marginTop: 32, marginBottom: 24 }} />

      <NexusApiConfig />
    </div>
  );
}
