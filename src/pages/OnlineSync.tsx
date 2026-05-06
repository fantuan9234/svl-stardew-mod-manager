import { Typography, Result } from 'antd';
import { useTranslation } from 'react-i18next';

const { Title } = Typography;

export default function OnlineSync() {
  const { t } = useTranslation();

  return (
    <div style={{ padding: 24 }}>
      <Title level={2}>{t('app.pages.onlineSync.title')}</Title>
      <Result
        status="warning"
        title={t('app.pages.onlineSync.placeholder')}
        subTitle={t('app.pages.onlineSync.disabled')}
        style={{ color: 'inherit' }}
      />
    </div>
  );
}
