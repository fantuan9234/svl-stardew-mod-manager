import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Modal, Button, message, Spin, Input, Switch, InputNumber, Form, Alert, Typography, Space } from 'antd';
import { EditOutlined, SaveOutlined, ReloadOutlined } from '@ant-design/icons';
import { readModConfig, updateModConfig, type ModConfigSchema, type ConfigField } from '../utils/advanced-features-api';

const { Text } = Typography;

interface ModConfigEditorProps {
  visible: boolean;
  onClose: () => void;
  modPath: string;
  onConfigUpdated: () => void;
}

function renderFieldValue(field: ConfigField, onChange: (key: string, value: string | number | boolean) => void, t: (key: string, options?: Record<string, unknown>) => string) {
  const rawValue = field.value.value;
  switch (field.field_type) {
    case 'Bool':
      return (
        <Switch
          checked={rawValue === true}
          onChange={(checked) => onChange(field.key, checked)}
        />
      );
    case 'Number':
      return (
        <InputNumber
          value={typeof rawValue === 'number' ? rawValue : 0}
          onChange={(val) => { if (val !== null) onChange(field.key, val); }}
          style={{ width: '100%' }}
        />
      );
    case 'String':
      return (
        <Input
          value={typeof rawValue === 'string' ? rawValue : String(rawValue ?? '')}
          onChange={(e) => onChange(field.key, e.target.value)}
        />
      );
    case 'Array':
      return (
        <Text type="secondary" style={{ fontSize: 12 }}>
          {t('app.arrayNotSupported', { count: Array.isArray(rawValue) ? rawValue.length : 0 })}
        </Text>
      );
    case 'Object':
      return (
        <Text type="secondary" style={{ fontSize: 12 }}>
          {t('app.objectNotSupported', { count: typeof rawValue === 'object' && rawValue !== null && !Array.isArray(rawValue) ? Object.keys(rawValue as Record<string, unknown>).length : 0 })}
        </Text>
      );
    default:
      return (
        <Input
          value={typeof rawValue === 'string' ? rawValue : String(rawValue ?? '')}
          onChange={(e) => onChange(field.key, e.target.value)}
        />
      );
  }
}

export default function ModConfigEditor({ visible, onClose, modPath, onConfigUpdated }: ModConfigEditorProps) {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [config, setConfig] = useState<ModConfigSchema | null>(null);
  const [changes, setChanges] = useState<Record<string, string | number | boolean>>({});

  const loadConfig = useCallback(async () => {
    setLoading(true);
    setChanges({});
    try {
      const result = await readModConfig(modPath);
      setConfig(result);
    } catch (err) {
      const errMsg = typeof err === 'string' ? err : String(err);
      if (errMsg.includes('No config.json')) {
        message.info(t('features.configEditor.noConfigFile'));
      } else {
        message.error(errMsg);
      }
      setConfig(null);
    } finally {
      setLoading(false);
    }
  }, [modPath, t]);

  useEffect(() => {
    if (visible && modPath) {
      loadConfig();
    }
  }, [visible, modPath, loadConfig]);

  const handleChange = useCallback((key: string, value: string | number | boolean) => {
    setChanges(prev => ({ ...prev, [key]: value }));
  }, []);

  const handleSave = useCallback(async () => {
    if (!config || Object.keys(changes).length === 0) return;
    setSaving(true);
    try {
      const updates = Object.entries(changes).map(([key, value]) => ({ key, value }));
      const result = await updateModConfig(modPath, updates);
      message.success(result.message);
      setChanges({});
      onConfigUpdated();
      loadConfig();
    } catch (err) {
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setSaving(false);
    }
  }, [config, changes, modPath, onConfigUpdated, loadConfig, t]);

  const handleCancel = useCallback(() => {
    setConfig(null);
    setChanges({});
    onClose();
  }, [onClose]);

  return (
    <Modal
      title={t('features.configEditor.title')}
      open={visible}
      onCancel={handleCancel}
      width={600}
      footer={
        <div style={{ display: 'flex', gap: 8, justifyContent: 'space-between' }}>
          <Button icon={<ReloadOutlined />} onClick={loadConfig} loading={loading}>
            {t('features.configEditor.reload')}
          </Button>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button onClick={handleCancel}>{t('app.common.cancel')}</Button>
            <Button
              type="primary"
              icon={<SaveOutlined />}
              onClick={handleSave}
              loading={saving}
              disabled={Object.keys(changes).length === 0}
            >
              {t('features.configEditor.saveChanges')}
            </Button>
          </div>
        </div>
      }
    >
      <Spin spinning={loading}>
        {config && config.fields.length > 0 && (
          <div>
            {Object.keys(changes).length > 0 && (
              <Alert
                message={t('features.configEditor.unsavedChanges', { count: Object.keys(changes).length })}
                type="warning"
                showIcon
                style={{ marginBottom: 16 }}
              />
            )}

            <Form layout="vertical" size="small">
              {config.fields.map((field) => {
                const isModified = changes.hasOwnProperty(field.key);
                return (
                  <Form.Item
                    key={field.key}
                    label={
                      <Space>
                        <Text strong>{field.key}</Text>
                        <Text type="secondary" style={{ fontSize: 11 }}>({field.field_type})</Text>
                        {isModified && <Text type="warning" style={{ fontSize: 11 }}>● {t('app.modified')}</Text>}
                      </Space>
                    }
                  >
                    {renderFieldValue(
                      isModified
                        ? { ...field, value: { type: field.field_type, value: changes[field.key] } }
                        : field,
                      handleChange,
                      t,
                    )}
                  </Form.Item>
                );
              })}
            </Form>
          </div>
        )}

        {config && config.fields.length === 0 && (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <EditOutlined style={{ fontSize: 48, color: '#faad14' }} />
            <p style={{ marginTop: 16, color: 'var(--svl-text-secondary)' }}>
              {t('features.configEditor.emptyConfig')}
            </p>
          </div>
        )}
      </Spin>
    </Modal>
  );
}
