import { useState, useEffect, useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import {
  Button, Input, Select, Spin, Progress, Typography, message, Collapse, Tag, Empty, Tooltip, Table, Checkbox,
  Modal, Segmented
} from 'antd';
import {
  ArrowLeftOutlined, TranslationOutlined, ApiOutlined, ScanOutlined,
  PlayCircleOutlined, ExperimentOutlined, LinkOutlined,
  WarningFilled, CloseCircleFilled, CheckCircleFilled, UndoOutlined,
  HistoryOutlined, ReloadOutlined, FileSearchOutlined, SearchOutlined
} from '@ant-design/icons';
import { listen } from '@tauri-apps/api/event';
import {
  scanTranslatableMods, translateModFile, testAiConnection, restoreTranslationBackup,
  scanTranslationBackups, getModUntranslatedEntries,
  type ModTranslationDetail, type UntranslatedEntry,
  detectGamePath, checkSmapiStatus,
  type AiConfig, type ModTranslationStatus, type BackupEntry
} from '../utils/tauri-api';
import { openUrl } from '../utils/openUrl';

const { Text, Title } = Typography;

const AI_PRESETS = [
  {
    name: 'DeepSeek',
    baseUrl: 'https://api.deepseek.com/v1',
    model: 'deepseek-v4-flash',
    models: [
      { value: 'deepseek-v4-flash', label: 'DeepSeek V4 Flash (快速)' },
      { value: 'deepseek-v4-pro', label: 'DeepSeek V4 Pro (旗舰)' },
      { value: 'deepseek-chat', label: 'DeepSeek V3 (旧版)' },
      { value: 'deepseek-reasoner', label: 'DeepSeek R1 (推理·旧版)' },
    ],
    rechargeUrl: 'https://platform.deepseek.com/api_keys',
  },
  {
    name: 'app.translator.providerQwen',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode/v1',
    model: 'qwen3.6-plus',
    models: [
      { value: 'qwen3.6-max-preview', label: 'Qwen3.6 Max Preview (旗舰)' },
      { value: 'qwen3.6-plus', label: 'Qwen3.6 Plus (推荐)' },
      { value: 'qwen3.6-flash', label: 'Qwen3.6 Flash (快速)' },
      { value: 'qwen3-plus', label: 'Qwen3 Plus' },
      { value: 'qwen3-turbo', label: 'Qwen3 Turbo' },
      { value: 'qwen-plus', label: 'Qwen Plus (旧版)' },
      { value: 'qwen-turbo', label: 'Qwen Turbo (旧版)' },
      { value: 'qwen-max', label: 'Qwen Max (旧版)' },
    ],
    rechargeUrl: 'https://dashscope.console.aliyun.com/apiKey',
  },
  {
    name: 'app.translator.providerGlm',
    baseUrl: 'https://open.bigmodel.cn/api/paas/v4',
    model: 'glm-4-0724',
    models: [
      { value: 'glm-5.1', label: 'GLM-5.1 (旗舰)' },
      { value: 'glm-4.5', label: 'GLM-4.5' },
      { value: 'glm-4-plus', label: 'GLM-4 Plus' },
      { value: 'glm-4-air', label: 'GLM-4 Air' },
      { value: 'glm-4-airx', label: 'GLM-4 AirX' },
      { value: 'glm-4-flash', label: 'GLM-4 Flash' },
      { value: 'glm-4-0724', label: 'GLM-4.7 Flash' },
      { value: 'glm-4-flashx', label: 'GLM-4 FlashX' },
      { value: 'glm-z1-air', label: 'GLM-Z1 Air (推理)' },
      { value: 'glm-z1-airx', label: 'GLM-Z1 AirX (推理)' },
      { value: 'glm-z1-flash', label: 'GLM-Z1 Flash (推理)' },
      { value: 'glm-4-long', label: 'GLM-4 Long (长文本)' },
    ],
    rechargeUrl: 'https://open.bigmodel.cn/usercenter/apikeys',
  },
  {
    name: 'app.translator.providerMoonshot',
    baseUrl: 'https://api.moonshot.cn/v1',
    model: 'kimi-latest',
    models: [
      { value: 'kimi-latest', label: 'Kimi Latest (自动升级)' },
      { value: 'kimi-k2.6', label: 'Kimi K2.6 (旗舰)' },
      { value: 'kimi-k2.5', label: 'Kimi K2.5' },
      { value: 'moonshot-v1-128k', label: 'Moonshot V1 128K' },
      { value: 'moonshot-v1-32k', label: 'Moonshot V1 32K' },
      { value: 'moonshot-v1-8k', label: 'Moonshot V1 8K' },
    ],
    rechargeUrl: 'https://platform.moonshot.cn/console/api-keys',
  },
  {
    name: 'app.translator.providerCustom',
    baseUrl: '',
    model: '',
    models: [],
    rechargeUrl: '',
  },
];

const TARGET_LANGS = [
  { value: '简体中文', label: 'app.translator.langSimplifiedChinese' },
  { value: '繁體中文', label: 'app.translator.langTraditionalChinese' },
  { value: '日本語', label: 'app.translator.langJapanese' },
  { value: '한국어', label: 'app.translator.langKorean' },
];

const TRANSLATE_MODES = [
  { value: 'missing', label: 'app.translator.modeMissing' },
  { value: 'all', label: 'app.translator.modeAll' },
];

type TranslateMode = 'missing' | 'all';

interface FileResult {
  filePath: string;
  modName: string;
  relativePath: string;
  success: boolean;
  message: string;
  backupPath: string | null;
}

interface TranslationSample {
  key: string;
  source: string;
  translation: string;
}

export default function ModTranslatorView({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [provider, setProvider] = useState(() => localStorage.getItem('svl-ai-provider') || 'DeepSeek');
  const [apiKey, setApiKey] = useState(() => localStorage.getItem('svl-ai-api-key') || '');
  const [model, setModel] = useState(() => localStorage.getItem('svl-ai-model') || 'deepseek-v4-flash');
  const [baseUrl, setBaseUrl] = useState(() => localStorage.getItem('svl-ai-base-url') || 'https://api.deepseek.com/v1');
  const [targetLang, setTargetLang] = useState(() => localStorage.getItem('svl-ai-target-lang') || '简体中文');
  const [translateMode, setTranslateMode] = useState<TranslateMode>('missing');
  const [testing, setTesting] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [mods, setMods] = useState<ModTranslationStatus[]>([]);
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [translating, setTranslating] = useState(false);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [results, setResults] = useState<FileResult[]>([]);
  const [currentFile, setCurrentFile] = useState('');
  const [detailProgress, setDetailProgress] = useState<{
    phase: string;
    chunkCurrent: number;
    chunkTotal: number;
    entryCurrent: number;
    entryTotal: number;
    currentKeys: string[];
    firstKey: string;
  } | null>(null);
  const [backups, setBackups] = useState<BackupEntry[]>([]);
  const [backupsLoading, setBackupsLoading] = useState(false);
  const [liveSamples, setLiveSamples] = useState<{
    translated: TranslationSample[];
    missing: TranslationSample[];
  }>({ translated: [], missing: [] });
  const [samplesModName, setSamplesModName] = useState<string>('');
  const [detailModal, setDetailModal] = useState<{
    open: boolean;
    loading: boolean;
    modName: string;
    modPath: string;
    detail: ModTranslationDetail | null;
    filter: 'all' | 'untranslated' | 'same_as_source';
    searchKey: string;
  }>({
    open: false,
    loading: false,
    modName: '',
    modPath: '',
    detail: null,
    filter: 'all',
    searchKey: '',
  });

  useEffect(() => {
    localStorage.setItem('svl-ai-provider', provider);
    localStorage.setItem('svl-ai-api-key', apiKey);
    localStorage.setItem('svl-ai-model', model);
    localStorage.setItem('svl-ai-base-url', baseUrl);
    localStorage.setItem('svl-ai-target-lang', targetLang);
  }, [provider, apiKey, model, baseUrl, targetLang]);

  useEffect(() => {
    const unlisten = listen('translate-progress', (event: any) => {
      const data = event.payload;
      if (data) {
        setDetailProgress({
          phase: data.phase || '',
          chunkCurrent: data.chunk_current || 0,
          chunkTotal: data.chunk_total || 0,
          entryCurrent: data.entry_current || 0,
          entryTotal: data.entry_total || 0,
          currentKeys: data.current_keys || [],
          firstKey: data.first_key || '',
        });
      }
    });

    const unlistenSample = listen('translate-sample', (event: any) => {
      const data = event.payload;
      if (data) {
        setLiveSamples({
          translated: data.translated || [],
          missing: data.missing || [],
        });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenSample.then((fn) => fn());
    };
  }, []);

  const handleProviderChange = useCallback((val: string) => {
    setProvider(val);
    const preset = AI_PRESETS.find(p => p.name === val);
    if (preset && preset.baseUrl) {
      setBaseUrl(preset.baseUrl);
      setModel(preset.model);
    }
  }, []);

  const getAiConfig = useCallback((): AiConfig => ({
    base_url: baseUrl,
    api_key: apiKey,
    model,
  }), [baseUrl, apiKey, model]);

  const getGamePath = useCallback(async (): Promise<string | null> => {
    try {
      const smapiInfo = await checkSmapiStatus();
      if (smapiInfo.installed && smapiInfo.game_path) {
        return smapiInfo.game_path;
      }
      const pathInfo = await detectGamePath();
      if (pathInfo.detected_path) {
        return pathInfo.detected_path;
      }
    } catch {}
    return null;
  }, []);

  const handleTest = async () => {
    if (!apiKey) { message.warning(t('app.translator.noAiConfig')); return; }
    setTesting(true);
    try {
      const reply = await testAiConnection(getAiConfig());
      message.success(t('app.translator.testSuccess') + ': ' + reply);
    } catch (e: any) {
      message.error(t('app.translator.testFailed') + ': ' + (e?.toString() || ''));
    } finally {
      setTesting(false);
    }
  };

  const handleScan = async () => {
    const gamePath = await getGamePath();
    if (!gamePath) { message.warning(t('app.translator.noGamePath')); return; }
    setScanning(true);
    setMods([]);
    setSelectedMods(new Set());
    setResults([]);
    try {
      const result = await scanTranslatableMods(gamePath, targetLang);
      setMods(result);
      const needProcess = result.filter(m => m.status === 'untranslated' || m.status === 'partial');
      if (result.length === 0) {
        message.info(t('app.translator.noFilesFound'));
      } else {
        message.success(t('app.translator.scanComplete', { count: result.length, need: needProcess.length }));
      }
    } catch (e: any) {
      message.error(e?.toString() || t('app.translator.scanFailed'));
    } finally {
      setScanning(false);
    }
  };

  const needProcessMods = useMemo(() =>
    mods.filter(m => m.status === 'untranslated' || m.status === 'partial'),
  [mods]);

  const noNeedMods = useMemo(() =>
    mods.filter(m => m.status === 'completed' || m.status === 'no_i18n'),
  [mods]);

  const toggleMod = (name: string) => {
    setSelectedMods(prev => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name); else next.add(name);
      return next;
    });
  };

  const openModDetail = async (record: ModTranslationStatus) => {
    if (!record.has_i18n) {
      message.info(t('app.translator.detailNoI18n'));
      return;
    }
    setDetailModal({
      open: true,
      loading: true,
      modName: record.mod_name,
      modPath: record.mod_path,
      detail: null,
      filter: 'all',
      searchKey: '',
    });
    try {
      const detail = await getModUntranslatedEntries(record.mod_path, targetLang);
      setDetailModal(prev => ({ ...prev, loading: false, detail }));
    } catch (e: any) {
      message.error(typeof e === 'string' ? e : (e?.message || String(e)));
      setDetailModal(prev => ({ ...prev, open: false, loading: false }));
    }
  };

  const toggleAllNeed = () => {
    const names = needProcessMods.map(m => m.mod_name);
    const allSelected = names.every(n => selectedMods.has(n));
    setSelectedMods(prev => {
      const next = new Set(prev);
      if (allSelected) {
        names.forEach(n => next.delete(n));
      } else {
        names.forEach(n => next.add(n));
      }
      return next;
    });
  };

  const handleTranslate = async () => {
    if (!apiKey) { message.warning(t('app.translator.noAiConfig')); return; }
    if (selectedMods.size === 0) { message.warning(t('app.translator.noModsSelected')); return; }

    const selectedModList = mods.filter(m => selectedMods.has(m.mod_name));
    const aiConfig = getAiConfig();

    let allFiles: { modName: string; filePath: string; fileType: string; relativePath: string }[] = [];

    for (const mod of selectedModList) {
      if (!mod.has_i18n) continue;

      if (translateMode === 'missing' && mod.status === 'partial' && mod.default_file) {
        allFiles.push({
          modName: mod.mod_name,
          filePath: mod.default_file,
          fileType: 'i18n',
          relativePath: `${mod.mod_name} (补充缺失)`,
        });
      } else {
        const sourceFile = mod.default_file || mod.target_file;
        if (sourceFile) {
          allFiles.push({
            modName: mod.mod_name,
            filePath: sourceFile,
            fileType: mod.file_type || 'i18n',
            relativePath: `${mod.mod_name} (${mod.file_type})`,
          });
        }
      }
    }

    const total = allFiles.length;
    if (total === 0) { message.warning(t('app.translator.noFilesToTranslate')); return; }

    setTranslating(true);
    setProgress({ current: 0, total });
    setResults([]);
    setDetailProgress(null);
    setLiveSamples({ translated: [], missing: [] });
    setSamplesModName('');

    const fileResults: FileResult[] = [];

    for (let i = 0; i < allFiles.length; i++) {
      const file = allFiles[i];
      setCurrentFile(file.modName);
      setProgress({ current: i, total });
      setLiveSamples({ translated: [], missing: [] });
      setSamplesModName(file.modName);
      try {
        const result = await translateModFile(file.filePath, file.fileType, aiConfig, targetLang);
        fileResults.push({
          filePath: file.filePath,
          modName: file.modName,
          relativePath: file.relativePath,
          success: result.success,
          message: result.message,
          backupPath: result.backup_path,
        });
      } catch (e: any) {
        fileResults.push({
          filePath: file.filePath,
          modName: file.modName,
          relativePath: file.relativePath,
          success: false,
          message: e?.toString() || 'Unknown error',
          backupPath: null,
        });
      }
      setResults([...fileResults]);
    }

    setProgress({ current: total, total });
    setTranslating(false);
    setCurrentFile('');
    setDetailProgress(null);

    const successCount = fileResults.filter(r => r.success).length;
    const failCount = fileResults.filter(r => !r.success).length;
    if (failCount === 0) {
      message.success(t('app.translator.allSuccess', { count: successCount }));
    } else {
      message.warning(t('app.translator.partialSuccess', { success: successCount, failed: failCount }));
    }

    await handleScan();
  };

  const handleRestore = async (filePath: string) => {
    try {
      await restoreTranslationBackup(filePath);
      message.success(t('app.translator.restored'));
      setResults(prev => prev.map(r => r.filePath === filePath ? { ...r, success: false, message: 'Restored' } : r));
      loadBackups();
    } catch (e: any) {
      message.error(t('app.translator.restoreFailed') + ': ' + (e?.toString() || ''));
    }
  };

  const loadBackups = useCallback(async () => {
    try {
      const pathInfo = await detectGamePath();
      const gp = pathInfo?.detected_path;
      if (!gp) return;
      setBackupsLoading(true);
      const modsDir = gp + '/Mods';
      const list = await scanTranslationBackups(modsDir);
      setBackups(list);
    } catch {
      setBackups([]);
    } finally {
      setBackupsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadBackups();
  }, [loadBackups]);

  const handleRestoreBackup = async (originalPath: string) => {
    try {
      await restoreTranslationBackup(originalPath);
      message.success(t('app.translator.restored'));
      loadBackups();
    } catch (e: any) {
      message.error(t('app.translator.restoreFailed') + ': ' + (e?.toString() || ''));
    }
  };

  const handleRecharge = async (url: string) => {
    try {
      await openUrl(url);
    } catch {
      message.error(t('app.translator.openUrlFailed'));
    }
  };

  const currentPreset = AI_PRESETS.find(p => p.name === provider);
  const successCount = results.filter(r => r.success).length;
  const failCount = results.filter(r => !r.success).length;

  const statusRender = (status: string) => {
    switch (status) {
      case 'completed':
        return <Tag icon={<CheckCircleFilled />} color="success">{t('app.translator.statusCompleted')}</Tag>;
      case 'partial':
        return <Tag icon={<WarningFilled />} color="warning">{t('app.translator.statusPartial')}</Tag>;
      case 'untranslated':
        return <Tag icon={<CloseCircleFilled />} color="error">{t('app.translator.statusUntranslated')}</Tag>;
      default:
        return <Tag color="default">{t('app.translator.statusNoI18n')}</Tag>;
    }
  };

  const progressRender = (record: ModTranslationStatus) => {
    if (record.status === 'no_i18n') return <Text type="secondary" style={{ fontSize: 12 }}>—</Text>;
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <Text strong style={{ fontSize: 13 }}>
          {record.translated_entries} / {record.total_entries}
        </Text>
        {record.remaining_entries > 0 && (
          <Text type="secondary" style={{ fontSize: 12 }}>
            {t('app.translator.remaining', { count: record.remaining_entries })}
          </Text>
        )}
      </div>
    );
  };

  const columns = [
    {
      title: t('app.translator.colSelect'),
      key: 'select',
      width: 60,
      render: (_: any, record: ModTranslationStatus) => (
        <Checkbox
          checked={selectedMods.has(record.mod_name)}
          onChange={() => toggleMod(record.mod_name)}
          disabled={(record.status === 'no_i18n' && record.file_type === 'none') || record.status === 'completed' || translating}
        />
      ),
    },
    {
      title: t('app.translator.colModName'),
      dataIndex: 'mod_name',
      key: 'mod_name',
      ellipsis: true,
    },
    {
      title: t('app.translator.colStatus'),
      key: 'status',
      width: 120,
      render: (_: any, record: ModTranslationStatus) => statusRender(record.status),
    },
    {
      title: t('app.translator.colProgress'),
      key: 'progress',
      width: 180,
      render: (_: any, record: ModTranslationStatus) => progressRender(record),
    },
  ];

  return (
    <div style={{ padding: '20px 24px', maxWidth: 900, margin: '0 auto' }}>
      <div style={{ display: 'flex', alignItems: 'center', marginBottom: 24 }}>
        <Button icon={<ArrowLeftOutlined />} onClick={onBack} style={{ marginRight: 12 }} />
        <TranslationOutlined style={{ fontSize: 22, color: '#722ed1', marginRight: 10 }} />
        <Title level={4} style={{ margin: 0 }}>{t('app.translator.title')}</Title>
      </div>

      <Collapse
        defaultActiveKey={['ai']}
        items={[{
          key: 'ai',
          label: <span><ApiOutlined style={{ marginRight: 6 }} />{t('app.translator.aiConfig')}</span>,
          children: (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
              <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
                <div style={{ flex: '0 0 160px' }}>
                  <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.translator.provider')}</Text>
                  <Select
                    value={provider}
                    onChange={handleProviderChange}
                    style={{ width: '100%' }}
                    options={AI_PRESETS.map(p => ({ value: p.name, label: t(p.name) }))}
                  />
                </div>
                <div style={{ flex: '1 1 300px', minWidth: 200 }}>
                  <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.translator.apiKey')}</Text>
                  <Input.Password
                    value={apiKey}
                    onChange={e => setApiKey(e.target.value)}
                    placeholder={t('app.translator.apiKeyPlaceholder')}
                  />
                </div>
                {currentPreset?.rechargeUrl && (
                  <div style={{ flex: '0 0 auto', alignSelf: 'flex-end' }}>
                    <Tooltip title={t('app.translator.rechargeTooltip')}>
                      <Button
                        icon={<LinkOutlined />}
                        onClick={() => handleRecharge(currentPreset.rechargeUrl!)}
                        size="small"
                      >
                        {t('app.translator.recharge')}
                      </Button>
                    </Tooltip>
                  </div>
                )}
              </div>
              <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
                <div style={{ flex: '1 1 200px', minWidth: 150 }}>
                  <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.translator.model')}</Text>
                  {currentPreset && currentPreset.models.length > 0 ? (
                    <Select
                      value={model}
                      onChange={setModel}
                      style={{ width: '100%' }}
                      options={currentPreset.models}
                    />
                  ) : (
                    <Input value={model} onChange={e => setModel(e.target.value)} placeholder="model-name" />
                  )}
                </div>
                <div style={{ flex: '2 1 300px', minWidth: 200 }}>
                  <Text type="secondary" style={{ fontSize: 12, display: 'block', marginBottom: 4 }}>{t('app.translator.baseUrl')}</Text>
                  <Input value={baseUrl} onChange={e => setBaseUrl(e.target.value)} />
                </div>
              </div>
              <div>
                <Button
                  icon={<ExperimentOutlined />}
                  onClick={handleTest}
                  loading={testing}
                  disabled={!apiKey}
                  size="small"
                >
                  {t('app.translator.testConnection')}
                </Button>
              </div>
            </div>
          ),
        }]}
        style={{ marginBottom: 20 }}
      />

      <div style={{
        background: 'rgba(114, 46, 209, 0.06)', borderRadius: 8, padding: '16px 20px', marginBottom: 20,
        border: '1px solid rgba(114, 46, 209, 0.15)'
      }}>
        <Text strong style={{ display: 'block', marginBottom: 12, fontSize: 14, color: '#531dab' }}>
          {t('app.translator.translateSettings')}
        </Text>
        <div style={{ display: 'flex', gap: 24, flexWrap: 'wrap', alignItems: 'center' }}>
          <div>
            <Text style={{ fontSize: 12, display: 'block', marginBottom: 4, color: '#666' }}>{t('app.translator.targetLang')}</Text>
            <Select value={targetLang} onChange={setTargetLang} style={{ width: 140 }} options={TARGET_LANGS.map(l => ({ value: l.value, label: t(l.label) }))} />
          </div>
          <div>
            <Text style={{ fontSize: 12, display: 'block', marginBottom: 4, color: '#666' }}>{t('app.translator.translateMode')}</Text>
            <Select
              value={translateMode}
              onChange={setTranslateMode}
              style={{ width: 160 }}
              options={TRANSLATE_MODES.map(m => ({ value: m.value, label: t(m.label) }))}
            />
          </div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: 12, marginBottom: 16, flexWrap: 'wrap', alignItems: 'center' }}>
        <Button
          icon={<ScanOutlined />}
          onClick={handleScan}
          loading={scanning}
        >
          {t('app.translator.scanMods')}
        </Button>
        {mods.length > 0 && (
          <>
            <Button size="small" onClick={toggleAllNeed}>
              {t('app.translator.selectAllNeed')}
            </Button>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {t('app.translator.selectedCount', { count: selectedMods.size, total: needProcessMods.length })}
            </Text>
          </>
        )}
        {selectedMods.size > 0 && (
          <Button
            type="primary"
            icon={<PlayCircleOutlined />}
            onClick={handleTranslate}
            loading={translating}
            disabled={!apiKey || translating}
            style={{ background: '#722ed1', borderColor: '#722ed1' }}
          >
            {translating ? t('app.translator.translating') : t('app.translator.startTranslate')}
          </Button>
        )}
      </div>

      {scanning && <Spin style={{ display: 'block', margin: '20px auto' }} />}

      {mods.length > 0 && !scanning && (
        <>
          <div style={{ marginBottom: 8 }}>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {t('app.translator.scanSummary', { total: mods.length, need: needProcessMods.length })}
            </Text>
          </div>

          {needProcessMods.length > 0 && (
            <div style={{ marginBottom: 16 }}>
              <div style={{
                background: 'rgba(250, 173, 20, 0.08)', borderRadius: '8px 8px 0 0',
                padding: '10px 16px', border: '1px solid rgba(250, 173, 20, 0.2)', borderBottom: 'none',
                display: 'flex', alignItems: 'center', gap: 8
              }}>
                <Text strong style={{ color: '#d48806' }}>{t('app.translator.needProcess')}</Text>
                <Tag color="warning">{needProcessMods.length}</Tag>
              </div>
              <Table
                dataSource={needProcessMods}
                columns={columns}
                rowKey="mod_name"
                size="small"
                pagination={false}
                style={{
                  border: '1px solid rgba(250, 173, 20, 0.2)',
                  borderRadius: '0 0 8px 8px',
                }}
                onRow={(record: ModTranslationStatus) => ({
                  onClick: (e: any) => {
                    const tag = (e?.target as HTMLElement)?.tagName;
                    if (tag === 'INPUT' || tag === 'BUTTON' || tag === 'LABEL' || tag === 'SPAN') return;
                    openModDetail(record);
                  },
                  style: { cursor: record.has_i18n ? 'pointer' : 'default' },
                })}
                locale={{ emptyText: '' }}
              />
            </div>
          )}

          {noNeedMods.length > 0 && (
            <div>
              <div style={{
                background: 'rgba(82, 196, 26, 0.06)', borderRadius: '8px 8px 0 0',
                padding: '10px 16px', border: '1px solid rgba(82, 196, 26, 0.2)', borderBottom: 'none',
                display: 'flex', alignItems: 'center', gap: 8
              }}>
                <Text strong style={{ color: '#389e0d' }}>{t('app.translator.noNeed')}</Text>
                <Tag color="success">{noNeedMods.length}</Tag>
              </div>
              <Table
                dataSource={noNeedMods}
                columns={columns}
                rowKey="mod_name"
                size="small"
                pagination={false}
                style={{
                  border: '1px solid rgba(82, 196, 26, 0.2)',
                  borderRadius: '0 0 8px 8px',
                }}
                onRow={(record: ModTranslationStatus) => ({
                  onClick: (e: any) => {
                    const tag = (e?.target as HTMLElement)?.tagName;
                    if (tag === 'INPUT' || tag === 'BUTTON' || tag === 'LABEL' || tag === 'SPAN') return;
                    openModDetail(record);
                  },
                  style: { cursor: record.has_i18n ? 'pointer' : 'default' },
                })}
                locale={{ emptyText: '' }}
              />
            </div>
          )}
        </>
      )}

      {mods.length === 0 && !scanning && (
        <Empty description={t('app.translator.noFilesFound')} style={{ marginBottom: 20 }} />
      )}

      {translating && (
        <div style={{
          marginBottom: 20,
          background: 'rgba(114, 46, 209, 0.06)',
          borderRadius: 10,
          padding: '16px 20px',
          border: '1px solid rgba(114, 46, 209, 0.15)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
            <Text strong style={{ fontSize: 14, color: '#531dab' }}>
              {t('app.translator.translating')}
            </Text>
            <Text type="secondary" style={{ fontSize: 12 }}>
              {progress.current}/{progress.total} {t('app.translator.files')}
            </Text>
          </div>
          <Progress
            percent={progress.total > 0 ? Math.round((progress.current / progress.total) * 100) : 0}
            status="active"
            strokeColor="#722ed1"
            showInfo={false}
            size="small"
            style={{ marginBottom: 8 }}
          />
          {currentFile && (
            <Text type="secondary" style={{ fontSize: 13, display: 'block', marginBottom: 6 }}>
              {t('app.translator.currentFile')}: <Text strong>{currentFile}</Text>
            </Text>
          )}
          {detailProgress && (
            <div style={{
              background: 'rgba(114, 46, 209, 0.04)',
              borderRadius: 8,
              padding: '10px 14px',
              border: '1px solid rgba(114, 46, 209, 0.1)',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
                <Text style={{ fontSize: 12, color: '#722ed1' }}>
                  {detailProgress.phase === 'i18n' ? 'i18n' : 'Content'} {t('app.translator.chunkProgress', { current: detailProgress.chunkCurrent, total: detailProgress.chunkTotal })}
                </Text>
                <Text type="secondary" style={{ fontSize: 12 }}>
                  {detailProgress.entryCurrent}/{detailProgress.entryTotal}
                </Text>
              </div>
              <Progress
                percent={detailProgress.entryTotal > 0 ? Math.round((detailProgress.entryCurrent / detailProgress.entryTotal) * 100) : 0}
                status="active"
                strokeColor="#b37feb"
                showInfo={false}
                size="small"
                style={{ marginBottom: 6 }}
              />
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
                <Text type="secondary" style={{ fontSize: 11 }}>{t('app.translator.currentKey')}:</Text>
                {detailProgress.currentKeys.length > 0 ? (
                  detailProgress.currentKeys.map((key, idx) => (
                    <Tag key={idx} style={{
                      fontSize: 11,
                      borderRadius: 4,
                      background: 'rgba(114, 46, 209, 0.1)',
                      color: '#722ed1',
                      border: 'none',
                      margin: 0,
                      maxWidth: 180,
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}>
                      {key}
                    </Tag>
                  ))
                ) : (
                  <Tag style={{
                    fontSize: 11,
                    borderRadius: 4,
                    background: 'rgba(114, 46, 209, 0.1)',
                    color: '#722ed1',
                    border: 'none',
                    margin: 0,
                  }}>
                    {detailProgress.firstKey}
                  </Tag>
                )}
                {detailProgress.entryTotal > detailProgress.currentKeys.length && detailProgress.currentKeys.length > 0 && (
                  <Text type="secondary" style={{ fontSize: 11 }}>
                    +{detailProgress.entryTotal - detailProgress.currentKeys.length} ...
                  </Text>
                )}
              </div>
            </div>
          )}

          {(liveSamples.translated.length > 0 || liveSamples.missing.length > 0) && (
            <div style={{
              marginTop: 10,
              background: 'rgba(114, 46, 209, 0.04)',
              borderRadius: 8,
              padding: '10px 14px',
              border: '1px solid rgba(114, 46, 209, 0.1)',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
                <Text style={{ fontSize: 12, color: '#722ed1', fontWeight: 600 }}>
                  {t('app.translator.livePreview')}
                </Text>
                {samplesModName && (
                  <Tag color="purple" style={{ fontSize: 11, margin: 0 }}>{samplesModName}</Tag>
                )}
                {liveSamples.translated.length > 0 && (
                  <Tag color="success" style={{ fontSize: 11, margin: 0 }}>
                    {t('app.translator.livePreviewTranslated', { count: liveSamples.translated.length })}
                  </Tag>
                )}
                {liveSamples.missing.length > 0 && (
                  <Tag color="warning" style={{ fontSize: 11, margin: 0 }}>
                    {t('app.translator.livePreviewSkipped', { count: liveSamples.missing.length })}
                  </Tag>
                )}
              </div>
              <div style={{ maxHeight: 280, overflowY: 'auto', borderRadius: 6, background: 'var(--svl-bg-elevated, rgba(0,0,0,0.2))' }}>
                <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: 12 }}>
                  <thead>
                    <tr style={{ position: 'sticky', top: 0, background: 'var(--svl-bg-elevated, rgba(0,0,0,0.3))', zIndex: 1 }}>
                      <th style={{ textAlign: 'left', padding: '6px 10px', fontWeight: 600, color: '#b37feb', borderBottom: '1px solid rgba(114,46,209,0.2)', width: '30%' }}>
                        {t('app.translator.livePreviewKey')}
                      </th>
                      <th style={{ textAlign: 'left', padding: '6px 10px', fontWeight: 600, color: '#b37feb', borderBottom: '1px solid rgba(114,46,209,0.2)', width: '35%' }}>
                        {t('app.translator.livePreviewSource')}
                      </th>
                      <th style={{ textAlign: 'left', padding: '6px 10px', fontWeight: 600, color: '#b37feb', borderBottom: '1px solid rgba(114,46,209,0.2)', width: '35%' }}>
                        {t('app.translator.livePreviewTranslation')}
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {liveSamples.translated.map((s, idx) => (
                      <tr key={`t-${idx}`} style={{ borderBottom: '1px solid rgba(114,46,209,0.06)' }}>
                        <td style={{ padding: '6px 10px', color: '#b37feb', fontFamily: 'monospace', fontSize: 11, wordBreak: 'break-all' }}>
                          {s.key}
                        </td>
                        <td style={{ padding: '6px 10px', color: 'var(--svl-text-secondary, #aaa)' }}>
                          {s.source}
                        </td>
                        <td style={{ padding: '6px 10px', color: 'var(--svl-text-primary, #fff)' }}>
                          {s.translation}
                        </td>
                      </tr>
                    ))}
                    {liveSamples.missing.map((s, idx) => (
                      <tr key={`m-${idx}`} style={{ borderBottom: '1px solid rgba(114,46,209,0.06)', opacity: 0.6 }}>
                        <td style={{ padding: '6px 10px', color: '#b37feb', fontFamily: 'monospace', fontSize: 11, wordBreak: 'break-all' }}>
                          {s.key}
                        </td>
                        <td style={{ padding: '6px 10px', color: 'var(--svl-text-secondary, #aaa)' }}>
                          {s.source}
                        </td>
                        <td style={{ padding: '6px 10px', color: 'var(--svl-text-tertiary, #888)', fontStyle: 'italic' }}>
                          {t('app.translator.livePreviewNotTranslated')}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}

      {results.length > 0 && (
        <div style={{
          background: 'rgba(114, 46, 209, 0.04)', borderRadius: 8, border: '1px solid rgba(114, 46, 209, 0.12)',
          maxHeight: 300, overflowY: 'auto', marginTop: 16
        }}>
          <div style={{ padding: '8px 16px', borderBottom: '1px solid rgba(114, 46, 209, 0.08)', display: 'flex', gap: 12 }}>
            <Text strong style={{ fontSize: 13 }}>
              {t('app.translator.results')}
            </Text>
            <Tag color="success">{successCount} {t('app.translator.success')}</Tag>
            {failCount > 0 && <Tag color="error">{failCount} {t('app.translator.failed')}</Tag>}
          </div>
          {results.map((r, i) => (
            <div
              key={i}
              style={{
                padding: '8px 16px', borderBottom: '1px solid rgba(114, 46, 209, 0.08)',
                display: 'flex', alignItems: 'center', gap: 8
              }}
            >
              {r.success ? (
                <CheckCircleFilled style={{ color: '#52c41a' }} />
              ) : (
                <CloseCircleFilled style={{ color: '#ff4d4f' }} />
              )}
              <Text style={{ flex: 1, fontSize: 13 }}>{r.modName} / {r.relativePath}</Text>
              {r.success && r.backupPath && (
                <Tooltip title={t('app.translator.restoreBackup')}>
                  <Button
                    type="link"
                    size="small"
                    icon={<UndoOutlined />}
                    onClick={() => handleRestore(r.filePath)}
                  >
                    {t('app.translator.restoreBackup')}
                  </Button>
                </Tooltip>
              )}
              {!r.success && r.message !== 'Restored' && (
                <Tooltip title={r.message}>
                  <Text type="danger" style={{ fontSize: 11, maxWidth: 200 }} ellipsis>
                    {r.message}
                  </Text>
                </Tooltip>
              )}
              {r.message === 'Restored' && (
                <Tag color="default" style={{ fontSize: 11 }}>{t('app.translator.restored')}</Tag>
              )}
            </div>
          ))}
        </div>
      )}

      <Modal
        open={detailModal.open}
        onCancel={() => setDetailModal(prev => ({ ...prev, open: false }))}
        width={920}
        title={
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <FileSearchOutlined style={{ color: '#722ed1' }} />
            <span>{detailModal.modName}</span>
            {detailModal.detail && (
              <Tag color="purple" style={{ marginLeft: 4 }}>
                {t('app.translator.detailUntranslatedCount', {
                  count: detailModal.detail.untranslated_count,
                  total: detailModal.detail.total_entries,
                })}
              </Tag>
            )}
          </div>
        }
        footer={
          <Button onClick={() => setDetailModal(prev => ({ ...prev, open: false }))}>
            {t('common.close')}
          </Button>
        }
        styles={{ body: { maxHeight: '70vh', overflowY: 'auto' } }}
      >
        {detailModal.loading && (
          <div style={{ textAlign: 'center', padding: '40px 0' }}>
            <Spin tip={t('app.translator.detailLoading')} />
          </div>
        )}

        {!detailModal.loading && detailModal.detail && (
          <div>
            <div style={{
              display: 'flex', gap: 8, marginBottom: 12, flexWrap: 'wrap', alignItems: 'center'
            }}>
              <Segmented
                value={detailModal.filter}
                onChange={(v: any) => setDetailModal(prev => ({ ...prev, filter: v }))}
                options={[
                  {
                    label: t('app.translator.detailFilterAll', { count: detailModal.detail!.entries.length }),
                    value: 'all',
                  },
                  {
                    label: t('app.translator.detailFilterUntranslated', {
                      count: detailModal.detail!.entries.filter(e => e.status === 'untranslated').length,
                    }),
                    value: 'untranslated',
                  },
                  {
                    label: t('app.translator.detailFilterSame', {
                      count: detailModal.detail!.entries.filter(e => e.status === 'same_as_source').length,
                    }),
                    value: 'same_as_source',
                  },
                ]}
              />
              <Input
                allowClear
                prefix={<SearchOutlined />}
                placeholder={t('app.translator.detailSearchPlaceholder')}
                style={{ flex: 1, minWidth: 180 }}
                value={detailModal.searchKey}
                onChange={(e) => setDetailModal(prev => ({ ...prev, searchKey: e.target.value }))}
              />
            </div>

            {(() => {
              const filtered = detailModal.detail!.entries.filter(e => {
                if (detailModal.filter !== 'all' && e.status !== detailModal.filter) return false;
                if (detailModal.searchKey) {
                  const k = detailModal.searchKey.toLowerCase();
                  return e.key.toLowerCase().includes(k)
                    || e.source.toLowerCase().includes(k)
                    || e.current.toLowerCase().includes(k);
                }
                return true;
              });
              if (filtered.length === 0) {
                return <Empty description={t('app.translator.detailNoMatches')} style={{ padding: '30px 0' }} />;
              }
              return (
                <Table
                  dataSource={filtered}
                  rowKey="key"
                  size="small"
                  pagination={{ pageSize: 50, showSizeChanger: false }}
                  columns={[
                    {
                      title: t('app.translator.detailColKey'),
                      dataIndex: 'key',
                      key: 'key',
                      width: 240,
                      render: (v: string) => (
                        <span style={{ fontFamily: 'monospace', fontSize: 11, color: '#722ed1', wordBreak: 'break-all' }}>
                          {v}
                        </span>
                      ),
                    },
                    {
                      title: t('app.translator.detailColSource'),
                      dataIndex: 'source',
                      key: 'source',
                      render: (v: string) => (
                        <span style={{ color: 'var(--svl-text-secondary, #aaa)' }}>{v}</span>
                      ),
                    },
                    {
                      title: t('app.translator.detailColCurrent'),
                      dataIndex: 'current',
                      key: 'current',
                      width: 140,
                      render: (v: string, record: UntranslatedEntry) => {
                        if (!v) {
                          return <Text type="secondary" style={{ fontStyle: 'italic' }}>{t('app.translator.detailEmpty')}</Text>;
                        }
                        return (
                          <Tag color={record.status === 'same_as_source' ? 'warning' : 'default'}>
                            {t('app.translator.detailSameAsSource')}
                          </Tag>
                        );
                      },
                    },
                  ]}
                />
              );
            })()}
          </div>
        )}
      </Modal>

      <div style={{ marginTop: 16 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
          <HistoryOutlined style={{ color: 'var(--svl-primary)' }} />
          <Text strong style={{ fontSize: 13 }}>{t('app.translator.backupHistory')}</Text>
          {backups.length > 0 && (
            <Text type="secondary" style={{ fontSize: 11 }}>({backups.length})</Text>
          )}
          <Button type="link" size="small" icon={<ReloadOutlined />} onClick={loadBackups} loading={backupsLoading} />
        </div>
        <div style={{ maxHeight: 200, overflowY: 'auto', borderRadius: 8, border: '1px solid var(--svl-border)', padding: 8 }}>
          {backups.length === 0 ? (
            <Text type="secondary" style={{ fontSize: 12, display: 'block', textAlign: 'center', padding: '12px 0' }}>
              {backupsLoading ? t('app.translator.scanning') : t('app.translator.noBackups')}
            </Text>
          ) : (
            backups.map((b, i) => (
              <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 0', borderBottom: i < backups.length - 1 ? '1px solid var(--svl-border-light)' : 'none' }}>
                <UndoOutlined style={{ color: 'var(--svl-primary)', fontSize: 12 }} />
                <Text style={{ flex: 1, fontSize: 12 }} ellipsis>{b.relative_path}</Text>
                <Text type="secondary" style={{ fontSize: 10, flexShrink: 0 }}>
                  {new Date(b.backup_time * 1000).toLocaleDateString()}
                </Text>
                <Button
                  type="link"
                  size="small"
                  style={{ fontSize: 11, padding: '0 4px' }}
                  onClick={() => handleRestoreBackup(b.original_path)}
                >
                  {t('app.translator.restoreBackup')}
                </Button>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
