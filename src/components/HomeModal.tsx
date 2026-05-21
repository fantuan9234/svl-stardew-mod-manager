import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { openUrl } from '../utils/openUrl';
import { ArrowRightOutlined, CloseOutlined } from '@ant-design/icons';
import appIcon from '../assets/donate/app-icon.png';

const adData = [
  {
    id: 1,
    title: '24小时无人值守星露谷物语联机',
    subtitle: '手机+PC+iOS三端互通联机',
    description: '无人数上限 · 可开大型社区服',
    tag: '星露谷专属',
    tagColor: '#00b894',
    link: 'https://yy.0play.cn/auth/register?ref=REF1330FA2E',
    features: ['24小时稳定运行', '三端互通联机', '无人数上限', '支持大型社区服'],
  },
];

export default function HomeModal() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(() => {
    try {
      const dismissed = localStorage.getItem('svl_home_modal_dismissed');
      if (dismissed === new Date().toISOString().split('T')[0]) return false;
      return true;
    } catch {
      return true;
    }
  });
  const [hoverClose, setHoverClose] = useState(false);

  const handleDontShowAgain = () => {
    try {
      localStorage.setItem('svl_home_modal_dismissed', new Date().toISOString().split('T')[0]);
    } catch {}
    setOpen(false);
  };

  const handleClose = () => {
    setOpen(false);
  };

  const handleEnterManager = () => {
    setOpen(false);
  };

  if (!open) return null;

  const ad = adData[0];

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 900,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'linear-gradient(160deg, #0f0d0a 0%, #1a1612 30%, #14110e 60%, #0d0b08 100%)',
        overflow: 'hidden',
      }}
    >
      {/* Decorative background elements */}
      <div
        style={{
          position: 'absolute',
          top: '-15%',
          left: '50%',
          transform: 'translateX(-50%)',
          width: '800px',
          height: '600px',
          background: 'radial-gradient(ellipse at center, rgba(139, 105, 20, 0.06) 0%, transparent 70%)',
          pointerEvents: 'none',
        }}
      />
      <div
        style={{
          position: 'absolute',
          bottom: '-20%',
          left: '-10%',
          width: '500px',
          height: '500px',
          borderRadius: '50%',
          background: 'radial-gradient(circle, rgba(139, 105, 20, 0.04) 0%, transparent 70%)',
          pointerEvents: 'none',
        }}
      />

      {/* Close button */}
      <div
        style={{
          position: 'absolute',
          top: 24,
          right: 24,
          width: 40,
          height: 40,
          borderRadius: 10,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          cursor: 'pointer',
          background: hoverClose ? 'rgba(255, 77, 79, 0.15)' : 'transparent',
          border: hoverClose ? '1px solid rgba(255, 77, 79, 0.3)' : '1px solid transparent',
          transition: 'all 0.2s',
          zIndex: 10,
        }}
        onClick={handleClose}
        onMouseEnter={() => setHoverClose(true)}
        onMouseLeave={() => setHoverClose(false)}
      >
        <CloseOutlined style={{ color: hoverClose ? '#ff4d4f' : '#5a5040', fontSize: 18 }} />
      </div>

      {/* Scrollable content */}
      <div
        style={{
          width: '100%',
          maxWidth: 680,
          maxHeight: '100vh',
          overflowY: 'auto',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          padding: '40px 32px 24px',
          position: 'relative',
          zIndex: 2,
        }}
      >
        {/* ===== 1. Brand Area ===== */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            marginBottom: 36,
          }}
        >
          <img
            src={appIcon}
            alt="SVL Mod Manager"
            style={{
              width: 64,
              height: 64,
              borderRadius: 16,
              marginBottom: 16,
              boxShadow: '0 6px 24px rgba(139, 105, 20, 0.2)',
              imageRendering: 'auto',
            }}
          />
          <h1
            style={{
              margin: 0,
              fontSize: 26,
              fontWeight: 700,
              color: '#f0e6d3',
              letterSpacing: '-0.3px',
              lineHeight: 1.2,
            }}
          >
            Stardew Valley Mod Manager
          </h1>
          <p
            style={{
              margin: '8px 0 0',
              fontSize: 14,
              color: '#8a7d6b',
              fontWeight: 400,
            }}
          >
            轻松管理你的星露谷模组
          </p>
        </div>

        {/* ===== 2. Core Action Area ===== */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            marginBottom: 32,
          }}
        >
          <button
            onClick={handleEnterManager}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 8,
              background: 'linear-gradient(135deg, #8b6914, #c49a3b)',
              color: '#fff',
              border: 'none',
              padding: '14px 48px',
              borderRadius: 14,
              fontSize: 18,
              fontWeight: 700,
              cursor: 'pointer',
              transition: 'all 0.25s ease',
              boxShadow: '0 4px 24px rgba(139, 105, 20, 0.35), 0 0 0 1px rgba(196, 154, 59, 0.1)',
              letterSpacing: '1px',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.transform = 'translateY(-2px)';
              e.currentTarget.style.boxShadow = '0 8px 32px rgba(139, 105, 20, 0.5), 0 0 0 1px rgba(196, 154, 59, 0.2)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = 'translateY(0)';
              e.currentTarget.style.boxShadow = '0 4px 24px rgba(139, 105, 20, 0.35), 0 0 0 1px rgba(196, 154, 59, 0.1)';
            }}
          >
            进入管理器
            <ArrowRightOutlined style={{ fontSize: 16 }} />
          </button>
          <p
            style={{
              margin: '12px 0 0',
              fontSize: 12,
              color: '#5a5040',
              textAlign: 'center',
            }}
          >
            进入即表示您已知晓并感谢赞助商的支持
          </p>
        </div>

        {/* ===== 3. Ad Display Area ===== */}
        {/* AD_CONTAINER_ID: splsh-ad-01 — reserved for future ad SDK integration */}
        <div
          id="splash-ad-01"
          style={{
            width: '100%',
            maxWidth: 560,
            background: 'rgba(255, 255, 255, 0.03)',
            border: '1px solid rgba(255, 255, 255, 0.06)',
            borderRadius: 16,
            padding: 20,
            marginBottom: 28,
          }}
        >
          {/* Ad section label */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              marginBottom: 14,
            }}
          >
            <div
              style={{
                width: 3,
                height: 14,
                borderRadius: 2,
                background: '#c49a3b',
              }}
            />
            <span style={{ color: '#8a7d6b', fontSize: 12, fontWeight: 600, letterSpacing: '0.5px' }}>
              赞助商推荐
            </span>
          </div>

          {/* Ad content container — 16:9 aspect ratio */}
          <div
            style={{
              position: 'relative',
              width: '100%',
              paddingBottom: '56.25%',
              borderRadius: 10,
              overflow: 'hidden',
              cursor: 'pointer',
              background: 'linear-gradient(135deg, #0a1a2e 0%, #112240 50%, #0d1b2a 100%)',
              transition: 'transform 0.2s ease, box-shadow 0.2s ease',
            }}
            onClick={() => ad.link && openUrl(ad.link)}
            onMouseEnter={(e) => {
              e.currentTarget.style.transform = 'scale(1.01)';
              e.currentTarget.style.boxShadow = '0 4px 24px rgba(0, 212, 255, 0.12)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            <div
              style={{
                position: 'absolute',
                inset: 0,
                display: 'flex',
                flexDirection: 'column',
                justifyContent: 'center',
                padding: '24px 32px',
              }}
            >
              <div
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  background: ad.tagColor,
                  color: '#fff',
                  padding: '3px 10px',
                  borderRadius: 4,
                  fontSize: 11,
                  fontWeight: 600,
                  width: 'fit-content',
                  marginBottom: 10,
                  opacity: 0.9,
                }}
              >
                {ad.tag}
              </div>
              <h2
                style={{
                  margin: '0 0 6px',
                  color: '#fff',
                  fontSize: 20,
                  fontWeight: 700,
                  letterSpacing: '-0.3px',
                }}
              >
                {ad.title}
              </h2>
              <p
                style={{
                  margin: '0 0 4px',
                  color: '#00d4ff',
                  fontSize: 13,
                  fontWeight: 600,
                }}
              >
                {ad.subtitle}
              </p>
              <p
                style={{
                  margin: '0 0 16px',
                  color: 'rgba(255,255,255,0.55)',
                  fontSize: 12,
                }}
              >
                {ad.description}
              </p>
              <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
                {ad.features.map((feature, idx) => (
                  <span
                    key={idx}
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: 4,
                      padding: '4px 10px',
                      borderRadius: 5,
                      background: 'rgba(255,255,255,0.06)',
                      border: '1px solid rgba(255,255,255,0.08)',
                      color: 'rgba(255,255,255,0.75)',
                      fontSize: 11,
                      fontWeight: 500,
                    }}
                  >
                    <span style={{ color: '#00ff88', fontSize: 10 }}>✓</span>
                    {feature}
                  </span>
                ))}
              </div>
            </div>

            {/* Decorative grid */}
            <div
              style={{
                position: 'absolute',
                inset: 0,
                backgroundImage:
                  'linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px)',
                backgroundSize: '32px 32px',
                pointerEvents: 'none',
              }}
            />
            {/* Decorative glow */}
            <div
              style={{
                position: 'absolute',
                top: '-30%',
                right: '-10%',
                width: '300px',
                height: '300px',
                borderRadius: '50%',
                background: 'radial-gradient(circle, rgba(0, 212, 255, 0.06) 0%, transparent 70%)',
                pointerEvents: 'none',
              }}
            />
          </div>
        </div>

        {/* ===== 4. Dismiss Button ===== */}
        <button
          onClick={handleDontShowAgain}
          style={{
            background: 'transparent',
            border: '1px solid rgba(255,255,255,0.08)',
            color: '#5a5040',
            padding: '10px 28px',
            borderRadius: 8,
            fontSize: 13,
            cursor: 'pointer',
            transition: 'all 0.2s',
            letterSpacing: '0.5px',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.color = '#8a7d6b';
            e.currentTarget.style.borderColor = 'rgba(196, 154, 59, 0.3)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.color = '#5a5040';
            e.currentTarget.style.borderColor = 'rgba(255,255,255,0.08)';
          }}
        >
          {t('home.ads.dontShowAgain')}
        </button>

        {/* Scroll spacer */}
        <div style={{ height: 8 }} />
      </div>
    </div>
  );
}