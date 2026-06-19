import { useState, useCallback, useEffect } from 'react';
import { Typography, Input, Button, Tag, Space, Pagination, message, Spin, Progress, Select, Empty, Alert, Card, Row, Col, Divider, theme, Steps, Modal, Collapse, Checkbox, Segmented, Tooltip } from 'antd';
import { SearchOutlined, DownloadOutlined, HeartOutlined, ThunderboltOutlined, FireOutlined, ReloadOutlined, GlobalOutlined, ArrowLeftOutlined, QuestionCircleOutlined, SettingOutlined, InfoCircleOutlined, UserOutlined, ClockCircleOutlined, CrownOutlined, SortAscendingOutlined } from '@ant-design/icons';
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

function formatDate(dateStr: string, t: (key: string, params?: any) => string): string {
  if (!dateStr) return '';
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    if (diffDays === 0) return t('app.nexus.dateToday');
    if (diffDays === 1) return t('app.nexus.dateYesterday');
    if (diffDays < 7) return t('app.nexus.dateDaysAgo', { days: diffDays });
    if (diffDays < 30) return t('app.nexus.dateWeeksAgo', { weeks: Math.floor(diffDays / 7) });
    if (diffDays < 365) return t('app.nexus.dateMonthsAgo', { months: Math.floor(diffDays / 30) });
    return t('app.nexus.dateYearsAgo', { years: Math.floor(diffDays / 365) });
  } catch {
    return dateStr;
  }
}

function ModGridCard({ mod, onDownload, onOpenNexus, downloading, downloadProgress, downloadStatus, t, token, selected, onSelect }: {
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
      style={{
        height: '100%',
        background: token.colorBgContainer,
        borderColor: selected ? token.colorPrimary : token.colorBorder,
        borderWidth: selected ? 2 : 1,
        borderRadius: 10,
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}
      bodyStyle={{ padding: 0, flex: 1, display: 'flex', flexDirection: 'column' }}
      className="mod-grid-card"
    >
      <div
        style={{
          position: 'relative',
          width: '100%',
          paddingTop: '56.25%',
          background: `linear-gradient(135deg, ${token.colorPrimaryBg}, ${token.colorBgLayout})`,
          overflow: 'hidden',
          cursor: 'pointer',
        }}
        onClick={() => onOpenNexus(mod.nexus_url)}
      >
        {mod.picture_url ? (
          <img
            src={mod.picture_url}
            alt={mod.name}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: '100%',
              objectFit: 'cover',
            }}
            onError={(e) => {
              (e.target as HTMLImageElement).style.display = 'none';
            }}
          />
        ) : (
          <div style={{
            position: 'absolute',
            top: 0,
            left: 0,
            width: '100%',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 40,
          }}>
            🎮
          </div>
        )}
        <div style={{
          position: 'absolute',
          top: 8,
          left: 8,
          display: 'flex',
          gap: 4,
          alignItems: 'center',
        }}>
          <Checkbox
            checked={selected}
            onChange={() => onSelect(mod.mod_id)}
            style={{ background: 'rgba(0,0,0,0.4)', borderRadius: 4, padding: '0 4px' }}
          />
          <Tag color="default" style={{ margin: 0, fontSize: 10, lineHeight: '18px', padding: '0 6px', borderRadius: 4, background: 'rgba(0,0,0,0.5)', color: '#fff', border: 'none' }}>
            #{mod.mod_id}
          </Tag>
          {mod.version && (
            <Tag color="blue" style={{ margin: 0, fontSize: 10, lineHeight: '18px', padding: '0 6px', borderRadius: 4 }}>
              v{mod.version}
            </Tag>
          )}
        </div>
        <div style={{
          position: 'absolute',
          bottom: 0,
          left: 0,
          right: 0,
          padding: '24px 10px 8px',
          background: 'linear-gradient(transparent, rgba(0,0,0,0.7))',
        }}>
          <Text style={{ color: '#fff', fontSize: 14, fontWeight: 600, display: 'block', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', textShadow: '0 1px 3px rgba(0,0,0,0.5)' }}>
            {mod.name}
          </Text>
        </div>
      </div>

      <div style={{ padding: '10px 12px', flex: 1, display: 'flex', flexDirection: 'column' }}>
        <div style={{ marginBottom: 6, display: 'flex', alignItems: 'center', gap: 6 }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            <UserOutlined /> {mod.author}
          </Text>
          {mod.uploaded_time && (
            <Text type="secondary" style={{ fontSize: 11, marginLeft: 'auto' }}>
              {formatDate(mod.uploaded_time, t)}
            </Text>
          )}
        </div>

        <Paragraph
          ellipsis={{ rows: 2 }}
          style={{ marginBottom: 8, color: token.colorTextSecondary, fontSize: 12, lineHeight: '18px', flex: 1 }}
        >
          {mod.summary || 'No description'}
        </Paragraph>

        <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
          <Tooltip title={t('features.nexus.endorsements')}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              <HeartOutlined style={{ color: '#ff4d4f' }} /> {formatNum(mod.endorsements)}
            </Text>
          </Tooltip>
          <Tooltip title={t('features.nexus.downloads')}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              <ThunderboltOutlined style={{ color: '#1890ff' }} /> {formatNum(mod.downloads)}
            </Text>
          </Tooltip>
        </div>

        {downloading ? (
          <div>
            <Text type="secondary" style={{ fontSize: 11 }}>{downloadStatus || t('features.nexus.downloading')}</Text>
            <Progress percent={downloadProgress} size="small" status="active" />
          </div>
        ) : (
          <div style={{ display: 'flex', gap: 6 }}>
            <Button
              type="primary"
              size="small"
              icon={<DownloadOutlined />}
              onClick={() => onDownload(mod)}
              style={{ flex: 1, borderRadius: 6 }}
            >
              {t('features.nexus.download')}
            </Button>
          </div>
        )}
      </div>
    </Card>
  );
}

type BrowseTab = 'trending' | 'recent' | 'monthly';
type SortBy = 'endorsements' | 'downloads' | 'updated';

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
  const [apiKey, setApiKey] = useState(() => localStorage.getItem('svl-nexus-api-key') || '');

  const [browseTab, setBrowseTab] = useState<BrowseTab>('trending');
  const [sortBy, setSortBy] = useState<SortBy>('endorsements');

  const [trendingMods, setTrendingMods] = useState<NexusModSearchResult[]>([]);
  const [trendingLoading, setTrendingLoading] = useState(false);
  const [trendingLoaded, setTrendingLoaded] = useState(false);

  const [recentlyUpdatedMods, setRecentlyUpdatedMods] = useState<NexusModSearchResult[]>([]);
  const [recentlyUpdatedLoading, setRecentlyUpdatedLoading] = useState(false);
  const [recentlyUpdatedLoaded, setRecentlyUpdatedLoaded] = useState(false);

  const [monthlyTopMods, setMonthlyTopMods] = useState<NexusModSearchResult[]>([]);
  const [monthlyTopLoading, setMonthlyTopLoading] = useState(false);
  const [monthlyTopLoaded, setMonthlyTopLoaded] = useState(false);

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
    const handleApiKeyChange = () => {
      setApiKey(localStorage.getItem('svl-nexus-api-key') || '');
    };
    window.addEventListener('nexus-api-key-changed', handleApiKeyChange);
    return () => window.removeEventListener('nexus-api-key-changed', handleApiKeyChange);
  }, []);

  useEffect(() => {
    if (apiKey) {
      loadTabData(browseTab, apiKey);
    }
  }, [apiKey]);

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

      message.loading({ content: t('app.nexus.cdnLinkCaptured'), key: 'cdn-download', duration: 0 });

      try {
        const result = await invoke('download_mod_from_cdn_link', { cdnLink: cdnUrl });
        if (result && (result as any).success) {
          message.success({ content: t('app.nexus.installSuccess', { name: (result as any).mod_name }), key: 'cdn-download' });
        } else {
          message.error({ content: t('app.nexus.installFailed', { error: (result as any)?.message || t('app.nexus.unknownError') }), key: 'cdn-download' });
        }
      } catch (err: any) {
        message.error({ content: t('app.nexus.downloadFailed', { error: typeof err === 'string' ? err : t('app.nexus.unknownError') }), key: 'cdn-download' });
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
      const mods = await invoke<NexusModSearchResult[]>('get_trending_nexus_mods', { apiKey: key });
      setTrendingMods(mods);
      setTrendingLoaded(true);
    } catch (err) {
    } finally {
      setTrendingLoading(false);
    }
  }, []);

  const loadRecentlyUpdatedMods = useCallback(async (key: string) => {
    setRecentlyUpdatedLoading(true);
    try {
      const mods = await invoke<NexusModSearchResult[]>('get_recently_updated_nexus_mods', { apiKey: key });
      setRecentlyUpdatedMods(mods);
      setRecentlyUpdatedLoaded(true);
    } catch (err) {
    } finally {
      setRecentlyUpdatedLoading(false);
    }
  }, []);

  const loadMonthlyTopMods = useCallback(async (key: string) => {
    setMonthlyTopLoading(true);
    try {
      const mods = await invoke<NexusModSearchResult[]>('get_monthly_top_nexus_mods', { apiKey: key });
      setMonthlyTopMods(mods);
      setMonthlyTopLoaded(true);
    } catch (err) {
    } finally {
      setMonthlyTopLoading(false);
    }
  }, []);

  const loadTabData = useCallback((tab: BrowseTab, key: string) => {
    switch (tab) {
      case 'trending':
        if (!trendingLoaded) loadTrendingMods(key);
        break;
      case 'recent':
        if (!recentlyUpdatedLoaded) loadRecentlyUpdatedMods(key);
        break;
      case 'monthly':
        if (!monthlyTopLoaded) loadMonthlyTopMods(key);
        break;
    }
  }, [trendingLoaded, recentlyUpdatedLoaded, monthlyTopLoaded, loadTrendingMods, loadRecentlyUpdatedMods, loadMonthlyTopMods]);

  const handleTabChange = (tab: string) => {
    setBrowseTab(tab as BrowseTab);
    if (apiKey) {
      loadTabData(tab as BrowseTab, apiKey);
    }
  };

  const handleRefreshTab = () => {
    if (!apiKey) return;
    switch (browseTab) {
      case 'trending':
        setTrendingLoaded(false);
        loadTrendingMods(apiKey);
        break;
      case 'recent':
        setRecentlyUpdatedLoaded(false);
        loadRecentlyUpdatedMods(apiKey);
        break;
      case 'monthly':
        setMonthlyTopLoaded(false);
        loadMonthlyTopMods(apiKey);
        break;
    }
  };

  const getCurrentMods = (): NexusModSearchResult[] => {
    let mods: NexusModSearchResult[] = [];
    switch (browseTab) {
      case 'trending': mods = trendingMods; break;
      case 'recent': mods = recentlyUpdatedMods; break;
      case 'monthly': mods = monthlyTopMods; break;
    }

    const sorted = [...mods];
    switch (sortBy) {
      case 'endorsements':
        sorted.sort((a, b) => b.endorsements - a.endorsements);
        break;
      case 'downloads':
        sorted.sort((a, b) => b.downloads - a.downloads);
        break;
      case 'updated':
        sorted.sort((a, b) => b.uploaded_time.localeCompare(a.uploaded_time));
        break;
    }
    return sorted;
  };

  const getCurrentLoading = (): boolean => {
    switch (browseTab) {
      case 'trending': return trendingLoading;
      case 'recent': return recentlyUpdatedLoading;
      case 'monthly': return monthlyTopLoading;
    }
  };

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
    doDownload(mod, currentApiKey);
  };

  const doDownload = async (mod: NexusModSearchResult, apiKey: string) => {
    setDownloadingModId(mod.mod_id);
    setDownloadProgress(0);
    setDownloadStatus(t('features.nexus.downloading'));

    try {
      const result = await downloadModFromNexus(mod.mod_id, apiKey, null);
      if (result.success) {
        message.success(t('features.nexus.downloadSuccess', { name: result.mod_name }));
      } else {
        message.error(t('features.nexus.downloadFailed', { name: result.mod_name, error: result.message }));
      }
    } catch (err: any) {
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

    const allMods = [...trendingMods, ...recentlyUpdatedMods, ...monthlyTopMods, ...results];
    const toDownload = allMods.filter(m => selectedModIds.has(m.mod_id));

    setBatchDownloading(true);
    let successCount = 0;
    let failCount = 0;

    for (const mod of toDownload) {
      try {
        setDownloadingModId(mod.mod_id);
        setDownloadProgress(0);
        setDownloadStatus(`${t('features.nexus.downloading')} (${successCount + failCount + 1}/${toDownload.length}): ${mod.name}`);
        const result = await downloadModFromNexus(mod.mod_id, currentApiKey, null, undefined);
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

  const browseTabs = [
    {
      value: 'trending',
      label: (
        <Space>
          <FireOutlined />
          {t('features.nexus.trending')}
        </Space>
      ),
    },
    {
      value: 'recent',
      label: (
        <Space>
          <ClockCircleOutlined />
          {t('features.nexus.recentlyUpdated')}
        </Space>
      ),
    },
    {
      value: 'monthly',
      label: (
        <Space>
          <CrownOutlined />
          {t('features.nexus.monthlyTop')}
        </Space>
      ),
    },
  ];

  const sortOptions = [
    { value: 'endorsements', label: t('features.nexus.sortByEndorsements') },
    { value: 'downloads', label: t('features.nexus.sortByDownloads') },
    { value: 'updated', label: t('features.nexus.sortByUpdated') },
  ];

  return (
    <div style={{ height: '100%', overflowY: 'auto', padding: '16px 24px', background: 'var(--svl-bg-primary)' }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 20 }}>
        <div>
          <Title level={3} style={{ margin: '0 0 4px 0' }}>
            {t('features.nexus.title')}
          </Title>
          <Text type="secondary">{t('features.nexus.description')}</Text>
        </div>
        <Space>
          {selectedModIds.size > 0 && (
            <>
              <Button
                onClick={handleClearSelection}
                style={{ borderColor: '#ff4d4f', color: '#ff4d4f' }}
              >
                {t('features.nexus.clearSelection')}
              </Button>
              <Button
                type="primary"
                icon={<DownloadOutlined />}
                onClick={handleBatchDownload}
                loading={batchDownloading}
                style={{ background: '#52c41a', borderColor: '#52c41a' }}
              >
                {t('features.nexus.batchDownload')} ({selectedModIds.size})
              </Button>
            </>
          )}
          <Button
            icon={<QuestionCircleOutlined />}
            onClick={() => setShowTutorial(true)}
          >
            {t('features.nexus.howToUse')}
          </Button>
        </Space>
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

      <div style={{
        display: 'flex',
        gap: 10,
        marginBottom: 20,
        padding: '12px 16px',
        background: token.colorBgContainer,
        borderRadius: 10,
        border: `1px solid ${token.colorBorder}`,
      }}>
        <Input
          placeholder={t('features.nexus.searchPlaceholder')}
          prefix={<SearchOutlined style={{ color: token.colorTextQuaternary }} />}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onPressEnter={() => handleSearch(searchQuery, 1, selectedCategory)}
          style={{ flex: 1, minWidth: 280 }}
          size="middle"
          disabled={!apiKey}
          allowClear
        />
        <Select
          value={selectedCategory}
          onChange={(val) => setSelectedCategory(val)}
          style={{ width: 160 }}
          size="middle"
          options={categoryOptions}
          disabled={!apiKey}
        />
        <Button
          type="primary"
          icon={<SearchOutlined />}
          onClick={() => handleSearch(searchQuery, 1, selectedCategory)}
          loading={loading}
          disabled={!apiKey}
        >
          {t('features.nexus.search')}
        </Button>
        <Button
          icon={<GlobalOutlined />}
          onClick={handleOpenBrowser}
          style={{ background: '#6c5ce7', borderColor: '#6c5ce7', color: '#fff' }}
        >
          {t('features.nexus.openBrowser') || 'N网浏览器'}
        </Button>
      </div>

      {hasSearched ? (
        <>
          <div style={{ marginBottom: 16, display: 'flex', alignItems: 'center', gap: 8 }}>
            <Button
              icon={<ArrowLeftOutlined />}
              onClick={handleBackToHome}
              size="small"
            >
              {t('features.nexus.backToHome')}
            </Button>
            <Text type="secondary" style={{ fontSize: 13 }}>
              {t('features.nexus.searchResultFor', { query: searchQuery })} · {results.length} {t('features.nexus.modsCount')}
            </Text>
          </div>

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
              <Row gutter={[16, 16]}>
                {results.map((mod) => (
                  <Col key={mod.mod_id} xs={24} sm={12} md={8} lg={6} xl={4}>
                    <ModGridCard
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

          {!loading && results.length === 0 && (
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
        </>
      ) : (
        <>
          {apiKey && (
            <>
              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                marginBottom: 16,
                padding: '0 4px',
              }}>
                <Segmented
                  value={browseTab}
                  onChange={handleTabChange}
                  options={browseTabs}
                  style={{ background: token.colorBgLayout }}
                />
                <Space>
                  <Select
                    value={sortBy}
                    onChange={setSortBy}
                    options={sortOptions}
                    size="small"
                    style={{ width: 140 }}
                    suffixIcon={<SortAscendingOutlined />}
                  />
                  <Button
                    size="small"
                    icon={<ReloadOutlined />}
                    onClick={handleRefreshTab}
                    loading={getCurrentLoading()}
                  >
                    {t('features.nexus.refresh')}
                  </Button>
                </Space>
              </div>

              {getCurrentLoading() && !getCurrentMods().length && (
                <div style={{ textAlign: 'center', padding: 80 }}>
                  <Spin size="large" />
                  <div style={{ marginTop: 16 }}>
                    <Text type="secondary">{t('features.nexus.loadingMods')}</Text>
                  </div>
                </div>
              )}

              {!getCurrentLoading() && getCurrentMods().length > 0 && (
                <Row gutter={[16, 16]}>
                  {getCurrentMods().map((mod) => (
                    <Col key={mod.mod_id} xs={24} sm={12} md={8} lg={6} xl={4}>
                      <ModGridCard
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
              )}

              {!getCurrentLoading() && getCurrentMods().length === 0 && (
                <div style={{ textAlign: 'center', padding: 60 }}>
                  <Empty description={t('features.nexus.noModsInCategory')} />
                </div>
              )}
            </>
          )}

          {!apiKey && (
            <div style={{ textAlign: 'center', padding: 80 }}>
              <SearchOutlined style={{ fontSize: 48, color: token.colorTextSecondary, marginBottom: 16 }} />
              <div>
                <Text type="secondary" style={{ fontSize: 16 }}>
                  {t('features.nexus.searchPlaceholder')}
                </Text>
              </div>
            </div>
          )}
        </>
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
              icon: <DownloadOutlined style={{ color: 'var(--svl-primary)' }} />,
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
