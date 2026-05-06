import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button } from 'antd';
import { RocketOutlined, SearchOutlined, ToolOutlined, CheckCircleOutlined } from '@ant-design/icons';

interface OnboardingProps {
  visible: boolean;
  onComplete: () => void;
  gamePath: string;
  smapiInstalled: boolean;
  onDetectGame: () => void;
  onInstallSmapi: () => void;
}

const steps = [
  { icon: <RocketOutlined />, titleKey: 'app.onboarding.step1Title', descKey: 'app.onboarding.step1Desc' },
  { icon: <SearchOutlined />, titleKey: 'app.onboarding.step2Title', descKey: 'app.onboarding.step2Desc' },
  { icon: <ToolOutlined />, titleKey: 'app.onboarding.step3Title', descKey: 'app.onboarding.step3Desc' },
  { icon: <CheckCircleOutlined />, titleKey: 'app.onboarding.step4Title', descKey: 'app.onboarding.step4Desc' },
];

export default function Onboarding({ visible, onComplete, gamePath, smapiInstalled, onDetectGame, onInstallSmapi }: OnboardingProps) {
  const { t } = useTranslation();
  const [current, setCurrent] = useState(0);

  const handleNext = () => {
    if (current < steps.length - 1) {
      setCurrent(current + 1);
    } else {
      onComplete();
    }
  };

  const handlePrev = () => {
    if (current > 0) {
      setCurrent(current - 1);
    }
  };

  const step = steps[current];

  return (
    <Modal
      open={visible}
      footer={null}
      closable={false}
      centered
      width={520}
      className="svl-onboarding-modal"
    >
      <div className="svl-onboarding">
        <div className="svl-onboarding-steps">
          {steps.map((s, i) => (
            <div
              key={i}
              className={`svl-onboarding-step ${i === current ? 'active' : ''} ${i < current ? 'done' : ''}`}
            >
              <div className="svl-onboarding-step-icon">{s.icon}</div>
            </div>
          ))}
        </div>

        <div className="svl-onboarding-content">
          <h2>{t(step.titleKey)}</h2>
          <p>{t(step.descKey)}</p>

          {current === 1 && !gamePath && (
            <Button type="primary" onClick={onDetectGame} block>
              {t('app.pages.modManager.detectGame')}
            </Button>
          )}

          {current === 2 && !smapiInstalled && (
            <Button type="primary" onClick={onInstallSmapi} block>
              {t('app.pages.modManager.downloadSmapi')}
            </Button>
          )}
        </div>

        <div className="svl-onboarding-footer">
          <Button type="link" onClick={onComplete}>
            {t('app.onboarding.skip')}
          </Button>
          <div className="svl-onboarding-actions">
            {current > 0 && (
              <Button onClick={handlePrev}>
                {t('app.onboarding.prev')}
              </Button>
            )}
            <Button type="primary" onClick={handleNext}>
              {current === steps.length - 1
                ? t('app.onboarding.done')
                : t('app.onboarding.next')}
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
