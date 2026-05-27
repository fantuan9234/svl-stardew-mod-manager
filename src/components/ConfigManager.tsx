import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Space, Spin, Empty, Typography, Input, Switch, InputNumber, message, Tag } from 'antd';
import { ArrowLeftOutlined, SaveOutlined, CheckOutlined } from '@ant-design/icons';
import {
  listModConfigs,
  readModConfig,
  updateModConfig,
  detectGamePath,
  type ModConfigListItem,
  type ModConfigListResult,
} from '../utils/tauri-api';

const { Text, Title } = Typography;

const ConfigIconSvg = ({ color, size = 20 }: { color: string; size?: number }) => (
  <svg width={size} height={size} viewBox="0 0 32 32" fill="none">
    <path d="M16 4l2.5 5 5.5 0.8-4 3.9 0.9 5.5L16 16.5l-4.9 2.6 0.9-5.5-4-3.9 5.5-0.8L16 4z" fill={color} opacity="0.15" stroke={color} strokeWidth="1.5" strokeLinejoin="round"/>
    <circle cx="16" cy="12" r="3" fill={color} opacity="0.3" stroke={color} strokeWidth="1.2"/>
    <path d="M16 19v8M12 23h8" stroke={color} strokeWidth="1.5" strokeLinecap="round" opacity="0.5"/>
  </svg>
);

interface ConfigField {
  key: string;
  value: any;
  field_type: string;
  description: string;
}

interface ModConfigSchema {
  mod_name: string;
  unique_id: string;
  config_path: string;
  fields: ConfigField[];
}

export default function ConfigManager({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [configs, setConfigs] = useState<ModConfigListItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [scanResult, setScanResult] = useState<ModConfigListResult | null>(null);
  const [selectedMod, setSelectedMod] = useState<ModConfigSchema | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [editedValues, setEditedValues] = useState<Record<string, any>>({});
  const [saving, setSaving] = useState(false);

  const doScan = async () => {
    setLoading(true);
    setSelectedMod(null);
    try {
      const pathInfo = await detectGamePath();
      const gamePath = pathInfo.detected_path;
      if (!gamePath) {
        message.warning(t('app.toolbox.configNeedGamePath'));
        return;
      }
      const result = await listModConfigs(gamePath + '\\Mods');
      setConfigs(result.configs);
      setScanResult(result);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.configScanFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleViewConfig = async (item: ModConfigListItem) => {
    setLoadingDetail(true);
    setEditedValues({});
    try {
      const schema = await readModConfig(item.folder_path);
      setSelectedMod(schema);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.configReadFailed'));
    } finally {
      setLoadingDetail(false);
    }
  };

  const unwrapConfigValue = (val: any): any => {
    if (val === null || val === undefined) return val;
    if (typeof val === 'object' && val !== null && 'type' in val && 'value' in val) {
      return val.value;
    }
    return val;
  };

  const handleValueChange = (key: string, value: any) => {
    setEditedValues(prev => ({ ...prev, [key]: value }));
  };

  const handleSave = async () => {
    if (!selectedMod || Object.keys(editedValues).length === 0) {
      message.info(t('app.toolbox.configNoChanges'));
      return;
    }

    setSaving(true);
    try {
      const updates = Object.entries(editedValues).map(([key, value]) => ({ key, value }));
      const modFolderPath = selectedMod.config_path.replace(/[/\\]config\.json$/i, '');
      await updateModConfig(modFolderPath, updates);
      message.success(t('app.toolbox.configSaveSuccess'));

      const schema = await readModConfig(modFolderPath);
      setSelectedMod(schema);
      setEditedValues({});
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.configSaveFailed'));
    } finally {
      setSaving(false);
    }
  };

  const renderFieldEditor = (field: ConfigField) => {
    const raw = editedValues.hasOwnProperty(field.key) ? editedValues[field.key] : field.value;
    const currentValue = unwrapConfigValue(raw);

    switch (field.field_type) {
      case 'Bool':
        return (
          <Switch
            checked={currentValue === true}
            onChange={(checked) => handleValueChange(field.key, checked)}
          />
        );
      case 'Number':
        return (
          <InputNumber
            value={typeof currentValue === 'number' ? currentValue : 0}
            onChange={(val) => handleValueChange(field.key, val ?? 0)}
            style={{ width: 200 }}
          />
        );
      case 'String':
        return (
          <Input
            value={typeof currentValue === 'string' ? currentValue : ''}
            onChange={(e) => handleValueChange(field.key, e.target.value)}
            style={{ width: 300 }}
          />
        );
      default:
        return (
          <Input.TextArea
            value={typeof currentValue === 'string' ? currentValue : JSON.stringify(currentValue, null, 2)}
            onChange={(e) => {
              try {
                handleValueChange(field.key, JSON.parse(e.target.value));
              } catch {
                handleValueChange(field.key, e.target.value);
              }
            }}
            rows={3}
            style={{ width: 400, fontFamily: 'monospace', fontSize: 12 }}
          />
        );
    }
  };

  const hasChanges = Object.keys(editedValues).length > 0;

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column', padding: '24px 28px', maxWidth: 1200, margin: '0 auto', overflow: 'hidden' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 20, flexShrink: 0 }}>
        <button
          onClick={onBack}
          style={{
            width: 36, height: 36, borderRadius: 10,
            border: '1px solid rgba(139,115,85,0.2)',
            background: 'rgba(61,50,37,0.5)',
            color: 'var(--svl-text-secondary)',
            cursor: 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            transition: 'all 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'rgba(61,50,37,0.8)';
            e.currentTarget.style.borderColor = 'rgba(139,115,85,0.4)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'rgba(61,50,37,0.5)';
            e.currentTarget.style.borderColor = 'rgba(139,115,85,0.2)';
          }}
        >
          <ArrowLeftOutlined />
        </button>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <ConfigIconSvg color="#c49a3b" size={22} />
          <Title level={4} style={{ margin: 0, fontWeight: 600 }}>{t('app.toolbox.configTitle')}</Title>
        </div>
      </div>

      <Button
        type="primary"
        icon={<ConfigIconSvg color="#fff" size={16} />}
        onClick={doScan}
        loading={loading}
        style={{
          marginBottom: 16,
          background: 'linear-gradient(135deg, #c49a3b, #d4aa4a)',
          border: 'none',
          borderRadius: 10,
          height: 36,
          padding: '0 20px',
          fontWeight: 500,
          alignSelf: 'flex-start',
          flexShrink: 0,
        }}
      >
        {t('app.toolbox.configScan')}
      </Button>

      {loading && (
        <div style={{ textAlign: 'center', padding: '80px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 16, color: 'var(--svl-text-muted)', fontSize: 14 }}>
            {t('app.toolbox.configScanning')}
          </div>
        </div>
      )}

      {!loading && scanResult && (
        <div style={{ display: 'flex', gap: 16, flex: 1, minHeight: 0 }}>
          <div style={{ flex: '0 0 320px', display: 'flex', flexDirection: 'column', minHeight: 0 }}>
            <div style={{
              borderRadius: 12,
              padding: '12px 16px',
              background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
              border: '1px solid rgba(139,115,85,0.12)',
              marginBottom: 12,
              flexShrink: 0,
            }}>
              <Text type="secondary" style={{ fontSize: 13 }}>
                {t('app.toolbox.configFound', { count: scanResult.total_mods_with_config, total: scanResult.total_mods_scanned })}
              </Text>
            </div>
            <div style={{ flex: 1, overflow: 'auto' }}>
              {configs.map((item) => (
                <div
                  key={item.unique_id || item.folder_path}
                  onClick={() => handleViewConfig(item)}
                  style={{
                    marginBottom: 8,
                    cursor: 'pointer',
                    borderRadius: 12,
                    padding: '12px 16px',
                    background: selectedMod?.unique_id === item.unique_id
                      ? 'linear-gradient(145deg, rgba(196,154,59,0.15), rgba(196,154,59,0.05))'
                      : 'linear-gradient(145deg, rgba(61,50,37,0.5), rgba(45,36,24,0.3))',
                    border: selectedMod?.unique_id === item.unique_id
                      ? '1px solid rgba(196,154,59,0.3)'
                      : '1px solid rgba(139,115,85,0.1)',
                    transition: 'all 0.2s',
                  }}
                  onMouseEnter={(e) => {
                    if (selectedMod?.unique_id !== item.unique_id) {
                      e.currentTarget.style.background = 'linear-gradient(145deg, rgba(61,50,37,0.7), rgba(45,36,24,0.5))';
                      e.currentTarget.style.borderColor = 'rgba(139,115,85,0.2)';
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (selectedMod?.unique_id !== item.unique_id) {
                      e.currentTarget.style.background = 'linear-gradient(145deg, rgba(61,50,37,0.5), rgba(45,36,24,0.3))';
                      e.currentTarget.style.borderColor = 'rgba(139,115,85,0.1)';
                    }
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <Text strong ellipsis style={{ display: 'block', fontSize: 14 }}>{item.mod_name}</Text>
                      <Text type="secondary" style={{ fontSize: 11 }}>{item.unique_id}</Text>
                    </div>
                    <Tag style={{ flexShrink: 0, fontSize: 11, padding: '0 8px', borderRadius: 8, background: 'rgba(196,154,59,0.15)', color: '#c49a3b', border: 'none' }}>
                      {item.field_count}
                    </Tag>
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
            {!selectedMod && !loadingDetail && (
              <Empty description={t('app.toolbox.configSelectHint')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
            )}

            {loadingDetail && (
              <div style={{ textAlign: 'center', padding: 40 }}>
                <Spin />
              </div>
            )}

            {selectedMod && !loadingDetail && (
              <div style={{
                borderRadius: 14,
                background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
                border: '1px solid rgba(139,115,85,0.12)',
                display: 'flex',
                flexDirection: 'column',
                flex: 1,
                minHeight: 0,
                overflow: 'hidden',
              }}>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '16px 20px',
                  borderBottom: '1px solid rgba(139,115,85,0.12)',
                  flexShrink: 0,
                }}>
                  <Space>
                    <ConfigIconSvg color="#c49a3b" size={18} />
                    <span style={{ fontWeight: 600, fontSize: 15 }}>{selectedMod.mod_name}</span>
                    <Tag style={{ fontSize: 11, borderRadius: 8, background: 'rgba(139,115,85,0.15)', color: 'var(--svl-text-secondary)', border: 'none' }}>
                      {selectedMod.unique_id}
                    </Tag>
                  </Space>
                  <Space>
                    {hasChanges && (
                      <Tag color="orange" style={{ fontSize: 11, borderRadius: 8 }}>{t('app.toolbox.configUnsaved')}</Tag>
                    )}
                    <Button
                      type="primary"
                      icon={<SaveOutlined />}
                      onClick={handleSave}
                      loading={saving}
                      disabled={!hasChanges}
                      size="small"
                      style={{
                        background: 'linear-gradient(135deg, #c49a3b, #d4aa4a)',
                        border: 'none',
                        borderRadius: 8,
                      }}
                    >
                      {t('app.toolbox.configSave')}
                    </Button>
                  </Space>
                </div>

                {selectedMod.fields.length === 0 ? (
                  <Empty description={t('app.toolbox.configEmpty')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
                ) : (
                  <div style={{ flex: 1, overflow: 'auto', padding: '8px 0' }}>
                    {selectedMod.fields.map((field) => (
                      <div
                        key={field.key}
                        style={{
                          display: 'flex',
                          alignItems: 'flex-start',
                          gap: 16,
                          padding: '12px 20px',
                          borderBottom: '1px solid rgba(139,115,85,0.08)',
                          transition: 'background 0.15s',
                        }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.background = 'rgba(196,154,59,0.04)';
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.background = 'transparent';
                        }}
                      >
                        <div style={{ flex: '0 0 160px', minWidth: 0, paddingTop: 4 }}>
                          <Text strong style={{ fontSize: 13, wordBreak: 'break-all' }}>{field.key}</Text>
                          <br />
                          <Tag style={{ fontSize: 10, borderRadius: 6, background: 'rgba(139,115,85,0.12)', color: 'var(--svl-text-muted)', border: 'none' }}>
                            {field.field_type}
                          </Tag>
                          {field.description && (
                            <>
                              <br />
                              <Text type="secondary" style={{ fontSize: 11, lineHeight: '1.4', display: 'block', marginTop: 4 }}>
                                {field.description}
                              </Text>
                            </>
                          )}
                        </div>
                        <div style={{ flex: 1, minWidth: 0, paddingTop: 2 }}>
                          {renderFieldEditor(field)}
                        </div>
                        {editedValues.hasOwnProperty(field.key) && (
                          <Tag color="green" icon={<CheckOutlined />} style={{ flexShrink: 0, borderRadius: 8, fontSize: 11, marginTop: 4 }} />
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      {!loading && !scanResult && (
        <Empty description={t('app.toolbox.configClickToScan')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}
    </div>
  );
}
