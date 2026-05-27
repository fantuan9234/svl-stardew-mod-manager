import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Table, Tag, Space, Spin, Empty, Progress, Typography, message } from 'antd';
import { ArrowLeftOutlined } from '@ant-design/icons';
import { scanMods, analyzeModStorage, type ModStorageInfo, type StorageAnalysisResult } from '../utils/tauri-api';

const { Text, Title } = Typography;

const StorageIconSvg = ({ color, size = 20 }: { color: string; size?: number }) => (
  <svg width={size} height={size} viewBox="0 0 32 32" fill="none">
    <rect x="3" y="7" width="26" height="6" rx="2" fill={color} opacity="0.25" stroke={color} strokeWidth="1.5"/>
    <rect x="3" y="14" width="26" height="6" rx="2" fill={color} opacity="0.4" stroke={color} strokeWidth="1.5"/>
    <rect x="3" y="21" width="26" height="6" rx="2" fill={color} opacity="0.55" stroke={color} strokeWidth="1.5"/>
    <circle cx="7.5" cy="10" r="1.2" fill={color}/>
    <circle cx="7.5" cy="17" r="1.2" fill={color}/>
    <circle cx="7.5" cy="24" r="1.2" fill={color}/>
  </svg>
);

export default function StorageAnalyzerView({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [analysis, setAnalysis] = useState<StorageAnalysisResult | null>(null);
  const [loading, setLoading] = useState(false);

  const doAnalyze = async () => {
    setLoading(true);
    try {
      const modsResult = await scanMods();
      const result = await analyzeModStorage(modsResult);
      setAnalysis(result);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.storageAnalysisFailed'));
    } finally {
      setLoading(false);
    }
  };

  const columns = [
    {
      title: t('app.toolbox.modName'),
      dataIndex: 'name',
      key: 'name',
      ellipsis: true,
      render: (name: string, record: ModStorageInfo) => (
        <Space>
          <Text style={{ fontWeight: 500 }}>{name}</Text>
          {!record.enabled && <Tag style={{ fontSize: 11, padding: '0 6px' }}>{t('app.toolbox.disabled')}</Tag>}
          {record.is_content_pack && <Tag color="blue" style={{ fontSize: 11, padding: '0 6px' }}>CP</Tag>}
        </Space>
      ),
    },
    {
      title: t('app.toolbox.size'),
      dataIndex: 'size_formatted',
      key: 'size_formatted',
      width: 110,
      sorter: (a: ModStorageInfo, b: ModStorageInfo) => a.size_bytes - b.size_bytes,
      defaultSortOrder: 'descend' as const,
    },
    {
      title: t('app.toolbox.fileCount'),
      dataIndex: 'file_count',
      key: 'file_count',
      width: 90,
      sorter: (a: ModStorageInfo, b: ModStorageInfo) => a.file_count - b.file_count,
    },
    {
      title: t('app.toolbox.version'),
      dataIndex: 'version',
      key: 'version',
      width: 90,
    },
  ];

  const enabledPercent = analysis && analysis.total_size_bytes > 0
    ? Math.round((analysis.enabled_size_bytes / analysis.total_size_bytes) * 100)
    : 0;

  return (
    <div style={{ padding: '24px 28px', maxWidth: 1200, margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 24 }}>
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
          <StorageIconSvg color="#6b9e3a" size={22} />
          <Title level={4} style={{ margin: 0, fontWeight: 600 }}>{t('app.toolbox.storageTitle')}</Title>
        </div>
      </div>

      <Button
        type="primary"
        icon={<StorageIconSvg color="#fff" size={16} />}
        onClick={doAnalyze}
        loading={loading}
        style={{
          marginBottom: 20,
          background: 'linear-gradient(135deg, #6b9e3a, #7db84a)',
          border: 'none',
          borderRadius: 10,
          height: 36,
          padding: '0 20px',
          fontWeight: 500,
        }}
      >
        {t('app.toolbox.analyzeStorage')}
      </Button>

      {loading && (
        <div style={{ textAlign: 'center', padding: '80px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 16, color: 'var(--svl-text-muted)', fontSize: 14 }}>
            {t('app.toolbox.analyzingStorage')}
          </div>
        </div>
      )}

      {!loading && analysis && (
        <>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 16, marginBottom: 24 }}>
            <div style={{
              borderRadius: 14,
              padding: '18px 20px',
              background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
              border: '1px solid rgba(139,115,85,0.12)',
            }}>
              <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.toolbox.totalSize')}</Text>
              <div style={{ fontSize: 26, fontWeight: 700, color: 'var(--svl-primary-light)', margin: '4px 0' }}>
                {analysis.total_size_formatted}
              </div>
              <Text type="secondary" style={{ fontSize: 12 }}>{t('app.toolbox.totalMods', { count: analysis.total_mods })}</Text>
            </div>
            <div style={{
              borderRadius: 14,
              padding: '18px 20px',
              background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
              border: '1px solid rgba(107,158,58,0.15)',
            }}>
              <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.toolbox.enabledSize')}</Text>
              <div style={{ fontSize: 22, fontWeight: 600, color: '#6b9e3a', margin: '4px 0' }}>
                {analysis.enabled_size_formatted}
              </div>
              <Progress percent={enabledPercent} size="small" strokeColor="#6b9e3a" trailColor="rgba(139,115,85,0.15)" />
            </div>
            <div style={{
              borderRadius: 14,
              padding: '18px 20px',
              background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
              border: '1px solid rgba(139,115,85,0.12)',
            }}>
              <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.toolbox.disabledSize')}</Text>
              <div style={{ fontSize: 22, fontWeight: 600, color: 'var(--svl-text-muted)', margin: '4px 0' }}>
                {analysis.disabled_size_formatted}
              </div>
              <Progress percent={100 - enabledPercent} size="small" strokeColor="var(--svl-text-muted)" trailColor="rgba(139,115,85,0.15)" />
            </div>
            {analysis.largest_mod && (
              <div style={{
                borderRadius: 14,
                padding: '18px 20px',
                background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
                border: '1px solid rgba(199,80,80,0.15)',
              }}>
                <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.toolbox.largestMod')}</Text>
                <div style={{ fontSize: 16, fontWeight: 600, margin: '4px 0', color: 'var(--svl-text-primary)' }}>
                  {analysis.largest_mod.name}
                </div>
                <Text style={{ color: '#c75050', fontWeight: 600, fontSize: 14 }}>{analysis.largest_mod.size_formatted}</Text>
              </div>
            )}
          </div>
          <Table
            dataSource={analysis.mods}
            columns={columns}
            rowKey="unique_id"
            size="small"
            pagination={{ pageSize: 20 }}
            style={{ borderRadius: 12, overflow: 'hidden' }}
          />
        </>
      )}

      {!loading && !analysis && (
        <Empty description={t('app.toolbox.clickToAnalyze')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}
    </div>
  );
}
