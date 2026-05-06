import { useState, useEffect, useCallback, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { message, Tooltip, notification, Modal, Typography } from 'antd';
import {
  FolderOpenOutlined,
  RocketOutlined,
  LoadingOutlined,
  ClockCircleOutlined,
  QuestionCircleOutlined,
} from '@ant-design/icons';
import {
  detectGamePath,
  checkSmapiStatus,
  scanMods,
  launchGame,
  getGameSessionInfo,
  restoreSvlWindow,
  toggleMod,
  deleteMod,
  checkAllModsUpdates,
  type SmapiInfo,
  type ModInfo,
  type GamePathInfo,
  type ProfileListItem,
  type ModUpdateStatus,
} from '../utils/tauri-api';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { useModTags } from '../hooks/useModTags';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { openUrl } from '../utils/openUrl';
import ModList from '../components/ModList';
import ModDetail from '../components/ModDetail';
import StatusBar from '../components/StatusBar';
import DropZone from '../components/DropZone';
import ProfileSelector from '../components/ProfileSelector';
import Onboarding from '../components/Onboarding';
import ModInstallWizard from '../components/ModInstallWizard';
import ApiKeyReminder from '../components/ApiKeyReminder';

const SMAPI_OFFICIAL_URL = 'https://smapi.io';

type FilterType = 'all' | 'enabled' | 'disabled';
type SortType = 'name-az' | 'name-za' | 'author' | 'version';
type CategoryFilter = 'all' | 'visual' | 'gameplay' | 'expansion' | 'framework' | 'ui' | 'seasonal' | 'multiplayer' | 'other';
type StatusFilter = 'all' | 'hasUpdate' | 'hasConflict' | 'uncategorized';

export default function ModManager() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { getTags, addTag, removeTag, getAllUniqueTags } = useModTags();
  const [detecting, setDetecting] = useState(false);
  const [launching, setLaunching] = useState(false);
  const [gameRunning, setGameRunning] = useState(false);
  const [gameDuration, setGameDuration] = useState('');
  const [smapiInfo, setSmapiInfo] = useState<SmapiInfo | null>(null);
  const [gamePathInfo, setGamePathInfo] = useState<GamePathInfo | null>(null);
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [searchText, setSearchText] = useState('');
  const [filterType, setFilterType] = useState<FilterType>('all');
  const [categoryFilter, setCategoryFilter] = useState<CategoryFilter>('all');
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('all');
  const [sortBy, setSortBy] = useState<SortType>('name-az');
  const [selectedMod, setSelectedMod] = useState<ModInfo | null>(null);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [showInstallWizard, setShowInstallWizard] = useState(false);
  const [updateStatuses, setUpdateStatuses] = useState<ModUpdateStatus[]>([]);
  const [checkingUpdates, setCheckingUpdates] = useState(false);

  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingRefreshRef = useRef(false);
  const installProgressUnlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    const firstRun = localStorage.getItem('svl-first-run');
    if (firstRun === null) {
      setShowOnboarding(true);
      localStorage.setItem('svl-first-run', 'false');
    }
    handleInit();

    const setupListener = async () => {
      if (installProgressUnlistenRef.current) {
        installProgressUnlistenRef.current();
        installProgressUnlistenRef.current = null;
      }

      const unlisten = await listen('mod-install-progress', (event) => {
        const payload = event.payload as { step: string; mod_name?: string; message?: string };
        console.log('[ModManager] mod-install-progress event received:', payload);
        if (payload.step === 'done') {
          console.log('[ModManager] install done, scheduling debounced refresh');
          if (refreshTimerRef.current) {
            pendingRefreshRef.current = true;
            return;
          }
          refreshTimerRef.current = setTimeout(() => {
            refreshTimerRef.current = null;
            handleRefresh();
            invoke('check_smapi_log').catch(() => {});
            if (pendingRefreshRef.current) {
              pendingRefreshRef.current = false;
              refreshTimerRef.current = setTimeout(() => {
                refreshTimerRef.current = null;
                handleRefresh();
                invoke('check_smapi_log').catch(() => {});
              }, 500);
            }
          }, 1000);
        }
      });

      installProgressUnlistenRef.current = unlisten;
    };

    setupListener();

    return () => {
      if (installProgressUnlistenRef.current) {
        installProgressUnlistenRef.current();
        installProgressUnlistenRef.current = null;
      }
      if (refreshTimerRef.current) {
        clearTimeout(refreshTimerRef.current);
        refreshTimerRef.current = null;
      }
    };
  }, []);

  useEffect(() => {
    if (!gameRunning) return;
    const interval = setInterval(() => {
      getGameSessionInfo().then((info) => {
        if (!info.is_running) {
          setGameRunning(false);
          setGameDuration('');
          restoreSvlWindow();
          getCurrentWindow().unminimize().catch(() => {});
          message.info(t('app.launchBar.gameExited'));
        } else {
          const pidStr = info.pid ? `PID: ${info.pid}` : '';
          setGameDuration(pidStr);
        }
      }).catch(() => {});
    }, 5000);
    return () => clearInterval(interval);
  }, [gameRunning, t]);

  const handleInit = async () => {
    try {
      setDetecting(true);
      const pathInfo = await detectGamePath();
      console.log('[handleInit] Game path info:', pathInfo);
      setGamePathInfo(pathInfo);

      if (pathInfo.detected_path) {
        console.log('[handleInit] Detected path:', pathInfo.detected_path);
        const status = await checkSmapiStatus(pathInfo.detected_path);
        console.log('[handleInit] SMAPI status:', status);
        setSmapiInfo(status);
        if (status.installed && status.game_path) {
          try {
            console.log('[handleInit] Scanning mods with game_path:', status.game_path);
            const modList = await scanMods(status.game_path);
            console.log('[handleInit] Found', modList.length, 'mods:', modList.map(m => m.name));
            setMods(modList);
          } catch (scanErr) {
            console.error('scanMods failed:', scanErr);
            message.error(typeof scanErr === 'string' ? scanErr : String(scanErr));
            setMods([]);
          }
        } else {
          console.log('[handleInit] SMAPI not installed or no game_path');
        }
      } else {
        console.log('[handleInit] No detected path');
      }
    } catch (err) {
      console.error('handleInit failed:', err);
      message.error(t('app.pages.modManager.detectFailed'));
    } finally {
      setDetecting(false);
    }
  };

  const handleLaunchGame = async () => {
    try {
      setLaunching(true);
      const path = smapiInfo?.game_path || gamePathInfo?.detected_path;
      const result = await launchGame(path || '');
      setGameRunning(true);

      notification.info({
        message: t('app.launchBar.launchNotification'),
        description: result.message,
        duration: 3,
        placement: 'bottomRight',
      });

      getCurrentWindow().minimize().catch(() => {});
    } catch {
    } finally {
      setLaunching(false);
    }
  };

  const handleDownloadSmapi = async () => {
    await openUrl(SMAPI_OFFICIAL_URL, t('app.smapiInstaller.openUrlFailed'));
  };

  const handleFindGameDir = useCallback(() => {
    const steamPaths = [
      'C:\\Program Files (x86)\\Steam\\steamapps\\common\\Stardew Valley',
      'C:\\Program Files\\Steam\\steamapps\\common\\Stardew Valley',
    ];
    
    Modal.info({
      title: t('app.pages.modManager.findGameDir'),
      content: (
        <div>
          <Typography.Paragraph>
            {t('app.pages.modManager.gameRootDirHint')}
          </Typography.Paragraph>
          <Typography.Paragraph strong>
            {t('app.pages.modManager.commonSteamPaths')}
          </Typography.Paragraph>
          {steamPaths.map((path, index) => (
            <Typography.Paragraph key={index} style={{ fontFamily: 'monospace', fontSize: 12 }}>
              {path}
            </Typography.Paragraph>
          ))}
        </div>
      ),
      okText: t('app.common.ok'),
      width: 600,
    });
  }, [t]);

  const handleOpenGamePath = useCallback(async () => {
    const gamePath = smapiInfo?.game_path || gamePathInfo?.detected_path;
    if (gamePath) {
      try {
        console.log('[handleOpenGamePath] revealing:', gamePath);
        await revealItemInDir(gamePath);
      } catch (err) {
        console.error('[handleOpenGamePath] failed:', err);
        message.error(t('app.smapiInstaller.openPathFailed'));
      }
    }
  }, [smapiInfo, gamePathInfo, t]);

  const handleRefresh = async () => {
    console.log('[handleRefresh] Starting diagnostic refresh...');
    console.time('handleRefresh');
    const path = smapiInfo?.game_path || gamePathInfo?.detected_path;
    console.log('[handleRefresh] Resolved path:', path);
    console.log('[handleRefresh] smapiInfo:', smapiInfo);
    console.log('[handleRefresh] gamePathInfo:', gamePathInfo);
    if (path) {
      try {
        const modList = await scanMods(path);
        console.log('[handleRefresh] scanMods returned', modList.length, 'mods');
        setMods(modList);
      } catch (err) {
        console.error('[handleRefresh] scanMods failed:', err);
        message.error(typeof err === 'string' ? err : String(err));
      }
    } else {
      console.warn('[handleRefresh] no game path available');
    }
    console.timeEnd('handleRefresh');
    console.log('[handleRefresh] Diagnostic refresh complete.');
  };

  const handleInstallSuccess = () => {
    console.log('[handleInstallSuccess] triggered, scheduling refresh in 800ms');
    setTimeout(() => {
      handleRefresh();
    }, 800);
  };

  const handleProfileChange = (_profile: ProfileListItem) => {
    setMods(prevMods =>
      prevMods.map(mod => ({
        ...mod,
        enabled: true,
      }))
    );
    handleRefresh();
  };

  const handleToggleMod = async (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;
    try {
      const extraPaths = mod.is_group && mod.sub_mods.length > 0
        ? mod.sub_mods.map(sm => sm.folder_path)
        : undefined;
      console.log('[handleToggleMod]', { modId, folder_path: mod.folder_path, enabled: mod.enabled, newEnabled: !mod.enabled, extraPaths, is_group: mod.is_group, sub_mods: mod.sub_mods });
      const result = await toggleMod(mod.folder_path, !mod.enabled, extraPaths);
      console.log('[handleToggleMod] result:', result);
      if (result) {
        message.success(t('app.modCard.toggleSuccess'));
        handleRefresh();
      } else {
        message.error(t('app.modCard.toggleFailed'));
      }
    } catch (err) {
      const detail = typeof err === 'string' ? err : String(err);
      console.error('[handleToggleMod] error:', err);
      message.error(`${t('app.modCard.toggleFailed')}: ${detail}`);
    }
  };

  const handleDeleteMod = async (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;
    try {
      await deleteMod(mod.folder_path);
      handleRefresh();
      message.success(t('app.modCard.uninstallSuccess'));
    } catch {
      message.error(t('app.modCard.uninstallFailed'));
    }
  };

  const handleOpenModFolder = async (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;
    try {
      await revealItemInDir(mod.folder_path);
    } catch (err) {
      console.error('[handleOpenModFolder] failed:', err);
      message.error(t('app.smapiInstaller.openPathFailed'));
    }
  };

  const handleCheckUpdate = async (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;
    try {
      const updates = await invoke('check_mod_updates', { uniqueId: mod.unique_id });
      if (updates && (updates as any).has_update) {
        message.info(t('app.modDetail.updateAvailable'));
        setSelectedMod({ ...mod, has_update: true });
      } else {
        message.success(t('app.modCard.upToDate'));
      }
    } catch (err) {
      console.error('[handleCheckUpdate] failed:', err);
      message.error(t('app.modCard.checkUpdateFailed'));
    }
  };

  const handleCheckAllUpdates = async () => {
    if (mods.length === 0) {
      message.warning(t('app.modList.noModsToCheck'));
      return;
    }

    setCheckingUpdates(true);
    try {
      const apiKey = localStorage.getItem('nexus-api-key') || '';
      const updates = await checkAllModsUpdates(
        mods.map(m => ({
          unique_id: m.unique_id,
          name: m.name,
          version: m.version,
          folder_path: m.folder_path,
          nexus_mod_id: m.url?.match(/mods\/(\d+)/)?.[1] || null,
        })),
        apiKey || undefined
      );

      setUpdateStatuses(updates);
      
      const updateCount = updates.filter(u => u.has_update).length;
      if (updateCount > 0) {
        message.info(t('app.modList.foundUpdates', { count: updateCount }));
      } else {
        message.success(t('app.modList.allUpToDate'));
      }

      setMods(prevMods =>
        prevMods.map(mod => {
          const update = updates.find(u => u.unique_id === mod.unique_id);
          return update && update.has_update
            ? { ...mod, has_update: true, latest_version: update.latest_version }
            : { ...mod, has_update: false, latest_version: null };
        })
      );
    } catch (err) {
      console.error('[handleCheckAllUpdates] failed:', err);
      message.error(t('app.modList.checkUpdatesFailed'));
    } finally {
      setCheckingUpdates(false);
    }
  };

  const handleBatchUpdate = async () => {
    const modsToUpdate = updateStatuses.filter(u => u.has_update);
    if (modsToUpdate.length === 0) {
      message.warning(t('app.modList.noModsToUpdate'));
      return;
    }

    Modal.confirm({
      title: t('app.modList.batchUpdateConfirm', { count: modsToUpdate.length }),
      content: t('app.modList.batchUpdateWarning'),
      okText: t('app.modList.batchUpdate'),
      cancelText: t('app.common.cancel'),
      onOk: async () => {
        try {
          const apiKey = localStorage.getItem('nexus-api-key') || '';
          const result = await invoke<{updated: number, total: number}>('batch_update_mods', {
            modsToUpdate: modsToUpdate.map(u => ({
              unique_id: u.unique_id,
              name: u.name,
              download_url: u.download_url,
            })),
            apiKey,
          });

          message.success(t('app.modList.batchUpdateSuccess', {
            updated: result.updated,
            total: result.total,
          }));

          handleRefresh();
        } catch (err) {
          console.error('[handleBatchUpdate] failed:', err);
          message.error(t('app.modList.batchUpdateFailed'));
        }
      },
    });
  };

  const gameFound = !!gamePathInfo?.detected_path;
  const smapiInstalled = smapiInfo?.installed || false;
  const gamePath = smapiInfo?.game_path || gamePathInfo?.detected_path || '';
  const modsPath = gamePath ? `${gamePath}\\Mods` : '';

  const filteredMods = mods.filter(mod => {
    if (searchText) {
      const lowerSearch = searchText.toLowerCase();
      const nameMatch = mod.name.toLowerCase().includes(lowerSearch);
      const authorMatch = mod.author.toLowerCase().includes(lowerSearch);
      const idMatch = mod.unique_id.toLowerCase().includes(lowerSearch);
      const tagMatch = (getTags(mod.unique_id) || []).some(tag =>
        tag.toLowerCase().includes(lowerSearch),
      );
      if (!nameMatch && !authorMatch && !idMatch && !tagMatch) {
        return false;
      }
    }

    if (filterType === 'enabled' && !mod.enabled) return false;
    if (filterType === 'disabled' && mod.enabled) return false;

    if (categoryFilter !== 'all' && mod.category !== categoryFilter) return false;

    if (statusFilter === 'hasUpdate' && !mod.has_update) return false;
    if (statusFilter === 'hasConflict' && !mod.has_conflict) return false;
    if (statusFilter === 'uncategorized' && mod.category !== 'other') return false;

    return true;
  }).sort((a, b) => {
    switch (sortBy) {
      case 'name-az': return a.name.localeCompare(b.name);
      case 'name-za': return b.name.localeCompare(a.name);
      case 'author': return a.author.localeCompare(b.author);
      case 'version': return a.version.localeCompare(b.version);
      default: return 0;
    }
  });

  return (
    <>
      <Onboarding
        visible={showOnboarding}
        onComplete={() => setShowOnboarding(false)}
        gamePath={gamePath}
        smapiInstalled={smapiInstalled}
        onDetectGame={handleInit}
        onInstallSmapi={handleDownloadSmapi}
      />

      <ApiKeyReminder />

      <div className="svl-header">
        <div className="svl-search-wrapper">
          <span className="svl-search-icon">🔍</span>
          <input
            type="text"
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            placeholder={t('app.pages.modManager.searchPlaceholder')}
          />
        </div>

        <ProfileSelector
          onProfileChange={handleProfileChange}
          onManageProfiles={() => navigate('/profiles')}
        />

        <button
          className="svl-check-updates-btn"
          onClick={handleCheckAllUpdates}
          disabled={checkingUpdates || mods.length === 0}
        >
          {checkingUpdates ? t('app.modList.checkingUpdates') : t('app.modList.checkAllUpdates')}
        </button>

        {updateStatuses.filter(u => u.has_update).length > 0 && (
          <button
            className="svl-batch-update-btn"
            onClick={handleBatchUpdate}
          >
            {t('app.modList.batchUpdate', { count: updateStatuses.filter(u => u.has_update).length })}
          </button>
        )}

        <div className="svl-header-actions">
          <div className="svl-filter-group">
            <button
              onClick={() => setFilterType('all')}
              className={`svl-filter-btn ${filterType === 'all' ? 'active' : ''}`}
            >
              {t('app.pages.modManager.filterAll')}
            </button>
            <button
              onClick={() => setFilterType('enabled')}
              className={`svl-filter-btn ${filterType === 'enabled' ? 'active' : ''}`}
            >
              {t('app.pages.modManager.filterEnabled')}
            </button>
            <button
              onClick={() => setFilterType('disabled')}
              className={`svl-filter-btn ${filterType === 'disabled' ? 'active' : ''}`}
            >
              {t('app.pages.modManager.filterDisabled')}
            </button>
          </div>

          <div className="svl-filter-group">
            <select
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value as CategoryFilter)}
              className="svl-category-select"
            >
              <option value="all">{t('app.categories.all')}</option>
              <option value="visual">{t('app.categories.visual')}</option>
              <option value="gameplay">{t('app.categories.gameplay')}</option>
              <option value="expansion">{t('app.categories.expansion')}</option>
              <option value="framework">{t('app.categories.framework')}</option>
              <option value="ui">{t('app.categories.ui')}</option>
              <option value="seasonal">{t('app.categories.seasonal')}</option>
              <option value="multiplayer">{t('app.categories.multiplayer')}</option>
              <option value="other">{t('app.categories.other')}</option>
            </select>

            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
              className="svl-status-select"
            >
              <option value="all">{t('app.statusFilters.all')}</option>
              <option value="hasUpdate">{t('app.statusFilters.hasUpdate')}</option>
              <option value="hasConflict">{t('app.statusFilters.hasConflict')}</option>
              <option value="uncategorized">{t('app.statusFilters.uncategorized')}</option>
            </select>
          </div>

          <div className="svl-sort">
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as SortType)}
            >
              <option value="name-az">{t('app.pages.modManager.sortNameAZ')}</option>
              <option value="name-za">{t('app.pages.modManager.sortNameZA')}</option>
              <option value="author">{t('app.pages.modManager.sortAuthor')}</option>
              <option value="version">{t('app.pages.modManager.sortVersion')}</option>
            </select>
          </div>

        </div>
      </div>

      <div className="svl-game-status-bar">
        {gameFound ? (
          <div className="svl-game-status svl-game-status--found">
            <span className="svl-game-status-icon">✓</span>
            <span className="svl-game-status-text">
              {t('app.pages.modManager.gameFound')}
              <span className="svl-game-path">{gamePath}</span>
            </span>
            <button className="svl-open-game-dir-btn" onClick={handleOpenGamePath} title={t('app.smapiInstaller.openGamePath')}>
              <FolderOpenOutlined />
            </button>
            <button className="svl-change-btn" onClick={handleInit}>
              {t('app.pages.modManager.change')}
            </button>
          </div>
        ) : (
          <div className="svl-game-status svl-game-status--not-found">
            <span className="svl-game-status-icon">✗</span>
            <span className="svl-game-status-text">
              {t('app.pages.modManager.gameNotFound')}
            </span>
          </div>
        )}

        {gameFound && (
          <div className="svl-game-actions">
            {smapiInstalled ? (
              <>
                <span className="svl-smapi-version">
                  {smapiInfo?.version && smapiInfo.version !== 'Installed'
                    ? `SMAPI v${smapiInfo.version}`
                    : t('app.pages.modManager.smapiInstalled')}
                </span>
                {gameRunning && gameDuration && (
                  <span className="svl-game-duration">
                    <ClockCircleOutlined /> {t('app.launchBar.duration', { duration: gameDuration })}
                  </span>
                )}
                <button
                  className="svl-launch-btn"
                  onClick={handleLaunchGame}
                  disabled={launching}
                >
                  {launching ? (
                    <>
                      <LoadingOutlined /> {t('app.launchBar.launching')}
                    </>
                  ) : (
                    <>
                      <RocketOutlined /> {t('app.launchBar.launchViaSmapi')}
                    </>
                  )}
                </button>
              </>
            ) : (
              <>
                <button
                  className="svl-install-smapi-btn"
                  onClick={handleDownloadSmapi}
                >
                  {t('app.pages.modManager.downloadSmapi')}
                </button>
                <Tooltip title={t('app.pages.modManager.smapiRequired')}>
                  <button
                    className="svl-launch-btn"
                    disabled
                  >
                    {t('app.pages.modManager.launchGame')}
                  </button>
                </Tooltip>
              </>
            )}
          </div>
        )}
      </div>

      <div className="svl-content">
        {!gameFound ? (
          <div className="svl-empty-state">
            <div className="svl-empty-icon">🎮</div>
            <div className="svl-empty-title">{t('app.pages.modManager.gameNotFound')}</div>
            <div className="svl-empty-desc">{t('app.pages.modManager.gameNotFoundDesc')}</div>
            <div className="svl-empty-hint" style={{ marginTop: 16, marginBottom: 16, maxWidth: 500 }}>
              <Tooltip title={t('app.pages.modManager.gameRootDirHint')}>
                <QuestionCircleOutlined style={{ marginRight: 8, color: 'var(--svl-primary)' }} />
              </Tooltip>
              <span style={{ fontSize: 14, color: 'var(--svl-text-secondary)' }}>
                {t('app.pages.modManager.gameRootDirHint')}
              </span>
            </div>
            <div style={{ display: 'flex', gap: 12, justifyContent: 'center' }}>
              <button
                className="svl-detect-btn"
                onClick={handleInit}
                disabled={detecting}
              >
                {detecting
                  ? t('app.pages.modManager.detecting')
                  : t('app.pages.modManager.detectGame')}
              </button>
              <button
                className="svl-detect-btn"
                onClick={handleFindGameDir}
                style={{ background: 'var(--svl-surface)', border: '1px solid var(--svl-border)' }}
              >
                {t('app.pages.modManager.findGameDir')}
              </button>
            </div>
          </div>
        ) : !smapiInstalled ? (
          <div className="svl-smapi-install-card">
            <div className="svl-smapi-install-header">
              <div className="svl-smapi-install-icon">🔧</div>
              <div className="svl-smapi-install-title">
                {t('app.pages.modManager.smapiNotInstalled')}
              </div>
            </div>
            <div className="svl-smapi-install-desc">
              {t('app.pages.modManager.installSmapiPrompt')}
            </div>

            <button
              className="svl-smapi-download-btn"
              onClick={handleDownloadSmapi}
            >
              {t('app.pages.modManager.downloadSmapi')}
            </button>
          </div>
        ) : (
          <>
            <DropZone
              modsPath={modsPath}
              onInstallSuccess={handleInstallSuccess}
            />

            <div className="svl-mod-list-wrapper">
              <div className="svl-mod-list-main">
                {filteredMods.length === 0 && mods.length === 0 ? (
                  <div className="svl-empty-state">
                    <div className="svl-empty-icon">📂</div>
                    <div className="svl-empty-title">{t('app.pages.modManager.noModsFound')}</div>
                    <div className="svl-empty-desc">{t('app.pages.modManager.noModsDesc')}</div>
                  </div>
                ) : (
                  <ModList
                    mods={filteredMods}
                    loading={false}
                    onRefresh={handleRefresh}
                    onUninstall={handleDeleteMod}
                    searchText={searchText}
                    filterType={filterType}
                    sortBy={sortBy}
                    statusFilter={statusFilter}
                    onToggleMod={handleToggleMod}
                    onDeleteMod={handleDeleteMod}
                    onSelectMod={setSelectedMod}
                    onOpenModFolder={handleOpenModFolder}
                    onCheckUpdate={handleCheckUpdate}
                    onAddTag={addTag}
                    onRemoveTag={removeTag}
                    getTags={getTags}
                  />
                )}
              </div>

              {selectedMod && (
                <div className="svl-mod-detail-panel">
                  <ModDetail
                    mod={selectedMod}
                    installedMods={mods}
                    onClose={() => setSelectedMod(null)}
                    onAddTag={addTag}
                    onRemoveTag={removeTag}
                    getTags={getTags}
                    allTags={getAllUniqueTags()}
                  />
                </div>
              )}
            </div>
          </>
        )}
      </div>

      <StatusBar
        smapiConnected={smapiInstalled}
        modsCount={mods.length}
      />

      <ModInstallWizard
        visible={showInstallWizard}
        onClose={() => setShowInstallWizard(false)}
        modsPath={modsPath}
        onInstallComplete={handleInstallSuccess}
      />
    </>
  );
}
