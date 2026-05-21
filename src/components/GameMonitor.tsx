import { useState, useEffect, useCallback, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button, Progress, Badge, List, Tag, Typography, Spin, Statistic, Row, Col, message } from 'antd';
import { PlayCircleOutlined, PauseCircleOutlined, CheckCircleOutlined, WarningOutlined, CloseCircleOutlined, DashboardOutlined } from '@ant-design/icons';
import { startGameMonitor, stopGameMonitor, getMonitorStatus, listenToMonitorUpdates, type ModMonitorStatus } from '../utils/advanced-features-api';

const { Text } = Typography;

interface GameMonitorProps {
  visible: boolean;
  onClose: () => void;
  totalMods: number;
}

export default function GameMonitor({ visible, onClose, totalMods }: GameMonitorProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<ModMonitorStatus | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  const loadStatus = useCallback(async () => {
    try {
      const result = await getMonitorStatus(totalMods);
      setStatus(result);
    } catch {
      // ignore
    }
  }, [totalMods]);

  const handleStart = useCallback(async () => {
    setLoading(true);
    try {
      await startGameMonitor();
      await loadStatus();

      if (unlistenRef.current) {
        unlistenRef.current();
      }

      unlistenRef.current = await listenToMonitorUpdates(() => {
        loadStatus();
      });
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setLoading(false);
    }
  }, [loadStatus, t]);

  const handleStop = useCallback(async () => {
    try {
      await stopGameMonitor();
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
      await loadStatus();
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    }
  }, [loadStatus, t]);

  useEffect(() => {
    if (visible) {
      loadStatus();
    }
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    };
  }, [visible, loadStatus]);

  const getHealthColor = (score: number) => {
    if (score >= 90) return '#52c41a';
    if (score >= 70) return '#faad14';
    if (score >= 50) return '#fa8c16';
    return '#ff4d4f';
  };

  const getHealthLabel = (score: number) => {
    if (score >= 90) return t('features.monitor.healthExcellent');
    if (score >= 70) return t('features.monitor.healthGood');
    if (score >= 50) return t('features.monitor.healthFair');
    return t('features.monitor.healthPoor');
  };

  return (
    <Modal
      title={t('features.monitor.title')}
      open={visible}
      onCancel={onClose}
      width={800}
      footer={
        <div style={{ display: 'flex', gap: 8, justifyContent: 'flex-end' }}>
          <Button onClick={onClose}>{t('app.common.close')}</Button>
          {status?.is_game_running ? (
            <Button icon={<PauseCircleOutlined />} onClick={handleStop} danger>
              {t('features.monitor.stopMonitor')}
            </Button>
          ) : (
            <Button type="primary" icon={<PlayCircleOutlined />} onClick={handleStart} loading={loading}>
              {t('features.monitor.startMonitor')}
            </Button>
          )}
        </div>
      }
    >
      <Spin spinning={loading}>
        {status && (
          <div>
            <Row gutter={16} style={{ marginBottom: 24 }}>
              <Col span={6}>
                <Statistic
                  title={t('features.monitor.loadedMods')}
                  value={status.loaded_mods}
                  suffix={`/ ${status.total_mods}`}
                  valueStyle={{ color: '#1677ff' }}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title={t('features.monitor.errors')}
                  value={status.error_count}
                  valueStyle={{ color: status.error_count > 0 ? '#ff4d4f' : '#52c41a' }}
                  prefix={<CloseCircleOutlined />}
                />
              </Col>
              <Col span={6}>
                <Statistic
                  title={t('features.monitor.warnings')}
                  value={status.warning_count}
                  valueStyle={{ color: status.warning_count > 0 ? '#faad14' : '#52c41a' }}
                  prefix={<WarningOutlined />}
                />
              </Col>
              <Col span={6}>
                <div style={{ textAlign: 'center' }}>
                  <Text type="secondary">{t('features.monitor.healthScore')}</Text>
                  <Progress
                    type="dashboard"
                    percent={Math.round(status.health_score)}
                    strokeColor={getHealthColor(status.health_score)}
                    format={() => getHealthLabel(status.health_score)}
                    style={{ marginTop: 8 }}
                  />
                </div>
              </Col>
            </Row>

            {status.error_events.length > 0 && (
              <div style={{ marginBottom: 16 }}>
                <Text strong style={{ display: 'block', marginBottom: 8 }}>
                  <CloseCircleOutlined style={{ color: '#ff4d4f', marginRight: 8 }} />
                  {t('features.monitor.errorEvents')} ({status.error_events.length})
                </Text>
                <List
                  dataSource={status.error_events.slice(0, 10)}
                  size="small"
                  renderItem={(event) => (
                    <List.Item style={{ padding: '4px 0' }}>
                      <div>
                        <Badge style={{ color: 'var(--svl-error)' }} />
                        <Text strong style={{ marginLeft: 8 }}>{event.mod_name}</Text>
                        <Text type="secondary" style={{ marginLeft: 8, fontSize: 12 }}>
                          {new Date(event.timestamp).toLocaleTimeString()}
                        </Text>
                      </div>
                      <Text type="danger" style={{ fontSize: 12, display: 'block', marginTop: 4 }}>
                        {event.error_message}
                      </Text>
                    </List.Item>
                  )}
                />
              </div>
            )}

            {status.warning_events.length > 0 && (
              <div style={{ marginBottom: 16 }}>
                <Text strong style={{ display: 'block', marginBottom: 8 }}>
                  <WarningOutlined style={{ color: '#faad14', marginRight: 8 }} />
                  {t('features.monitor.warningEvents')} ({status.warning_events.length})
                </Text>
                <List
                  dataSource={status.warning_events.slice(0, 10)}
                  size="small"
                  renderItem={(event) => (
                    <List.Item style={{ padding: '4px 0' }}>
                      <div>
                        <Badge style={{ color: 'var(--svl-warning)' }} />
                        <Text strong style={{ marginLeft: 8 }}>{event.mod_name}</Text>
                        <Text type="secondary" style={{ marginLeft: 8, fontSize: 12 }}>
                          {new Date(event.timestamp).toLocaleTimeString()}
                        </Text>
                      </div>
                    </List.Item>
                  )}
                />
              </div>
            )}

            {status.mod_load_events.length > 0 && (
              <div>
                <Text strong style={{ display: 'block', marginBottom: 8 }}>
                  <CheckCircleOutlined style={{ color: '#52c41a', marginRight: 8 }} />
                  {t('features.monitor.loadEvents')} ({status.mod_load_events.length})
                </Text>
                <List
                  dataSource={status.mod_load_events.slice(-20).reverse()}
                  size="small"
                  renderItem={(event) => (
                    <List.Item style={{ padding: '4px 0' }}>
                      <Badge style={{ color: 'var(--svl-success)' }} />
                      <Text style={{ marginLeft: 8 }}>{event.mod_name}</Text>
                      <Tag className="svl-tag-info" style={{ marginLeft: 8 }}>{event.unique_id}</Tag>
                    </List.Item>
                  )}
                />
              </div>
            )}

            {status.mod_load_events.length === 0 && status.error_events.length === 0 && status.warning_events.length === 0 && (
              <div style={{ textAlign: 'center', padding: '40px 0' }}>
                <DashboardOutlined style={{ fontSize: 48, color: 'var(--svl-text-tertiary)' }} />
                <p style={{ marginTop: 16, color: 'var(--svl-text-secondary)' }}>
                  {t('features.monitor.noEvents')}
                </p>
              </div>
            )}
          </div>
        )}
      </Spin>
    </Modal>
  );
}
