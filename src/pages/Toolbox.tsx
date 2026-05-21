import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Table, Tag, Space, Spin, Empty, Progress, Card, Typography, message } from 'antd';
import { DatabaseOutlined } from '@ant-design/icons';
import {
  scanMods,
  analyzeModStorage,
  type ModStorageInfo,
  type StorageAnalysisResult,
} from '../utils/tauri-api';

const { Text, Title } = Typography;

export default function Toolbox() {
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
      title: t('app.toolbox.modName'), dataIndex: 'name', key: 'name', ellipsis: true,
      render: (name: string, record: ModStorageInfo) => (
        <Space>
          <Text>{name}</Text>
          {!record.enabled && <Tag>{t('app.toolbox.disabled')}</Tag>}
          {record.is_content_pack && <Tag color="blue">CP</Tag>}
        </Space>
      ),
    },
    { title: t('app.toolbox.size'), dataIndex: 'size_formatted', key: 'size_formatted', width: 110, sorter: (a: ModStorageInfo, b: ModStorageInfo) => a.size_bytes - b.size_bytes, defaultSortOrder: 'descend' as const },
    { title: t('app.toolbox.fileCount'), dataIndex: 'file_count', key: 'file_count', width: 90, sorter: (a: ModStorageInfo, b: ModStorageInfo) => a.file_count - b.file_count },
    { title: t('app.toolbox.version'), dataIndex: 'version', key: 'version', width: 90 },
  ];

  const enabledPercent = analysis && analysis.total_size_bytes > 0 ? Math.round((analysis.enabled_size_bytes / analysis.total_size_bytes) * 100) : 0;

  return (
    <div style={{ padding: '24px' }}>
      <Title level={3} style={{ marginBottom: 8 }}>📦 {t('app.toolbox.title')}</Title>
      <Text type="secondary" style={{ marginBottom: 24, display: 'block' }}>{t('app.toolbox.subtitle')}</Text>

      <Space style={{ marginBottom: 20 }}>
        <Button type="primary" size="large" icon={<DatabaseOutlined />} onClick={doAnalyze} loading={loading}>
          {t('app.toolbox.analyzeStorage')}
        </Button>
      </Space>

      {loading && (
        <div style={{ textAlign: 'center', padding: '80px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 16, color: 'var(--svl-text-muted)' }}>{t('app.toolbox.analyzingStorage')}</div>
        </div>
      )}

      {!loading && analysis && (
        <>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 16, marginBottom: 24 }}>
            <Card size="small">
              <Text type="secondary" style={{ fontSize: 12 }}>{t('app.toolbox.totalSize')}</Text>
              <div style={{ fontSize: 28, fontWeight: 700, color: 'var(--svl-primary)', margin: '4px 0' }}>{analysis.total_size_formatted}</div>
              <Text type="secondary" style={{ fontSize: 12 }}>{t('app.toolbox.totalMods', { count: analysis.total_mods })}</Text>
            </Card>
            <Card size="small">
              <Text type="secondary" style={{ fontSize: 12 }}>{t('app.toolbox.enabledSize')}</Text>
              <div style={{ fontSize: 22, fontWeight: 600, color: '#52c41a', margin: '4px 0' }}>{analysis.enabled_size_formatted}</div>
              <Progress percent={enabledPercent} size="small" strokeColor="#52c41a" />
            </Card>
            <Card size="small">
              <Text type="secondary" style={{ fontSize: 12 }}>{t('app.toolbox.disabledSize')}</Text>
              <div style={{ fontSize: 22, fontWeight: 600, color: 'var(--svl-text-muted)', margin: '4px 0' }}>{analysis.disabled_size_formatted}</div>
              <Progress percent={100 - enabledPercent} size="small" strokeColor="var(--svl-text-muted)" />
            </Card>
            {analysis.largest_mod && (
              <Card size="small">
                <Text type="secondary" style={{ fontSize: 12 }}>{t('app.toolbox.largestMod')}</Text>
                <div style={{ fontSize: 18, fontWeight: 600, margin: '4px 0' }}>{analysis.largest_mod.name}</div>
                <Text style={{ color: '#ff4d4f', fontWeight: 600 }}>{analysis.largest_mod.size_formatted}</Text>
              </Card>
            )}
          </div>
          <Table dataSource={analysis.mods} columns={columns} rowKey="unique_id" size="small" pagination={{ pageSize: 20 }} />
        </>
      )}

      {!loading && !analysis && (
        <Empty description={t('app.toolbox.clickToAnalyze')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}
    </div>
  );
}