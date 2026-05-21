import { useState, useCallback, useEffect } from 'react';
import { Typography, Input, Button, List, Tag, Space, Pagination, message, Spin, Progress, Select, Empty, Alert, Card, Row, Col, Divider, theme, Steps, Modal, Collapse, Checkbox } from 'antd';
import { SearchOutlined, DownloadOutlined, HeartOutlined, ThunderboltOutlined, StarOutlined, LinkOutlined, FireOutlined, ReloadOutlined, GlobalOutlined, ArrowLeftOutlined, QuestionCircleOutlined, SettingOutlined, InfoCircleOutlined, UserOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { downloadModFromNexus } from '../utils/tauri-api';

const { Title, Text, Paragraph } = Typography;

interface NexusModSearchResult {
  mod_id: string;
  name: string;
  summary: string;
  version: string;
  author: string;
  picture_url: string | null;
  downloads: number;
  endorsements: number;
  uploaded_time: string;
  nexus_url: string;
  size: number;
}

function formatNum(num: number): string {
    if (num >= 1000000) return `${(num / 1000000).toFixed(1)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(1)}K`;
    return num.toString();
  }

function ModCard({ mod, onDownload, onOpenNexus, downloading, downloadProgress, downloadStatus, t, token, selected, onSelect }: {
  mod: NexusModSearchResult;
  onDownload: (mod: NexusModSearchResult) => void;
  onOpenNexus: (url: string) => void;
  downloading: boolean;
  downloadProgress: number;
  downloadStatus: string;
  t: (key: string, params?: any) => string;
  token: any;
  selected: boolean;
  onSelect: (modId: string) => void;
}) {

  return (
    <Card
      hoverable
      style={{ height: '100%', background: token.colorBgContainer, borderColor: selected ? token.colorPrimary : token.colorBorder, borderWidth: selected ? 2 : 1 }}
      bodyStyle={{ padding: 12, display: 'flex', flexDirection: 'column', height: '100%' }}
    >
      <div style={{ display: 'flex', gap: 12, flex: 1 }}>
        <Checkbox
          checked={selected}
          onChange={() => onSelect(mod.mod_id)}
          style={{ flexShrink: 0, marginTop: 2 }}
        />
        {mod.picture_url ? (
          <img
            src={mod.picture_url}
            alt={mod.name}
            style={{
              width: 64,
              height: 64,
              objectFit: 'cover',
              borderRadius: 6,
              flexShrink: 0,
            }}
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = 'none';
            }}
          />
        ) : (
          <div style={{
            width: 64,
            height: 64,
            borderRadius: 6,
            background: token.colorPrimaryBg,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 24,
            flexShrink: 0,
          }}>
            🎮
          </div>
        )}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ marginBottom: 4 }}>
            <Text strong ellipsis style={{ fontSize: 14, maxWidth: '100%' }}>{mod.name}</Text>
            {mod.version && <Tag color="blue" style={{ marginLeft: 4, fontSize: 11 }}>v{mod.version}</Tag>}
          </div>
          <Paragraph
            ellipsis={{ rows: 2 }}
            style={{ marginBottom: 6, color: token.colorTextSecondary, fontSize: 12, lineHeight: '18px' }}
          >
            {mod.summary || 'No description'}
          </Paragraph>
          <Space size={12} wrap>
            <Text type="secondary" style={{ fontSize: 12 }}>
              <HeartOutlined /> {formatNum(mod.endorsements)}
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              <ThunderboltOutlined /> {formatNum(mod.downloads)}
            </Text>
          </Space>
        </div>
      </div>
      <div style={{ marginTop: 8, display: 'flex', gap: 6 }}>
        {downloading ? (
          <div style={{ flex: 1 }}>
            <Text type="secondary" style={{ fontSize: 11 }}>{downloadStatus || t('features.nexus.downloading')}</Text>
            <Progress percent={downloadProgress} size="small" status="active" />
          </div>
        ) : (
          <>
            <Button
              type="primary"
              size="small"
              icon={<DownloadOutlined />}
              onClick={() => onDownload(mod)}
              style={{ flex: 1 }}
            >
              {t('features.nexus.download')}
            </Button>
            <Button
              size="small"
              icon={<LinkOutlined />}
              onClick={() => onOpenNexus(mod.nexus_url)}
            />
          </>
        )}
      </div>
    </Card>
  );
}

export default function NexusModBrowser() {
  const { t } = useTranslation();
  const { token } = theme.useToken();
  const [searchQuery, setSearchQuery] = useState('');
  const [results, setResults] = useState<NexusModSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(0);
  const [downloadingModId, setDownloadingModId] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [downloadStatus, setDownloadStatus] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [hasSearched, setHasSearched] = useState(false);
  const [apiKey] = useState(() => localStorage.getItem('svl-nexus-api-key') || '');
  const [trendingMods, setTrendingMods] = useState<NexusModSearchResult[]>([]);
  const [trendingLoading, setTrendingLoading] = useState(false);
  const [trendingLoaded, setTrendingLoaded] = useState(false);
  const [showTutorial, setShowTutorial] = useState(false);
  const [neverShowTutorial, setNeverShowTutorial] = useState(
    localStorage.getItem('svl-never-show-nexus-tutorial') === 'true'
  );
  const [selectedModIds, setSelectedModIds] = useState<Set<string>>(new Set());
  const [batchDownloading, setBatchDownloading] = useState(false);

  useEffect(() => {
    if (!neverShowTutorial) {
      setShowTutorial(true);
    }
  }, []);

  useEffect(() => {
    const unlisten = listen('mod-install-progress', (event: any) => {
      const data = event.payload;
      if (data.step === 'download_progress' && data.percent !== undefined) {
        setDownloadProgress(data.percent);
      }
      if (data.step === 'completed' || data.step === 'done') {
        sessionStorage.setItem('svl-mod-installed', 'true');
      }
      if (data.step === 'completed') {
        let msg = data.message || '';
        if (data.scan_found !== undefined) {
          msg += data.scan_found ? ' [scan:OK]' : ' [scan:MISS]';
        }
        if (data.installed_path) {
          msg += '\n路径: ' + data.installed_path;
        }
        setDownloadStatus(msg);
      } else if (data.message) {
        setDownloadStatus(data.message);
      }
    });

    const unlistenCdn = listen('nexus-cdn-link-captured', async (event: any) => {
      const cdnUrl = event.payload?.url;
      if (!cdnUrl) return;

      console.log('[NexusModBrowser] 捕获到 CDN 链接: ' + cdnUrl);
      message.loading({ content: '已捕获下载链接，正在下载安装...', key: 'cdn-download', duration: 0 });

      try {
        const result = await invoke('download_mod_from_cdn_link', { cdnLink: cdnUrl });
        if (result && (result as any).success) {
          message.success({ content: `模组 ${(result as any).mod_name} 安装成功！`, key: 'cdn-download' });
        } else {
          message.error({ content: `安装失败: ${(result as any)?.message || '未知错误'}`, key: 'cdn-download' });
        }
      } catch (err: any) {
        message.error({ content: `下载失败: ${typeof err === 'string' ? err : '未知错误'}`, key: 'cdn-download' });
      }
    });

    return () => {
      unlisten.then(fn => fn());
      unlistenCdn.then(fn => fn());
    };
  }, []);

  const loadTrendingMods = useCallback(async (key: string) => {
    setTrendingLoading(true);
    try {
      const mods = await invoke<NexusModSearchResult[]>('get_trending_nexus_mods', {
        apiKey: key,
      });
      setTrendingMods(mods);
      setTrendingLoaded(true);
    } catch (err) {
      console.error('Failed to load trending mods:', err);
    } finally {
      setTrendingLoading(false);
    }
  }, []);

  const categoryOptions = [
    { value: 'all', label: t('features.nexus.allCategories') },
  ];

  const handleSearch = useCallback(async (query: string, page: number = 1, category: string = 'all') => {
    if (!query.trim()) {
      message.warning(t('features.nexus.enterSearchQuery'));
      return;
    }

    const currentApiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!currentApiKey) {
      message.error(t('features.nexus.noApiKey'));
      return;
    }

    setLoading(true);
    setResults([]);
    setHasSearched(true);
    try {
      const categoryParam = category === 'all' ? null : category;
      const [mods, pages] = await invoke<[NexusModSearchResult[], number]>('search_nexus_mods', {
        apiKey: currentApiKey,
        query,
        page,
        category: categoryParam,
      });
      setResults(mods);
      setTotalPages(pages);
      setCurrentPage(page);
    } catch (err: any) {
      console.error('Search failed:', err);
      message.error(typeof err === 'string' ? err : t('features.nexus.searchFailed'));
    } finally {
      setLoading(false);
    }
  }, [t]);

  const handleDownload = async (mod: NexusModSearchResult) => {
    const currentApiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!currentApiKey) {
      message.error(t('features.nexus.noApiKey'));
      return;
    }

    setDownloadingModId(mod.mod_id);
    setDownloadProgress(0);
    setDownloadStatus(t('features.nexus.downloading'));

    try {
      const result = await downloadModFromNexus(mod.mod_id, currentApiKey, null);
      if (result.success) {
        message.success(t('features.nexus.downloadSuccess', { name: result.mod_name }));
      } else {
        message.error(t('features.nexus.downloadFailed', { name: result.mod_name, error: result.message }));
      }
    } catch (err: any) {
      console.error('Download failed:', err);
      message.error(t('features.nexus.downloadError', { name: mod.name }) + ': ' + (typeof err === 'string' ? err : ''));
    } finally {
      setDownloadingModId(null);
      setDownloadProgress(0);
      setDownloadStatus('');
    }
  };

  const handleOpenNexus = (url: string) => {
    invoke('open_nexus_browser', { initialUrl: url });
  };

  const handleOpenBrowser = () => {
    invoke('open_nexus_browser', { initialUrl: null });
  };

  const toggleModSelect = (modId: string) => {
    setSelectedModIds(prev => {
      const next = new Set(prev);
      if (next.has(modId)) {
        next.delete(modId);
      } else {
        next.add(modId);
      }
      return next;
    });
  };

  const handleBatchDownload = async () => {
    if (selectedModIds.size === 0) return;
    const currentApiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!currentApiKey) {
      message.error(t('features.nexus.noApiKey'));
      return;
    }

    const allMods = [...trendingMods, ...results];
    const toDownload = allMods.filter(m => selectedModIds.has(m.mod_id));

    setBatchDownloading(true);
    let successCount = 0;
    let failCount = 0;

    for (const mod of toDownload) {
      try {
        setDownloadingModId(mod.mod_id);
        setDownloadProgress(0);
        setDownloadStatus(`${t('features.nexus.downloading')} (${successCount + failCount + 1}/${toDownload.length}): ${mod.name}`);
        const result = await downloadModFromNexus(mod.mod_id, currentApiKey, null);
        if (result.success) {
          successCount++;
          setSelectedModIds(prev => {
            const next = new Set(prev);
            next.delete(mod.mod_id);
            return next;
          });
        } else {
          failCount++;
        }
      } catch {
        failCount++;
      }
    }

    setDownloadingModId(null);
    setDownloadProgress(0);
    setDownloadStatus('');
    setBatchDownloading(false);

    if (failCount === 0) {
      message.success(t('features.nexus.batchDownloadSuccess', { count: successCount }));
    } else {
      message.warning(t('features.nexus.batchDownloadPartial', { success: successCount, fail: failCount }));
    }
  };

  const handleBackToHome = () => {
    setHasSearched(false);
    setResults([]);
    setSearchQuery('');
    setSelectedCategory('all');
    setTotalPages(0);
    setCurrentPage(1);
  };

  const handleClearSelection = () => {
    setSelectedModIds(new Set());
  };

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '';
    const mb = bytes / 1024 / 1024;
    return `${mb.toFixed(1)} MB`;
  };

  return (
    <div style={{ height: '100%', overflowY: 'auto', padding: '16px 24px', background: 'var(--svl-bg-primary)' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 24 }}>
        <div>
          <Title level={3} style={{ margin: '0 0 4px 0' }}>
            {t('features.nexus.title')}
          </Title>
          <Text type="secondary">{t('features.nexus.description')}</Text>
        </div>
        <Button
          icon={<QuestionCircleOutlined />}
          onClick={() => setShowTutorial(true)}
        >
          {t('features.nexus.howToUse') || '使用教程'}
        </Button>
      </div>

      {!apiKey && (
        <Alert
          message={t('app.nexusApi.title')}
          description={t('features.nexus.noApiKey')}
          type="warning"
          showIcon
          style={{ marginBottom: 16 }}
        />
      )}

      <div style={{ display: 'flex', gap: 12, marginBottom: 24 }}>
        <Input
          placeholder={t('features.nexus.searchPlaceholder')}
          prefix={<SearchOutlined />}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onPressEnter={() => handleSearch(searchQuery, 1, selectedCategory)}
          style={{ flex: 1, minWidth: 300 }}
          size="large"
          disabled={!apiKey}
        />
        <Select
          value={selectedCategory}
          onChange={(val) => setSelectedCategory(val)}
          style={{ width: 180 }}
          size="large"
          options={categoryOptions}
          disabled={!apiKey}
        />
        <Button
          type="primary"
          size="large"
          icon={<SearchOutlined />}
          onClick={() => handleSearch(searchQuery, 1, selectedCategory)}
          loading={loading}
          disabled={!apiKey}
        >
          {t('features.nexus.search')}
        </Button>
        <Button
          size="large"
          icon={<GlobalOutlined />}
          onClick={handleOpenBrowser}
          style={{ background: '#6c5ce7', borderColor: '#6c5ce7', color: '#fff' }}
        >
          {t('features.nexus.openBrowser') || 'N网浏览器'}
        </Button>
        {selectedModIds.size > 0 && (
          <>
            <Button
              size="large"
              onClick={handleClearSelection}
              style={{ borderColor: '#ff4d4f', color: '#ff4d4f' }}
            >
              {t('features.nexus.clearSelection')}
            </Button>
            <Button
              type="primary"
              size="large"
              icon={<DownloadOutlined />}
              onClick={handleBatchDownload}
              loading={batchDownloading}
              style={{ background: '#52c41a', borderColor: '#52c41a' }}
            >
              {t('features.nexus.batchDownload')} ({selectedModIds.size})
            </Button>
          </>
        )}
      </div>

      {!hasSearched && apiKey && (
        <div style={{ marginBottom: 32 }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 16 }}>
              <Space>
                <FireOutlined style={{ color: '#ff4d4f', fontSize: 20 }} />
                <Title level={4} style={{ margin: 0 }}>{t('features.nexus.trending')}</Title>
              </Space>
              {!trendingLoaded && (
                <Button
                  size="small"
                  icon={<FireOutlined />}
                  onClick={() => loadTrendingMods(apiKey)}
                  loading={trendingLoading}
                >
                  {t('features.nexus.loadTrending')}
                </Button>
              )}
              {trendingLoaded && (
                <Button
                  size="small"
                  icon={<ReloadOutlined />}
                  onClick={() => loadTrendingMods(apiKey)}
                  loading={trendingLoading}
                >
                  {t('features.nexus.refresh')}
                </Button>
              )}
            </div>

            {trendingLoading && (
              <div style={{ textAlign: 'center', padding: 40 }}>
                <Spin size="large" />
                <div style={{ marginTop: 12 }}>
                  <Text type="secondary">{t('features.nexus.loadingTrending')}</Text>
                </div>
              </div>
            )}

            {!trendingLoading && trendingMods.length > 0 && (
              <>
              <Row gutter={[12, 12]}>
                {trendingMods.map((mod) => (
                  <Col key={mod.mod_id} xs={24} sm={12} md={8} lg={6}>
                    <ModCard
                      mod={mod}
                      onDownload={handleDownload}
                      onOpenNexus={handleOpenNexus}
                      downloading={downloadingModId === mod.mod_id}
                      downloadProgress={downloadProgress}
                      downloadStatus={downloadStatus}
                      t={t}
                      token={token}
                      selected={selectedModIds.has(mod.mod_id)}
                      onSelect={toggleModSelect}
                    />
                  </Col>
                ))}
              </Row>
              </>
            )}

            {!trendingLoading && !trendingLoaded && (
              <div style={{ textAlign: 'center', padding: 40 }}>
                <Text type="secondary" style={{ fontSize: 14 }}>
                  {t('features.nexus.trendingHint')}
                </Text>
              </div>
            )}
          </div>
        </div>
      )}

      {loading && (
        <div style={{ textAlign: 'center', padding: 60 }}>
          <Spin size="large" />
          <div style={{ marginTop: 16 }}>
            <Text type="secondary">{t('features.nexus.searching')}</Text>
          </div>
        </div>
      )}

      {!loading && results.length > 0 && (
        <>
          <div style={{ marginBottom: 16, display: 'flex', alignItems: 'center', gap: 8 }}>
            <Button
              icon={<ArrowLeftOutlined />}
              onClick={handleBackToHome}
              size="small"
            >
              {t('features.nexus.backToHome')}
            </Button>
            {selectedCategory !== 'all' && !searchQuery.trim() && (
              <>
                <Tag color="purple" style={{ fontSize: 14, padding: '4px 12px' }}>
                  {categoryOptions.find(c => c.value === selectedCategory)?.label || selectedCategory}
                </Tag>
                <Text type="secondary">{results.length} 个模组</Text>
              </>
            )}
          </div>
          <List
            dataSource={results}
            renderItem={(mod) => (
              <List.Item
                style={{
                  marginBottom: 16,
                  padding: 16,
                  background: token.colorBgContainer,
                  borderRadius: 8,
                  border: `1px solid ${selectedModIds.has(mod.mod_id) ? token.colorPrimary : token.colorBorder}`,
                  borderWidth: selectedModIds.has(mod.mod_id) ? 2 : 1,
                }}
              >
                <Checkbox
                  checked={selectedModIds.has(mod.mod_id)}
                  onChange={() => toggleModSelect(mod.mod_id)}
                  style={{ marginRight: 8, alignSelf: 'flex-start', marginTop: 4 }}
                />
                <List.Item.Meta
                  avatar={
                    mod.picture_url ? (
                      <img
                        src={mod.picture_url}
                        alt={mod.name}
                        style={{
                          width: 80,
                          height: 80,
                          objectFit: 'cover',
                          borderRadius: 8,
                        }}
                        onError={(e) => {
                          (e.target as HTMLImageElement).style.display = 'none';
                        }}
                      />
                    ) : (
                      <div style={{
                        width: 80,
                        height: 80,
                        borderRadius: 8,
                        background: token.colorPrimaryBg,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        fontSize: 32,
                      }}>
                        🎮
                      </div>
                    )
                  }
                  title={
                    <Space>
                      <Text strong style={{ fontSize: 16 }}>{mod.name}</Text>
                      {mod.version && <Tag color="blue">v{mod.version}</Tag>}
                    </Space>
                  }
                  description={
                    <div>
                      <Paragraph
                        ellipsis={{ rows: 2, expandable: false }}
                        style={{ marginBottom: 8, color: token.colorTextSecondary }}
                      >
                        {mod.summary || 'No description available.'}
                      </Paragraph>
                      <Space size={16} wrap>
                        <Text type="secondary">
                          <HeartOutlined /> {formatNum(mod.endorsements)}
                        </Text>
                        <Text type="secondary">
                          <ThunderboltOutlined /> {formatNum(mod.downloads)}
                        </Text>
                        <Text type="secondary">
                          <StarOutlined /> {mod.author}
                        </Text>
                        {mod.size > 0 && <Text type="secondary">{formatSize(mod.size)}</Text>}
                      </Space>
                    </div>
                  }
                />
                <Space direction="vertical" size="small" align="end">
                  {downloadingModId === mod.mod_id ? (
                    <div style={{ width: 160 }}>
                      <Text type="secondary" style={{ fontSize: 12 }}>{downloadStatus || t('features.nexus.downloading')}</Text>
                      <Progress percent={downloadProgress} size="small" status="active" />
                    </div>
                  ) : (
                    <>
                      <Button
                        type="primary"
                        icon={<DownloadOutlined />}
                        onClick={() => handleDownload(mod)}
                        loading={downloadingModId !== null}
                      >
                        {t('features.nexus.download')}
                      </Button>
                      <Button
                        icon={<LinkOutlined />}
                        size="small"
                        onClick={() => handleOpenNexus(mod.nexus_url)}
                      >
                        {t('features.nexus.viewOnNexus')}
                      </Button>
                    </>
                  )}
                </Space>
              </List.Item>
            )}
          />

          {totalPages > 1 && (
            <div style={{ textAlign: 'center', marginTop: 24, marginBottom: 24 }}>
              <Pagination
                current={currentPage}
                total={totalPages * 20}
                pageSize={20}
                onChange={(page) => {
                  handleSearch(searchQuery, page, selectedCategory);
                }}
                showSizeChanger={false}
              />
            </div>
          )}
        </>
      )}

      {!loading && hasSearched && results.length === 0 && (
        <div style={{ textAlign: 'center', padding: 60 }}>
          <Empty description={t('features.nexus.noResults')} />
          <Button
            icon={<ArrowLeftOutlined />}
            onClick={handleBackToHome}
            style={{ marginTop: 16 }}
          >
            {t('features.nexus.backToHome')}
          </Button>
        </div>
      )}

      {!loading && !hasSearched && !apiKey && (
        <div style={{ textAlign: 'center', padding: 80 }}>
          <SearchOutlined style={{ fontSize: 48, color: token.colorTextSecondary, marginBottom: 16 }} />
          <div>
            <Text type="secondary" style={{ fontSize: 16 }}>
              {t('features.nexus.searchPlaceholder')}
            </Text>
          </div>
        </div>
      )}

      <Modal
        open={showTutorial}
        onCancel={() => setShowTutorial(false)}
        footer={[
          <Checkbox
            key="never"
            checked={neverShowTutorial}
            onChange={(e) => {
              setNeverShowTutorial(e.target.checked);
              localStorage.setItem('svl-never-show-nexus-tutorial', e.target.checked ? 'true' : 'false');
            }}
          >
            {t('features.nexus.neverShowAgain')}
          </Checkbox>,
          <Button key="close" type="primary" onClick={() => setShowTutorial(false)}>
            {t('features.nexus.gotIt')}
          </Button>,
        ]}
        width={720}
        title={
          <Space>
            <SettingOutlined style={{ fontSize: 20, color: token.colorPrimary }} />
            <Title level={4} style={{ margin: 0 }}>{t('features.nexus.tutorialTitle')}</Title>
          </Space>
        }
        styles={{
          body: { maxHeight: '70vh', overflowY: 'auto' },
        }}
      >
        <div style={{ marginBottom: 24 }}>
          <Text type="secondary">{t('features.nexus.tutorialSubtitle')}</Text>
        </div>

        <Steps
          direction="vertical"
          current={-1}
          items={[
            {
              title: t('features.nexus.tutorial.step1Title'),
              description: (
                <div>
                  <Text>{t('features.nexus.tutorial.step1Desc')}</Text>
                  <div style={{ marginTop: 8, padding: 12, background: token.colorBgLayout, borderRadius: 8 }}>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      在 SVL 主页或模组列表中点击下载按钮，会自动弹出 N 网浏览器窗口
                    </Text>
                  </div>
                </div>
              ),
              icon: <GlobalOutlined style={{ color: '#1890ff' }} />,
            },
            {
              title: t('features.nexus.tutorial.step2Title'),
              description: (
                <div>
                  <Text>{t('features.nexus.tutorial.step2Desc')}</Text>
                  <div style={{ marginTop: 8, padding: 12, background: token.colorBgLayout, borderRadius: 8 }}>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      在弹出的浏览器窗口右上角点击登录，输入你的 N 网账号和密码
                    </Text>
                    <br />
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      如果没有账号，点击注册免费创建一个
                    </Text>
                  </div>
                </div>
              ),
              icon: <UserOutlined style={{ color: '#52c41a' }} />,
            },
            {
              title: t('features.nexus.tutorial.step3Title'),
              description: (
                <div>
                  <Text>{t('features.nexus.tutorial.step3Desc')}</Text>
                  <div style={{ marginTop: 8, padding: 12, background: token.colorBgLayout, borderRadius: 8 }}>
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      • SVL 会自动尝试完成下载和 Slow Download 流程
                    </Text>
                    <br />
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      • 如果最后一步没有自动点击，请手动点击 <Text strong>「Slow Download」</Text> 按钮
                    </Text>
                    <br />
                    <Text type="secondary" style={{ fontSize: 12 }}>
                      • 下载完成后自动解压安装到 Mods 文件夹
                    </Text>
                  </div>
                </div>
              ),
              icon: <DownloadOutlined style={{ color: '#722ed1' }} />,
            },
          ]}
        />

        <Divider style={{ borderColor: token.colorBorder }} />

        <Card
          size="small"
          title={
            <Space>
              <InfoCircleOutlined style={{ color: token.colorPrimary }} />
              <Text strong>{t('features.nexus.tipsTitle')}</Text>
            </Space>
          }
          style={{ background: token.colorBgLayout, borderColor: token.colorBorder }}
          bodyStyle={{ padding: 12 }}
        >
          <Collapse
            defaultActiveKey={['faq1']}
            ghost
            items={[
              {
                key: 'faq1',
                label: <Text strong style={{ fontSize: 13 }}>{t('features.nexus.faq.q1Title')}</Text>,
                children: (
                  <Text type="secondary" style={{ fontSize: 12 }}>{t('features.nexus.faq.q1Answer')}</Text>
                ),
              },
              {
                key: 'faq2',
                label: <Text strong style={{ fontSize: 13 }}>{t('features.nexus.faq.q2Title')}</Text>,
                children: (
                  <Text type="secondary" style={{ fontSize: 12 }}>{t('features.nexus.faq.q2Answer')}</Text>
                ),
              },
              {
                key: 'faq3',
                label: <Text strong style={{ fontSize: 13 }}>{t('features.nexus.faq.q3Title')}</Text>,
                children: (
                  <Text type="secondary" style={{ fontSize: 12 }}>{t('features.nexus.faq.q3Answer')}</Text>
                ),
              },
            ]}
          />
        </Card>
      </Modal>
    </div>
  );
}
