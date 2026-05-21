import { useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Input, Button, Tag, message, Typography, Divider, Tooltip } from 'antd';
import { LockOutlined, LinkOutlined, CheckCircleOutlined, LoadingOutlined, DisconnectOutlined, QuestionCircleOutlined, ExportOutlined } from '@ant-design/icons';
import { openUrl } from '../utils/openUrl';
import { registerNxmProtocol } from '../utils/tauri-api';
import { useNexusStatus, verifyNexusConnection } from '../hooks/useNexusStatus';

const { Text } = Typography;

const NEXUS_API_KEY_URL = 'https://next.nexusmods.com/settings/api-keys';

export default function NexusApiConfig() {
  const { t } = useTranslation();
  const { status, isPremium, reconnect, disconnect } = useNexusStatus();
  const [apiKey, setApiKey] = useState(() => localStorage.getItem('svl-nexus-api-key') || '');
  const [nxmRegistered, setNxmRegistered] = useState(() => localStorage.getItem('svl-nxm-registered') === 'true');
  const [registering, setRegistering] = useState(false);
  const mountedRef = useRef(true);

  const handleConnect = async () => {
    if (!apiKey.trim()) {
      message.warning(t('app.nexusApi.enterApiKey'));
      return;
    }
    const key = apiKey.trim();
    localStorage.setItem('svl-nexus-api-key', key);
    await verifyNexusConnection(key);
  };

  const handleDisconnect = () => {
    disconnect();
    setApiKey('');
    message.info(t('app.nexusApi.disconnected'));
  };

  const handleReconnect = async () => {
    const key = apiKey.trim() || localStorage.getItem('svl-nexus-api-key');
    if (!key) {
      message.warning(t('app.nexusApi.enterApiKey'));
      return;
    }
    await reconnect();
  };

  const handleRegisterNxm = async () => {
    setRegistering(true);
    try {
      const result = await registerNxmProtocol();
      if (!mountedRef.current) return;
      if (result.success) {
        setNxmRegistered(true);
        localStorage.setItem('svl-nxm-registered', 'true');
        message.success(t('app.nexusApi.nxmRegistered'));
      } else {
        message.warning(result.message);
      }
    } catch (err: any) {
      if (!mountedRef.current) return;
      message.error(err.message || t('app.nexusApi.nxmRegisterFailed'));
    } finally {
      if (mountedRef.current) {
        setRegistering(false);
      }
    }
  };

  const handleOpenDocs = async () => {
    try {
      await openUrl(NEXUS_API_KEY_URL);
    } catch {
      message.error(t('app.nexusApi.openUrlFailed'));
    }
  };

  const getStatusTag = () => {
    switch (status) {
      case 'checking':
        return (
          <>
            <Tag icon={<LoadingOutlined spin />} color="processing">{t('app.nexusApi.verifying')}</Tag>
            <Text type="secondary" style={{ fontSize: 12, marginLeft: 8 }}>{t('app.nexusApi.verifyingHint')}</Text>
          </>
        );
      case 'connected':
        return <Tag icon={<CheckCircleOutlined />} className="svl-tag-success">
          {t('app.nexusApi.connected')}
          {isPremium && <Tag className="svl-tag-warning" style={{ marginLeft: 8 }}>{t('app.nexusApi.premium')}</Tag>}
        </Tag>;
      default:
        return <Tag color="default">{t('app.nexusApi.disconnected')}</Tag>;
    }
  };

  return (
    <div className="svl-nexus-config">
      <div className="svl-nexus-config-header">
        <div className="svl-nexus-config-title">
          <span className="svl-nexus-icon">🎮</span>
          <Text strong>{t('app.nexusApi.title')}</Text>
        </div>
        {getStatusTag()}
      </div>

      <div className="svl-nexus-config-body">
        <div className="svl-nexus-input-group">
          <label>{t('app.nexusApi.apiKeyLabel')}</label>
          <div style={{ display: 'flex', gap: 8 }}>
            <Input.Password
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder={t('app.nexusApi.apiKeyPlaceholder')}
              prefix={<LockOutlined />}
              disabled={status === 'checking'}
              className="svl-nexus-api-input"
              style={{ flex: 1 }}
            />
            <Tooltip title={t('app.nexusApi.getApiKeyTooltip')}>
              <Button
                icon={<QuestionCircleOutlined />}
                onClick={handleOpenDocs}
              />
            </Tooltip>
          </div>
        </div>

        <div className="svl-nexus-actions">
          {status === 'connected' ? (
            <Button
              danger
              icon={<DisconnectOutlined />}
              onClick={handleDisconnect}
              className="svl-nexus-btn"
            >
              {t('app.nexusApi.disconnect')}
            </Button>
          ) : (
            <Button
              type="primary"
              icon={<LinkOutlined />}
              onClick={handleConnect}
              loading={status === 'checking'}
              disabled={status === 'checking'}
              className="svl-nexus-btn svl-nexus-btn-connect"
            >
              {t('app.nexusApi.connect')}
            </Button>
          )}
          {status === 'connected' && (
            <Button
              icon={<LinkOutlined />}
              onClick={handleReconnect}
              className="svl-nexus-btn"
              style={{ marginLeft: 8 }}
            >
              {t('app.nexusApi.reconnect')}
            </Button>
          )}
        </div>

        <Divider style={{ margin: '16px 0' }} />

        <div className="svl-nexus-protocol-section">
          <div className="svl-nexus-protocol-header">
            <Text strong style={{ fontSize: 14 }}>{t('app.nexusApi.nxmProtocol')}</Text>
            {nxmRegistered && (
              <Tag icon={<CheckCircleOutlined />} className="svl-tag-success" style={{ marginLeft: 8 }}>
                {t('app.nexusApi.nxmRegistered')}
              </Tag>
            )}
          </div>
          <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 12 }}>
            {t('app.nexusApi.nxmProtocolDesc')}
          </Text>
          <Button
            type="default"
            onClick={handleRegisterNxm}
            loading={registering}
            disabled={nxmRegistered}
            className="svl-nexus-btn svl-nexus-btn-protocol"
          >
            {nxmRegistered ? t('app.nexusApi.nxmAlreadyRegistered') : t('app.nexusApi.registerNxm')}
          </Button>
        </div>

        <div className="svl-nexus-help">
          <div className="svl-nexus-help-title">
            <QuestionCircleOutlined style={{ marginRight: 6 }} />
            {t('app.nexusApi.getApiKeyHelp')}
          </div>
          <ol className="svl-nexus-help-steps">
            <li>{t('app.nexusApi.step1')}</li>
            <li>{t('app.nexusApi.step2')}</li>
            <li>{t('app.nexusApi.step3')}</li>
            <li>{t('app.nexusApi.step4')}</li>
          </ol>
          <Button
            type="primary"
            icon={<ExportOutlined />}
            onClick={handleOpenDocs}
            className="svl-nexus-btn svl-nexus-btn-link"
            style={{ marginTop: 8 }}
          >
            {t('app.nexusApi.openApiKeyPage')}
          </Button>
        </div>
      </div>
    </div>
  );
}
