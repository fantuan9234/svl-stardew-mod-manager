import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button, message, Spin, Tag, Alert, List, Typography } from 'antd';
import {
  SortAscendingOutlined,
  CheckCircleOutlined,
  BulbOutlined,
  ArrowUpOutlined,
} from '@ant-design/icons';
import { calculateOptimalLoadOrder, applyLoadOrder, type LoadOrderReport } from '../utils/advanced-features-api';
import type { ModInfo } from '../utils/tauri-api';

const { Text } = Typography;

interface LoadOrderModalProps {
  visible: boolean;
  onClose: () => void;
  mods: ModInfo[];
  gamePath: string;
  onOrderApplied: () => void;
}

const layerColors: Record<string, string> = {
  Framework: 'purple',
  Library: 'blue',
  Core: 'green',
  Content: 'orange',
  Expansion: 'magenta',
  Override: 'red',
};

const layerKeys: string[] = ['Framework', 'Library', 'Core', 'Content', 'Expansion', 'Override'];

function getLayerLabel(key: string, t: (key: string) => string): string {
  const map: Record<string, string> = {
    Framework: t('app.features.loadOrder.layerFramework'),
    Library: t('app.features.loadOrder.layerLibrary'),
    Core: t('app.features.loadOrder.layerCore'),
    Content: t('app.features.loadOrder.layerContent'),
    Expansion: t('app.features.loadOrder.layerExpansion'),
    Override: t('app.features.loadOrder.layerOverride'),
  };
  return map[key] || key;
}

export default function LoadOrderModal({ visible, onClose, mods, gamePath, onOrderApplied }: LoadOrderModalProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [applying, setApplying] = useState(false);
  const [report, setReport] = useState<LoadOrderReport | null>(null);

  const handleCalculate = useCallback(async () => {
    setLoading(true);
    try {
      const result = await calculateOptimalLoadOrder(mods);
      setReport(result);
      message.success(t('features.loadOrder.calculationComplete'));
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setLoading(false);
    }
  }, [mods, t]);

  const handleApply = useCallback(async () => {
    if (!report) return;
    setApplying(true);
    try {
      const order = report.ordered_mods.map(m => m.unique_id);
      const result = await applyLoadOrder(gamePath, order);
      message.success(result.message);
      onOrderApplied();
      onClose();
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setApplying(false);
    }
  }, [report, gamePath, onOrderApplied, onClose, t]);

  const handleClose = useCallback(() => {
    setReport(null);
    onClose();
  }, [onClose]);

  return (
    <Modal
      title={t('features.loadOrder.title')}
      open={visible}
      onCancel={handleClose}
      width={800}
      footer={
        <div style={{ display: 'flex', gap: 12, justifyContent: 'space-between' }}>
          <Button icon={<SortAscendingOutlined />} onClick={handleCalculate} loading={loading} disabled={loading}>
            {report ? t('features.loadOrder.recalculate') : t('features.loadOrder.calculate')}
          </Button>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button onClick={handleClose}>{t('app.common.close')}</Button>
            <Button
              type="primary"
              icon={<CheckCircleOutlined />}
              onClick={handleApply}
              loading={applying}
              disabled={!report || applying}
            >
              {t('features.loadOrder.applyOrder')}
            </Button>
          </div>
        </div>
      }
    >
      <Spin spinning={loading}>
        {!report && !loading && (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <ArrowUpOutlined style={{ fontSize: 48, color: '#1677ff' }} />
            <p style={{ marginTop: 16, color: 'var(--svl-text-secondary)' }}>
              {t('features.loadOrder.description')}
            </p>
            <Button type="primary" icon={<SortAscendingOutlined />} onClick={handleCalculate}>
              {t('features.loadOrder.calculate')}
            </Button>
          </div>
        )}

        {report && (
          <div>
            {report.conflicts.length > 0 && (
              <Alert
                message={t('features.loadOrder.conflictsFound', { count: report.conflicts.length })}
                type="error"
                showIcon
                style={{ marginBottom: 16 }}
              />
            )}

            {report.suggestions.map((s, i) => (
              <Alert
                key={i}
                message={s}
                type="info"
                icon={<BulbOutlined />}
                style={{ marginBottom: 8 }}
              />
            ))}

            <div style={{ marginTop: 16, marginBottom: 8, display: 'flex', gap: 8, flexWrap: 'wrap' }}>
              {layerKeys.map((key) => (
                <Tag key={key} color={layerColors[key]}>
                  {getLayerLabel(key, t)}
                </Tag>
              ))}
            </div>

            <List
              dataSource={report.ordered_mods}
              size="small"
              renderItem={(item) => (
                <List.Item style={{ padding: '6px 0' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12, width: '100%' }}>
                    <Tag color={layerColors[item.layer] || 'default'} style={{ minWidth: 80, textAlign: 'center' }}>
                      {getLayerLabel(item.layer, t)}
                    </Tag>
                    <Text strong style={{ flex: 1 }}>{item.name}</Text>
                    <Text type="secondary" style={{ fontSize: 12 }}>{item.unique_id}</Text>
                  </div>
                </List.Item>
              )}
            />
          </div>
        )}
      </Spin>
    </Modal>
  );
}
