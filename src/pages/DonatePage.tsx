import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Typography, Modal } from 'antd';
import wechatQr from '../assets/donate/2aad98a5e1fec89ef7e8c427214f2fc0.jpg';
import alipayQr from '../assets/donate/app-icon.png';

const { Title, Text, Paragraph } = Typography;

export default function DonatePage() {
  const { t } = useTranslation();
  const [modalOpen, setModalOpen] = useState(false);
  const [modalImage, setModalImage] = useState('');

  const handleImageClick = (imageSrc: string) => {
    setModalImage(imageSrc);
    setModalOpen(true);
  };

  return (
    <div style={{ padding: 24, maxWidth: 720, margin: '0 auto' }}>
      <Title level={2} style={{ textAlign: 'center', marginBottom: 4 }}>
        {t('donate.title')}
      </Title>
      <Paragraph
        style={{
          textAlign: 'center',
          color: 'var(--svl-text-muted)',
          fontSize: 14,
          marginBottom: 32,
        }}
      >
        {t('donate.subtitle')}
      </Paragraph>

      <div
        style={{
          display: 'flex',
          justifyContent: 'center',
          gap: 40,
          flexWrap: 'wrap',
        }}
      >
        <div
          style={{
            background: 'var(--svl-card-bg)',
            border: '1px solid var(--svl-border)',
            borderRadius: 8,
            padding: 24,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            minWidth: 220,
          }}
        >
          <Text
            strong
            style={{
              marginBottom: 16,
              fontSize: 15,
              color: 'var(--svl-success)',
            }}
          >
            {t('donate.wechat')}
          </Text>
          <img
            src={wechatQr}
            alt={t('app.altWechatQr')}
            onClick={() => handleImageClick(wechatQr)}
            style={{
              width: 200,
              height: 200,
              imageRendering: 'pixelated',
              borderRadius: 8,
              cursor: 'pointer',
              transition: 'transform 0.2s',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.transform = 'scale(1.05)')}
            onMouseLeave={(e) => (e.currentTarget.style.transform = 'scale(1)')}
          />
          <Text
            style={{
              marginTop: 8,
              fontSize: 12,
              color: 'var(--svl-text-muted)',
            }}
          >
            {t('donate.clickToEnlarge')}
          </Text>
        </div>

        <div
          style={{
            background: 'var(--svl-card-bg)',
            border: '1px solid var(--svl-border)',
            borderRadius: 8,
            padding: 24,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            minWidth: 220,
          }}
        >
          <Text
            strong
            style={{
              marginBottom: 16,
              fontSize: 15,
              color: 'var(--svl-primary)',
            }}
          >
            {t('donate.alipay')}
          </Text>
          <img
            src={alipayQr}
            alt={t('app.altAlipayQr')}
            onClick={() => handleImageClick(alipayQr)}
            style={{
              width: 200,
              height: 200,
              imageRendering: 'pixelated',
              borderRadius: 8,
              cursor: 'pointer',
              transition: 'transform 0.2s',
            }}
            onMouseEnter={(e) => (e.currentTarget.style.transform = 'scale(1.05)')}
            onMouseLeave={(e) => (e.currentTarget.style.transform = 'scale(1)')}
          />
          <Text
            style={{
              marginTop: 8,
              fontSize: 12,
              color: 'var(--svl-text-muted)',
            }}
          >
            {t('donate.clickToEnlarge')}
          </Text>
        </div>
      </div>

      <Paragraph
        style={{
          textAlign: 'center',
          marginTop: 24,
          fontSize: 14,
          color: 'var(--svl-text-muted)',
        }}
      >
        QQ频道：<a href="https://pd.qq.com/s/pd68573550" target="_blank" rel="noopener noreferrer" style={{ color: 'var(--svl-primary)' }}>pd68573550</a>
      </Paragraph>

      <Paragraph
        style={{
          textAlign: 'center',
          marginTop: 32,
          fontSize: 14,
          color: 'var(--svl-text-muted)',
        }}
      >
        {t('donate.thanks')}
      </Paragraph>

      <Modal
        open={modalOpen}
        footer={null}
        onCancel={() => setModalOpen(false)}
        centered
        width={400}
        styles={{
          body: {
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
            padding: '24px',
          },
        }}
      >
        <img
          src={modalImage}
          alt={t('app.altEnlargedQr')}
          style={{
            width: 350,
            height: 'auto',
            imageRendering: 'pixelated',
            borderRadius: 8,
          }}
        />
      </Modal>
    </div>
  );
}
