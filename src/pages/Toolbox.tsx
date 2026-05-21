import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Table, Tag, Space, Spin, Empty, Progress, Card, Typography, message } from 'antd';
import {
  DatabaseOutlined,
  WarningOutlined,
  InfoCircleOutlined,
  ExclamationCircleOutlined,
  ArrowLeftOutlined,
} from '@ant-design/icons';
import {
  scanMods,
  checkConflicts,
  analyzeModStorage,
  type ConflictReport,
  type ModStorageInfo,
  type StorageAnalysisResult,
} from '../utils/tauri-api';

const { Text, Title } = Typography;

type ToolView = 'home' | 'conflicts' | 'storage';

function ConflictDetectorView({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [conflicts, setConflicts] = useState<ConflictReport[]>([]);
  const [loading, setLoading] = useState(false);
  const [checked, setChecked] = useState(false);

  const doCheck = async () => {
    setLoading(true);
    setConflicts([]);
    try {
      const modsResult = await scanMods();
      const result = await checkConflicts(modsResult);
      setConflicts(result);
      setChecked(true);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.conflictCheckFailed'));
    } finally {
      setLoading(false);
    }
  };

  const errorCount = conflicts.filter(c => c.severity === 'Error').length;
  const warningCount = conflicts.filter(c => c.severity === 'Warning').length;
  const infoCount = conflicts.filter(c => c.severity === 'Info').length;

  const columns = [
    {
      title: t('app.toolbox.severity'), dataIndex: 'severity', key: 'severity', width: 90,
      render: (s: string) => {
        if (s === 'Error') return <Tag color="error"><ExclamationCircleOutlined /> Error</Tag>;
        if (s === 'Warning') return <Tag color="warning"><WarningOutlined /> Warning</Tag>;
        return <Tag><InfoCircleOutlined /> Info</Tag>;
      },
    },
    {
      title: t('app.toolbox.conflictType'), dataIndex: 'conflict_type', key: 'conflict_type', width: 140,
      render: (ct: string) => {
        const m: Record<string, string> = { MissingDependency: 'red', HardcodedPatch: 'red', VersionConflict: 'orange', AssetConflict: 'orange', ContentPackTargetConflict: 'blue', ContentPackConflict: 'blue', Incompatibility: 'red', OptionalDependencyMissing: 'default' };
        return <Tag color={m[ct] || 'default'}>{ct}</Tag>;
      },
    },
    { title: t('app.toolbox.description'), dataIndex: 'description', key: 'description', ellipsis: true },
    { title: t('app.toolbox.solution'), dataIndex: 'solution', key: 'solution', width: 220, ellipsis: true },
  ];

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20 }}>
        <Button icon={<ArrowLeftOutlined />} onClick={onBack} type="text" />
        <Title level={4} style={{ margin: 0 }}><WarningOutlined style={{ marginRight: 8 }} />{t('app.toolbox.conflictDetector')}</Title>
      </div>

      <Space style={{ marginBottom: 16 }}>
        <Button type="primary" icon={<WarningOutlined />} onClick={doCheck} loading={loading}>
          {t('app.toolbox.checkConflicts')}
        </Button>
      </Space>

      {loading && (
        <div style={{ textAlign: 'center', padding: '60px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>{t('app.toolbox.checkingConflicts')}</div>
        </div>
      )}

      {!loading && conflicts.length > 0 && (
        <>
          <Space style={{ marginBottom: 12 }}>
            {errorCount > 0 && <Tag color="error">{t('app.toolbox.errors', { count: errorCount })}</Tag>}
            {warningCount > 0 && <Tag color="warning">{t('app.toolbox.warnings', { count: warningCount })}</Tag>}
            {infoCount > 0 && <Tag>{t('app.toolbox.infos', { count: infoCount })}</Tag>}
          </Space>
          <Table dataSource={conflicts} columns={columns} rowKey={(r) => `${r.unique_id}-${r.conflict_type}-${r.description}`} size="small" pagination={{ pageSize: 20 }} />
        </>
      )}

      {!loading && checked && conflicts.length === 0 && (
        <Empty description={t('app.toolbox.noConflicts')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}

      {!loading && !checked && (
        <Empty description={t('app.toolbox.clickToCheckConflicts')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}
    </div>
  );
}

function StorageAnalyzerView({ onBack }: { onBack: () => void }) {
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
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 20 }}>
        <Button icon={<ArrowLeftOutlined />} onClick={onBack} type="text" />
        <Title level={4} style={{ margin: 0 }}><DatabaseOutlined style={{ marginRight: 8 }} />{t('app.toolbox.storageAnalyzer')}</Title>
      </div>

      <Space style={{ marginBottom: 16 }}>
        <Button type="primary" icon={<DatabaseOutlined />} onClick={doAnalyze} loading={loading}>
          {t('app.toolbox.analyzeStorage')}
        </Button>
      </Space>

      {loading && (
        <div style={{ textAlign: 'center', padding: '60px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>{t('app.toolbox.analyzingStorage')}</div>
        </div>
      )}

      {!loading && analysis && (
        <>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 16, marginBottom: 20 }}>
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

const toolCards = [
  { key: 'conflicts' as ToolView, icon: <WarningOutlined style={{ fontSize: 32, color: '#fa8c16' }} />, color: '#fa8c16' },
  { key: 'storage' as ToolView, icon: <DatabaseOutlined style={{ fontSize: 32, color: '#52c41a' }} />, color: '#52c41a' },
];

export default function Toolbox() {
  const { t } = useTranslation();
  const [view, setView] = useState<ToolView>('home');

  if (view === 'conflicts') return <ConflictDetectorView onBack={() => setView('home')} />;
  if (view === 'storage') return <StorageAnalyzerView onBack={() => setView('home')} />;

  return (
    <div style={{ padding: '24px' }}>
      <Title level={3} style={{ marginBottom: 8 }}>🧰 {t('app.toolbox.title')}</Title>
      <Text type="secondary" style={{ marginBottom: 28, display: 'block' }}>{t('app.toolbox.subtitle')}</Text>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 20 }}>
        {toolCards.map(tool => (
          <Card
            key={tool.key}
            hoverable
            onClick={() => setView(tool.key)}
            style={{
              cursor: 'pointer',
              borderLeft: `4px solid ${tool.color}`,
              transition: 'all 0.2s ease',
            }}
            styles={{ body: { padding: '24px 20px' } }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
              <div style={{
                width: 56, height: 56, borderRadius: 12,
                background: `${tool.color}15`, display: 'flex', alignItems: 'center', justifyContent: 'center',
                flexShrink: 0,
              }}>
                {tool.icon}
              </div>
              <div style={{ flex: 1, minWidth: 0 }}>
                <Title level={5} style={{ margin: '0 0 4px' }}>{t(`app.toolbox.${tool.key}Title`)}</Title>
                <Text type="secondary" style={{ fontSize: 13, lineHeight: 1.5 }}>{t(`app.toolbox.${tool.key}Desc`)}</Text>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}