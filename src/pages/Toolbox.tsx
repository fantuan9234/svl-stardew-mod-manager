import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Tabs, Button, Table, Tag, Space, Spin, Empty, Progress, Card, Typography, message } from 'antd';
import {
  CloudDownloadOutlined,
  ThunderboltOutlined,
  DatabaseOutlined,
  WarningOutlined,
  InfoCircleOutlined,
  DownloadOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';
import {
  scanMods,
  checkAllModsUpdates,
  batchUpdateMods,
  downloadModUpdate,
  checkConflicts,
  analyzeModStorage,
  type ModInfo,
  type ModUpdateStatus,
  type ConflictReport,
  type ModStorageInfo,
  type StorageAnalysisResult,
} from '../utils/tauri-api';

const { Text, Title } = Typography;

function UpdateCheckerTab({ mods }: { mods: ModInfo[] }) {
  const { t } = useTranslation();
  const [updates, setUpdates] = useState<ModUpdateStatus[]>([]);
  const [loading, setLoading] = useState(false);
  const [updating, setUpdating] = useState(false);
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);

  const checkUpdates = useCallback(async () => {
    if (mods.length === 0) return;
    setLoading(true);
    setUpdates([]);
    setSelectedRowKeys([]);
    try {
      const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
      const modsData = mods.map(m => ({
        unique_id: m.unique_id,
        version: m.version,
        name: m.name,
        folder_path: m.folder_path,
        nexus_mod_id: m.nexus_mod_id?.toString() || null,
      }));
      const result = await checkAllModsUpdates(modsData, apiKey);
      setUpdates(result);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.updateCheckFailed'));
    } finally {
      setLoading(false);
    }
  }, [mods, t]);

  const handleBatchUpdate = async () => {
    if (selectedRowKeys.length === 0) return;
    const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!apiKey) {
      message.warning(t('app.toolbox.needApiKey'));
      return;
    }

    const gamePath = localStorage.getItem('svl-game-path') || '';
    if (!gamePath) {
      message.warning(t('app.toolbox.needGamePath'));
      return;
    }

    setUpdating(true);
    try {
      const selectedMods = selectedRowKeys.map(key => {
        const update = updates.find(u => u.unique_id === key);
        return {
          unique_id: key,
          name: update?.name || '',
          nexus_mod_id: update?.nexus_mod_id || null,
          download_url: update?.download_url || null,
        };
      });
      const result = await batchUpdateMods(selectedMods, apiKey, gamePath);
      if (result.updated > 0) {
        message.success(t('app.toolbox.batchUpdateSuccess', { count: result.updated }));
      }
      if (result.failed > 0) {
        message.warning(t('app.toolbox.batchUpdateFailed', { count: result.failed }));
      }
      setSelectedRowKeys([]);
      checkUpdates();
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.updateFailed'));
    } finally {
      setUpdating(false);
    }
  };

  const handleSingleUpdate = async (record: ModUpdateStatus) => {
    const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!apiKey) {
      message.warning(t('app.toolbox.needApiKey'));
      return;
    }
    const gamePath = localStorage.getItem('svl-game-path') || '';
    if (!gamePath) {
      message.warning(t('app.toolbox.needGamePath'));
      return;
    }

    if (!record.nexus_mod_id) {
      if (record.download_url) {
        window.open(record.download_url, '_blank');
      }
      return;
    }

    try {
      await downloadModUpdate(record.nexus_mod_id, apiKey, gamePath, record.unique_id);
      message.success(t('app.toolbox.updateSuccess', { name: record.name }));
      checkUpdates();
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.updateFailed'));
    }
  };

  const columns = [
    {
      title: t('app.toolbox.modName'),
      dataIndex: 'name',
      key: 'name',
      ellipsis: true,
    },
    {
      title: t('app.toolbox.currentVersion'),
      dataIndex: 'current_version',
      key: 'current_version',
      width: 120,
    },
    {
      title: t('app.toolbox.latestVersion'),
      dataIndex: 'latest_version',
      key: 'latest_version',
      width: 120,
      render: (v: string | null) => v ? <Text strong style={{ color: '#52c41a' }}>{v}</Text> : '-',
    },
    {
      title: t('app.toolbox.updateSource'),
      dataIndex: 'update_source',
      key: 'update_source',
      width: 120,
      render: (source: string) => {
        const colorMap: Record<string, string> = {
          SmapiList: 'blue',
          NexusApi: 'green',
          UnofficialUpdate: 'orange',
        };
        const labelMap: Record<string, string> = {
          SmapiList: 'SMAPI',
          NexusApi: 'Nexus',
          UnofficialUpdate: t('app.toolbox.unofficial'),
        };
        return <Tag color={colorMap[source] || 'default'}>{labelMap[source] || source}</Tag>;
      },
    },
    {
      title: t('app.toolbox.action'),
      key: 'action',
      width: 100,
      render: (_: any, record: ModUpdateStatus) => (
        <Button
          size="small"
          type="primary"
          icon={<DownloadOutlined />}
          onClick={() => handleSingleUpdate(record)}
        >
          {t('app.toolbox.update')}
        </Button>
      ),
    },
  ];

  return (
    <div>
      <Space style={{ marginBottom: 16 }}>
        <Button
          type="primary"
          icon={<CloudDownloadOutlined />}
          onClick={checkUpdates}
          loading={loading}
        >
          {t('app.toolbox.checkUpdates')}
        </Button>
        {selectedRowKeys.length > 0 && (
          <Button
            icon={<ThunderboltOutlined />}
            onClick={handleBatchUpdate}
            loading={updating}
          >
            {t('app.toolbox.batchUpdate')} ({selectedRowKeys.length})
          </Button>
        )}
      </Space>

      {loading && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>
            {t('app.toolbox.checkingUpdates')}
          </div>
        </div>
      )}

      {!loading && updates.length > 0 && (
        <Table
          dataSource={updates}
          columns={columns}
          rowKey="unique_id"
          size="small"
          pagination={false}
          rowSelection={{
            selectedRowKeys,
            onChange: (keys) => setSelectedRowKeys(keys),
            getCheckboxProps: (record) => ({
              disabled: !record.nexus_mod_id,
            }),
          }}
        />
      )}

      {!loading && updates.length === 0 && mods.length > 0 && (
        <Empty description={t('app.toolbox.clickToCheck')} />
      )}
    </div>
  );
}

function ConflictDetectorTab({ mods }: { mods: ModInfo[] }) {
  const { t } = useTranslation();
  const [conflicts, setConflicts] = useState<ConflictReport[]>([]);
  const [loading, setLoading] = useState(false);

  const checkAllConflicts = async () => {
    if (mods.length === 0) return;
    setLoading(true);
    try {
      const result = await checkConflicts(mods);
      setConflicts(result);
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
      title: t('app.toolbox.severity'),
      dataIndex: 'severity',
      key: 'severity',
      width: 80,
      render: (s: string) => {
        if (s === 'Error') return <Tag color="error"><ExclamationCircleOutlined /> Error</Tag>;
        if (s === 'Warning') return <Tag color="warning"><WarningOutlined /> Warning</Tag>;
        return <Tag color="default"><InfoCircleOutlined /> Info</Tag>;
      },
    },
    {
      title: t('app.toolbox.conflictType'),
      dataIndex: 'conflict_type',
      key: 'conflict_type',
      width: 150,
      render: (ct: string) => {
        const colorMap: Record<string, string> = {
          MissingDependency: 'red',
          HardcodedPatch: 'red',
          VersionConflict: 'orange',
          AssetConflict: 'orange',
          ContentPackTargetConflict: 'blue',
          ContentPackConflict: 'blue',
          Incompatibility: 'red',
          OptionalDependencyMissing: 'default',
        };
        return <Tag color={colorMap[ct] || 'default'}>{ct}</Tag>;
      },
    },
    {
      title: t('app.toolbox.description'),
      dataIndex: 'description',
      key: 'description',
      ellipsis: true,
    },
    {
      title: t('app.toolbox.solution'),
      dataIndex: 'solution',
      key: 'solution',
      width: 200,
      ellipsis: true,
    },
  ];

  return (
    <div>
      <Space style={{ marginBottom: 16 }}>
        <Button
          type="primary"
          icon={<WarningOutlined />}
          onClick={checkAllConflicts}
          loading={loading}
        >
          {t('app.toolbox.checkConflicts')}
        </Button>
      </Space>

      {loading && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>
            {t('app.toolbox.checkingConflicts')}
          </div>
        </div>
      )}

      {!loading && conflicts.length > 0 && (
        <>
          <Space style={{ marginBottom: 12 }}>
            {errorCount > 0 && <Tag color="error">{t('app.toolbox.errors', { count: errorCount })}</Tag>}
            {warningCount > 0 && <Tag color="warning">{t('app.toolbox.warnings', { count: warningCount })}</Tag>}
            {infoCount > 0 && <Tag color="default">{t('app.toolbox.infos', { count: infoCount })}</Tag>}
          </Space>
          <Table
            dataSource={conflicts}
            columns={columns}
            rowKey={(r) => `${r.unique_id}-${r.conflict_type}-${r.description}`}
            size="small"
            pagination={{ pageSize: 20 }}
          />
        </>
      )}

      {!loading && conflicts.length === 0 && mods.length > 0 && (
        <Empty description={t('app.toolbox.clickToCheckConflicts')} />
      )}
    </div>
  );
}

function StorageAnalyzerTab({ mods }: { mods: ModInfo[] }) {
  const { t } = useTranslation();
  const [analysis, setAnalysis] = useState<StorageAnalysisResult | null>(null);
  const [loading, setLoading] = useState(false);

  const analyze = async () => {
    if (mods.length === 0) return;
    setLoading(true);
    try {
      const result = await analyzeModStorage(mods);
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
          <Text>{name}</Text>
          {!record.enabled && <Tag>{t('app.toolbox.disabled')}</Tag>}
          {record.is_content_pack && <Tag color="blue">CP</Tag>}
        </Space>
      ),
    },
    {
      title: t('app.toolbox.size'),
      dataIndex: 'size_formatted',
      key: 'size_formatted',
      width: 120,
      sorter: (a: ModStorageInfo, b: ModStorageInfo) => a.size_bytes - b.size_bytes,
      defaultSortOrder: 'descend' as const,
    },
    {
      title: t('app.toolbox.fileCount'),
      dataIndex: 'file_count',
      key: 'file_count',
      width: 100,
      sorter: (a: ModStorageInfo, b: ModStorageInfo) => a.file_count - b.file_count,
    },
    {
      title: t('app.toolbox.version'),
      dataIndex: 'version',
      key: 'version',
      width: 100,
    },
  ];

  const enabledPercent = analysis && analysis.total_size_bytes > 0
    ? Math.round((analysis.enabled_size_bytes / analysis.total_size_bytes) * 100)
    : 0;

  return (
    <div>
      <Space style={{ marginBottom: 16 }}>
        <Button
          type="primary"
          icon={<DatabaseOutlined />}
          onClick={analyze}
          loading={loading}
        >
          {t('app.toolbox.analyzeStorage')}
        </Button>
      </Space>

      {loading && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <Spin size="large" />
          <div style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>
            {t('app.toolbox.analyzingStorage')}
          </div>
        </div>
      )}

      {!loading && analysis && (
        <>
          <div style={{ display: 'flex', gap: 16, marginBottom: 16, flexWrap: 'wrap' }}>
            <Card size="small" style={{ minWidth: 160, flex: 1 }}>
              <Text type="secondary">{t('app.toolbox.totalSize')}</Text>
              <div style={{ fontSize: 24, fontWeight: 700, color: 'var(--svl-primary)' }}>
                {analysis.total_size_formatted}
              </div>
              <Text type="secondary">{t('app.toolbox.totalMods', { count: analysis.total_mods })}</Text>
            </Card>
            <Card size="small" style={{ minWidth: 160, flex: 1 }}>
              <Text type="secondary">{t('app.toolbox.enabledSize')}</Text>
              <div style={{ fontSize: 18, fontWeight: 600, color: '#52c41a' }}>
                {analysis.enabled_size_formatted}
              </div>
              <Progress percent={enabledPercent} size="small" strokeColor="#52c41a" />
            </Card>
            <Card size="small" style={{ minWidth: 160, flex: 1 }}>
              <Text type="secondary">{t('app.toolbox.disabledSize')}</Text>
              <div style={{ fontSize: 18, fontWeight: 600, color: 'var(--svl-text-muted)' }}>
                {analysis.disabled_size_formatted}
              </div>
              <Progress percent={100 - enabledPercent} size="small" strokeColor="var(--svl-text-muted)" />
            </Card>
            {analysis.largest_mod && (
              <Card size="small" style={{ minWidth: 160, flex: 1 }}>
                <Text type="secondary">{t('app.toolbox.largestMod')}</Text>
                <div style={{ fontSize: 16, fontWeight: 600 }}>
                  {analysis.largest_mod.name}
                </div>
                <Text style={{ color: '#ff4d4f' }}>{analysis.largest_mod.size_formatted}</Text>
              </Card>
            )}
          </div>

          <Table
            dataSource={analysis.mods}
            columns={columns}
            rowKey="unique_id"
            size="small"
            pagination={{ pageSize: 20 }}
          />
        </>
      )}

      {!loading && !analysis && mods.length > 0 && (
        <Empty description={t('app.toolbox.clickToAnalyze')} />
      )}
    </div>
  );
}

export default function Toolbox() {
  const { t } = useTranslation();
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [modsLoaded, setModsLoaded] = useState(false);

  useEffect(() => {
    scanMods()
      .then(result => {
        setMods(result);
        setModsLoaded(true);
      })
      .catch(() => {
        setModsLoaded(true);
      });
  }, []);

  const tabItems = [
    {
      key: 'updates',
      label: (
        <span>
          <CloudDownloadOutlined style={{ marginRight: 6 }} />
          {t('app.toolbox.updateChecker')}
        </span>
      ),
      children: <UpdateCheckerTab mods={mods} />,
    },
    {
      key: 'conflicts',
      label: (
        <span>
          <WarningOutlined style={{ marginRight: 6 }} />
          {t('app.toolbox.conflictDetector')}
        </span>
      ),
      children: <ConflictDetectorTab mods={mods} />,
    },
    {
      key: 'storage',
      label: (
        <span>
          <DatabaseOutlined style={{ marginRight: 6 }} />
          {t('app.toolbox.storageAnalyzer')}
        </span>
      ),
      children: <StorageAnalyzerTab mods={mods} />,
    },
  ];

  if (!modsLoaded) {
    return (
      <div style={{ padding: '24px', textAlign: 'center', paddingTop: '80px' }}>
        <Spin size="large" />
        <div style={{ marginTop: 12, color: 'var(--svl-text-muted)' }}>
          {t('app.toolbox.loadingMods')}
        </div>
      </div>
    );
  }

  return (
    <div style={{ padding: '24px' }}>
      <Title level={3} style={{ marginBottom: 24 }}>
        🧰 {t('app.toolbox.title')}
      </Title>
      <Tabs items={tabItems} size="large" />
    </div>
  );
}
