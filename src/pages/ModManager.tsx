import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
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
  setCustomGamePath,
  scanMods,
  launchGame,
  getGameSessionInfo,
  restoreSvlWindow,
  toggleMod,
  deleteMod,
  checkAllModsUpdates,
  downloadModUpdate,
  backupModBeforeUpdate,
  profileGetActive,
  profileClearActive,
  parseSmapiLog,
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
import { open } from '@tauri-apps/plugin-dialog';
import { openUrl } from '../utils/openUrl';
import ModList from '../components/ModList';
import ModDetail from '../components/ModDetail';
import StatusBar from '../components/StatusBar';
import DropZone from '../components/DropZone';
import ProfileSelector from '../components/ProfileSelector';
import Onboarding from '../components/Onboarding';
import ModInstallWizard from '../components/ModInstallWizard';
import ApiKeyReminder from '../components/ApiKeyReminder';
import LoadOrderModal from '../components/LoadOrderModal';
import ModConfigEditor from '../components/ModConfigEditor';
import ModBackupManager from '../components/ModBackupManager';
import ModBackupConfirmModal from '../components/ModBackupConfirmModal';
import GameMonitor from '../components/GameMonitor';
import SecurityScanner from '../components/SecurityScanner';
import LogParser from '../components/LogParser';
import DependencyResolver from '../components/DependencyResolver';

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
  const [downloadingModIds, setDownloadingModIds] = useState<Set<string>>(new Set());
  const [showUpdatePanel, setShowUpdatePanel] = useState(false);
  const [selectedUpdateMods, setSelectedUpdateMods] = useState<Set<string>>(new Set());

  const [showLoadOrder, setShowLoadOrder] = useState(false);
  const [showConfigEditor, setShowConfigEditor] = useState(false);
  const [showBackupManager, setShowBackupManager] = useState(false);
  const [showGameMonitor, setShowGameMonitor] = useState(false);
  const [showSecurityScanner, setShowSecurityScanner] = useState(false);
  const [showLogParser, setShowLogParser] = useState(false);
  const [showDepResolver, setShowDepResolver] = useState(false);
  const [selectedModForConfig, setSelectedModForConfig] = useState<ModInfo | null>(null);
  const [selectedModForBackup, setSelectedModForBackup] = useState<ModInfo | null>(null);
  const [activeProfileName, setActiveProfileName] = useState<string | null>(null);

  // Backup confirmation modal state
  const [showBackupConfirm, setShowBackupConfirm] = useState(false);
  const [backupTargetMod, setBackupTargetMod] = useState<{ modPath: string; nexusModId: string; uniqueId?: string } | null>(null);
  const [pendingDownloadNexusId, setPendingDownloadNexusId] = useState<string | null>(null);

  // Batch update backup state
  const [batchBackupQueue, setBatchBackupQueue] = useState<Array<{ modPath: string; name: string; uniqueId: string; version: string }>>([]);
  const [pendingBatchUpdate, setPendingBatchUpdate] = useState<Array<{ unique_id: string; name: string; download_url: string | null; nexus_mod_id: string | null }> | null>(null);

  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingRefreshRef = useRef(false);
  const installProgressUnlistenRef = useRef<(() => void) | null>(null);
  const handleRefreshRef = useRef<(() => Promise<void>) | null>(null);
  const smapiInfoRef = useRef<SmapiInfo | null>(null);
  const gamePathInfoRef = useRef<GamePathInfo | null>(null);

  useEffect(() => {
    smapiInfoRef.current = smapiInfo;
    gamePathInfoRef.current = gamePathInfo;
  });

  useEffect(() => {
    const firstRun = localStorage.getItem('svl-first-run');
    if (firstRun === null) {
      setShowOnboarding(true);
      localStorage.setItem('svl-first-run', 'false');
    }
    handleInit();

    const needsRefresh = sessionStorage.getItem('svl-mod-installed');
    if (needsRefresh === 'true') {
      sessionStorage.removeItem('svl-mod-installed');
      setTimeout(() => {
        if (handleRefreshRef.current) {
          handleRefreshRef.current();
        }
      }, 1200);
    }

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
            if (handleRefreshRef.current) {
              handleRefreshRef.current();
            }
            invoke('check_smapi_log').catch(() => {});
            if (pendingRefreshRef.current) {
              pendingRefreshRef.current = false;
              refreshTimerRef.current = setTimeout(() => {
                refreshTimerRef.current = null;
                if (handleRefreshRef.current) {
                  handleRefreshRef.current();
                }
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
          setLaunching(false);
          setGameDuration('');
          restoreSvlWindow();
          getCurrentWindow().unminimize().catch(() => {});
          message.info(t('app.launchBar.gameExited'));

          const path = smapiInfo?.game_path || gamePathInfo?.detected_path;
          if (path && smapiInfo?.installed) {
            parseSmapiLog()
              .then(() => {
                setShowLogParser(true);
              })
              .catch(() => {});
          }
        } else {
          const pidStr = info.pid ? `PID: ${info.pid}` : '';
          setGameDuration(pidStr);
        }
      }).catch(() => {});
    }, 5000);
    return () => clearInterval(interval);
  }, [gameRunning, t, smapiInfo, gamePathInfo]);

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
            const activeProfile = await profileGetActive(status.game_path);
            setActiveProfileName(activeProfile);
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
    } catch (err) {
      const detail = typeof err === 'string' ? err : String(err);
      message.error(`${t('app.launchBar.launchFailed')}: ${detail}`);
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

  const handleChangeGamePath = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t('app.pages.modManager.selectGameDir'),
      });
      if (!selected) return;

      const dirPath = typeof selected === 'string' ? selected : selected;
      setDetecting(true);
      const pathInfo = await setCustomGamePath(dirPath);
      setGamePathInfo(pathInfo);

      if (pathInfo.detected_path) {
        const status = await checkSmapiStatus(pathInfo.detected_path);
        setSmapiInfo(status);
        if (status.installed && status.game_path) {
          const modList = await scanMods(status.game_path);
          setMods(modList);
          message.success(t('app.pages.modManager.gamePathChanged'));
        } else {
          message.warning(t('app.errors.smapiNotInstalled'));
        }
      } else {
        message.error(t('app.errors.gamePathNotFound'));
      }
    } catch (err) {
      console.error('[handleChangeGamePath] failed:', err);
      message.error(t('app.errors.gamePathNotFound'));
    } finally {
      setDetecting(false);
    }
  }, [t]);

  const handleRefresh = useCallback(async () => {
    console.log('[handleRefresh] Starting diagnostic refresh...');
    console.time('handleRefresh');
    const currentSmapi = smapiInfoRef.current;
    const currentGamePath = gamePathInfoRef.current;
    const path = currentSmapi?.game_path || currentGamePath?.detected_path;
    console.log('[handleRefresh] Resolved path from refs:', path);
    console.log('[handleRefresh] smapiInfoRef.game_path:', currentSmapi?.game_path);
    console.log('[handleRefresh] gamePathInfoRef.detected_path:', currentGamePath?.detected_path);
    if (path) {
      try {
        const modList = await scanMods(path);
        console.log('[handleRefresh] scanMods returned', modList.length, 'mods:', modList.map(m => m.name));
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
  }, []);

  useEffect(() => {
    handleRefreshRef.current = handleRefresh;
  });

  const handleInstallSuccess = useCallback(() => {
    console.log('[handleInstallSuccess] triggered, scheduling refresh in 800ms');
    setTimeout(() => {
      console.log('[handleInstallSuccess] timeout fired, calling handleRefreshRef.current');
      if (handleRefreshRef.current) {
        handleRefreshRef.current();
      } else {
        console.error('[handleInstallSuccess] handleRefreshRef.current is null!');
      }
    }, 800);
  }, []);

  const handleProfileChange = (_profile: ProfileListItem) => {
    setMods(prevMods =>
      prevMods.map(mod => ({
        ...mod,
        enabled: true,
      }))
    );
    setActiveProfileName(_profile.name);
    handleRefresh();
  };

  const handleToggleMod = async (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;

    if (activeProfileName) {
      Modal.confirm({
        title: t('app.modCard.exitProfileTitle'),
        content: t('app.modCard.exitProfileContent', { profile: activeProfileName }),
        okText: t('app.profiles.exitProfile'),
        cancelText: t('app.common.cancel'),
        onOk: async () => {
          const path = smapiInfo?.game_path || gamePathInfo?.detected_path;
          if (path) {
            try {
              await profileClearActive(path);
              setActiveProfileName(null);
              doToggleMod(modId);
            } catch {
              message.error(t('app.modCard.toggleFailed'));
            }
          }
        },
      });
      return;
    }

    doToggleMod(modId);
  };

  const doToggleMod = async (modId: string) => {
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

  const handleEnableAllMods = async () => {
    if (activeProfileName) {
      Modal.confirm({
        title: t('app.modCard.exitProfileTitle'),
        content: t('app.modCard.exitProfileContent', { profile: activeProfileName }),
        okText: t('app.profiles.exitProfile'),
        cancelText: t('app.common.cancel'),
        onOk: async () => {
          const path = smapiInfo?.game_path || gamePathInfo?.detected_path;
          if (path) {
            try {
              await profileClearActive(path);
              setActiveProfileName(null);
              doEnableAllMods();
            } catch (err) {
              message.error(String(err));
            }
          }
        },
      });
      return;
    }
    doEnableAllMods();
  };

  const doEnableAllMods = async () => {
    const disabledMods = mods.filter(m => !m.enabled);
    if (disabledMods.length === 0) {
      message.info(t('app.modList.batchEnableSuccess', { count: 0 }));
      return;
    }
    let successCount = 0;
    for (const mod of disabledMods) {
      try {
        const extraPaths = mod.is_group && mod.sub_mods.length > 0
          ? mod.sub_mods.map(sm => sm.folder_path)
          : undefined;
        const result = await toggleMod(mod.folder_path, true, extraPaths);
        if (result) successCount++;
      } catch {}
    }
    message.success(t('app.modList.batchEnableSuccess', { count: successCount }));
    handleRefresh();
  };

  const handleDisableAllMods = async () => {
    if (activeProfileName) {
      Modal.confirm({
        title: t('app.modCard.exitProfileTitle'),
        content: t('app.modCard.exitProfileContent', { profile: activeProfileName }),
        okText: t('app.profiles.exitProfile'),
        cancelText: t('app.common.cancel'),
        onOk: async () => {
          const path = smapiInfo?.game_path || gamePathInfo?.detected_path;
          if (path) {
            try {
              await profileClearActive(path);
              setActiveProfileName(null);
              doDisableAllMods();
            } catch (err) {
              message.error(String(err));
            }
          }
        },
      });
      return;
    }
    doDisableAllMods();
  };

  const doDisableAllMods = async () => {
    const enabledMods = mods.filter(m => m.enabled && !m.is_required);
    if (enabledMods.length === 0) {
      message.info(t('app.modList.batchDisableSuccess', { count: 0 }));
      return;
    }
    let successCount = 0;
    for (const mod of enabledMods) {
      try {
        const extraPaths = mod.is_group && mod.sub_mods.length > 0
          ? mod.sub_mods.map(sm => sm.folder_path)
          : undefined;
        const result = await toggleMod(mod.folder_path, false, extraPaths);
        if (result) successCount++;
      } catch {}
    }
    message.success(t('app.modList.batchDisableSuccess', { count: successCount }));
    handleRefresh();
  };

  const handleDeleteMod = async (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;

    Modal.confirm({
      title: t('app.modCard.uninstall'),
      content: t('app.modList.batchDeleteWarning'),
      okText: t('app.common.delete'),
      cancelText: t('app.common.cancel'),
      okButtonProps: { danger: true },
      onOk: async () => {
        try {
          await deleteMod(mod.folder_path);
          setSelectedMod(null);
          handleRefresh();
          message.success(t('app.modCard.uninstallSuccess'));
        } catch {
          message.error(t('app.modCard.uninstallFailed'));
        }
      },
    });
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
      const apiKey = localStorage.getItem('svl-nexus-api-key') || undefined;
      const result = await invoke<{ unique_id: string; has_update: boolean; latest_version: string | null; update_source: string | null }>('check_single_mod_update', {
        uniqueId: mod.unique_id,
        currentVersion: mod.version,
        modFolderPath: mod.folder_path,
        apiKey: apiKey || null,
      });
      if (result && result.has_update) {
        message.info(t('app.modDetail.updateAvailable'));
        setSelectedMod({ ...mod, has_update: true, latest_version: result.latest_version });
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
      const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
      const updates = await checkAllModsUpdates(
        mods.map(m => ({
          unique_id: m.unique_id,
          name: m.name,
          version: m.version,
          folder_path: m.folder_path,
          nexus_mod_id: m.nexus_mod_id ? String(m.nexus_mod_id) : null,
        })),
        apiKey || undefined
      );

      setUpdateStatuses(updates);
      setShowUpdatePanel(true);
      
      const updatable = updates.filter(u => u.has_update);
      setSelectedUpdateMods(new Set(updatable.map(u => u.unique_id)));
      
      const updateCount = updatable.length;
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

  const handleBatchUpdate = () => {
    const modsToUpdate = updateStatuses.filter(u => u.has_update && selectedUpdateMods.has(u.unique_id));
    if (modsToUpdate.length === 0) {
      message.warning(t('app.modList.noSelectedMods'));
      return;
    }

    // Find mods that have existing installations and need backup
    const modsNeedingBackup: Array<{ modPath: string; name: string; uniqueId: string; version: string }> = [];
    for (const u of modsToUpdate) {
      const existingMod = mods.find(m => m.unique_id === u.unique_id);
      if (existingMod && existingMod.folder_path) {
        modsNeedingBackup.push({
          modPath: existingMod.folder_path,
          name: existingMod.name,
          uniqueId: existingMod.unique_id,
          version: existingMod.version,
        });
      }
    }

    if (modsNeedingBackup.length > 0) {
      // Show backup confirmation for the first mod
      setBatchBackupQueue(modsNeedingBackup);
      setPendingBatchUpdate(modsToUpdate.map(u => ({
        unique_id: u.unique_id,
        name: u.name,
        download_url: u.download_url,
        nexus_mod_id: u.nexus_mod_id,
      })));
      setBackupTargetMod({
        modPath: modsNeedingBackup[0].modPath,
        nexusModId: '',
        uniqueId: modsNeedingBackup[0].uniqueId,
      });
      setShowBackupConfirm(true);
      return;
    }

    // No backups needed, proceed directly
    executeBatchUpdate(modsToUpdate);
  };

  const executeBatchUpdate = async (modsToUpdate: typeof updateStatuses) => {
    try {
      const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
      const result = await invoke<{updated: number, total: number}>('batch_update_mods', {
        modsToUpdate: modsToUpdate.map(u => ({
          unique_id: u.unique_id,
          name: u.name,
          download_url: u.download_url,
          nexus_mod_id: u.nexus_mod_id,
        })),
        apiKey,
        modsPath: modsPath,
      });

      message.success(t('app.modList.batchUpdateSuccess', {
        updated: result.updated,
        total: result.total,
      }));

      handleRefresh();
      setShowUpdatePanel(false);
      setSelectedUpdateMods(new Set());
    } catch (err) {
      console.error('[handleBatchUpdate] failed:', err);
      message.error(t('app.modList.batchUpdateFailed'));
    }
  };

  const toggleUpdateModSelect = (uniqueId: string) => {
    setSelectedUpdateMods(prev => {
      const next = new Set(prev);
      if (next.has(uniqueId)) {
        next.delete(uniqueId);
      } else {
        next.add(uniqueId);
      }
      return next;
    });
  };

  const selectAllUpdateMods = () => {
    setSelectedUpdateMods(new Set(
      updateStatuses.filter(u => u.has_update).map(u => u.unique_id)
    ));
  };

  const deselectAllUpdateMods = () => {
    setSelectedUpdateMods(new Set());
  };

  const invertUpdateModSelection = () => {
    const updatableIds = new Set(updateStatuses.filter(u => u.has_update).map(u => u.unique_id));
    setSelectedUpdateMods(prev => {
      const next = new Set<string>();
      updatableIds.forEach(id => {
        if (!prev.has(id)) next.add(id);
      });
      return next;
    });
  };

  const handleDownloadModUpdate = async (nexusModId: string, uniqueId?: string) => {
    const apiKey = localStorage.getItem('svl-nexus-api-key') || '';
    if (!apiKey) {
      message.warning(t('app.logParser.needApiKey'));
      return;
    }
    if (!modsPath) {
      message.error(t('app.pages.modManager.gameNotFound'));
      return;
    }

    // Find the mod to backup
    const targetMod = mods.find(m => m.unique_id === uniqueId);
    if (targetMod && targetMod.folder_path) {
      setBackupTargetMod({ modPath: targetMod.folder_path, nexusModId, uniqueId });
      setPendingDownloadNexusId(nexusModId);
      setShowBackupConfirm(true);
    } else {
      // No mod found or no path, proceed without backup
      await doDownloadModUpdate(nexusModId, uniqueId, apiKey);
    }
  };

  // Internal function: actually performs the download
  const doDownloadModUpdate = async (nexusModId: string, uniqueId?: string, apiKey?: string) => {
    const key = apiKey || localStorage.getItem('svl-nexus-api-key') || '';
    setDownloadingModIds(prev => new Set(prev).add(nexusModId));
    try {
      const result = await downloadModUpdate(nexusModId, key, modsPath, uniqueId);
      message.success(result);
      handleRefresh();
      setUpdateStatuses(prev => prev.filter(u => u.nexus_mod_id !== nexusModId));
    } catch (err) {
      console.error('[doDownloadModUpdate] failed:', err);
      message.error(typeof err === 'string' ? err : String(err));
    } finally {
      setDownloadingModIds(prev => {
        const next = new Set(prev);
        next.delete(nexusModId);
        return next;
      });
    }
  };

  // Backup confirmation handlers
  const handleBackupConfirm = async (customBackupDir: string | null) => {
    // If we're in batch update mode, process the batch backup queue
    if (pendingBatchUpdate && batchBackupQueue.length > 0) {
      await processBatchBackupQueue(customBackupDir);
      return;
    }

    // Single mod update mode
    if (!backupTargetMod) return;
    const { modPath, nexusModId, uniqueId } = backupTargetMod;
    setShowBackupConfirm(false);
    try {
      await backupModBeforeUpdate(modPath, customBackupDir || undefined);
      message.success('备份完成，正在下载更新...');
      await doDownloadModUpdate(nexusModId, uniqueId);
    } catch (err) {
      console.error('[handleBackupConfirm] backup failed:', err);
      message.error('备份失败，但将继续下载更新');
      await doDownloadModUpdate(nexusModId, uniqueId);
    }
    setBackupTargetMod(null);
    setPendingDownloadNexusId(null);
  };

  const processBatchBackupQueue = async (customBackupDir: string | null) => {
    const queue = [...batchBackupQueue];
    if (queue.length === 0) {
      setBatchBackupQueue([]);
      setPendingBatchUpdate(null);
      setBackupTargetMod(null);
      if (pendingBatchUpdate) {
        await executeBatchUpdate(
          updateStatuses.filter(u => 
            pendingBatchUpdate.some(p => p.unique_id === u.unique_id)
          )
        );
      }
      return;
    }

    // Backup each mod in the queue sequentially
    for (let i = 0; i < queue.length; i++) {
      const current = queue[i];
      try {
        await backupModBeforeUpdate(current.modPath, customBackupDir || undefined);
      } catch (err) {
        console.error('[processBatchBackupQueue] backup failed:', err);
        message.warning(`备份 ${current.name} 失败，将继续更新`);
      }
    }

    // All backups complete, now execute the batch update
    setBatchBackupQueue([]);
    setPendingBatchUpdate(null);
    setBackupTargetMod(null);
    if (pendingBatchUpdate) {
      await executeBatchUpdate(
        updateStatuses.filter(u => 
          pendingBatchUpdate.some(p => p.unique_id === u.unique_id)
        )
      );
    }
  };

  const handleBackupSkip = async () => {
    setShowBackupConfirm(false);
    
    // If we're in batch update mode
    if (pendingBatchUpdate) {
      // Skip remaining backups and execute batch update
      setBatchBackupQueue([]);
      setPendingBatchUpdate(null);
      setBackupTargetMod(null);
      await executeBatchUpdate(
        updateStatuses.filter(u => 
          pendingBatchUpdate.some(p => p.unique_id === u.unique_id)
        )
      );
      return;
    }

    // Single mod update mode
    if (pendingDownloadNexusId && backupTargetMod) {
      await doDownloadModUpdate(pendingDownloadNexusId, backupTargetMod.uniqueId);
    }
    setBackupTargetMod(null);
    setPendingDownloadNexusId(null);
  };

  const gameFound = !!gamePathInfo?.detected_path;
  const smapiInstalled = smapiInfo?.installed || false;
  const gamePath = smapiInfo?.game_path || gamePathInfo?.detected_path || '';
  const modsPath = gamePath ? `${gamePath}\\Mods` : '';

  const handleOpenConfigEditor = (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;
    setSelectedModForConfig(mod);
    setShowConfigEditor(true);
  };

  const handleOpenBackupManager = (modId: string) => {
    const mod = mods.find(m => m.unique_id === modId);
    if (!mod) return;
    setSelectedModForBackup(mod);
    setShowBackupManager(true);
  };

  const handleBackupRestore = () => {
    handleRefresh();
  };

  const filteredMods = useMemo(() => {
    console.log('[filteredMods] Input mods count:', mods.length);
    console.log('[filteredMods] Input mods names:', mods.map(m => m.name));

    const subModIds = new Set<string>();
    mods.forEach(mod => {
      if (mod.is_group && mod.sub_mods) {
        mod.sub_mods.forEach(sm => subModIds.add(sm.unique_id));
      }
    });

    const result = mods.filter(mod => {
      if (subModIds.has(mod.unique_id)) {
        console.log('[filteredMods] Filtered out sub_mod already in group:', mod.name);
        return false;
      }

      if (searchText) {
        const lowerSearch = searchText.toLowerCase();
        const nameMatch = mod.name.toLowerCase().includes(lowerSearch);
        const authorMatch = mod.author.toLowerCase().includes(lowerSearch);
        const idMatch = mod.unique_id.toLowerCase().includes(lowerSearch);
        const tagMatch = (getTags(mod.unique_id) || []).some(tag =>
          tag.toLowerCase().includes(lowerSearch),
        );
        if (!nameMatch && !authorMatch && !idMatch && !tagMatch) {
          console.log('[filteredMods] Filtered out by search:', mod.name);
          return false;
        }
      }

      if (filterType === 'enabled' && !mod.enabled) {
        console.log('[filteredMods] Filtered out by enabled filter:', mod.name);
        return false;
      }
      if (filterType === 'disabled' && mod.enabled) {
        console.log('[filteredMods] Filtered out by disabled filter:', mod.name);
        return false;
      }

      if (categoryFilter !== 'all' && mod.category !== categoryFilter) {
        console.log('[filteredMods] Filtered out by category:', mod.name, 'cat=', mod.category, 'filter=', categoryFilter);
        return false;
      }

      if (statusFilter === 'hasUpdate' && !mod.has_update) {
        console.log('[filteredMods] Filtered out by hasUpdate:', mod.name);
        return false;
      }
      if (statusFilter === 'hasConflict' && !mod.has_conflict) {
        console.log('[filteredMods] Filtered out by hasConflict:', mod.name);
        return false;
      }
      if (statusFilter === 'uncategorized' && mod.category !== 'other') {
        console.log('[filteredMods] Filtered out by uncategorized:', mod.name, 'cat=', mod.category);
        return false;
      }

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
    
    console.log('[filteredMods] Output count:', result.length);
    console.log('[filteredMods] Output names:', result.map(m => m.name));
    return result;
  }, [mods, searchText, filterType, categoryFilter, statusFilter, sortBy, getTags]);

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
          onProfileExit={() => setActiveProfileName(null)}
          onManageProfiles={() => navigate('/profiles')}
        />

        <button
          className="svl-check-updates-btn"
          onClick={handleCheckAllUpdates}
          disabled={checkingUpdates || mods.length === 0}
        >
          {checkingUpdates ? t('app.modList.checkingUpdates') : t('app.modList.checkAllUpdates')}
        </button>

        <button
          className="svl-dep-resolver-btn"
          onClick={() => setShowDepResolver(true)}
          disabled={mods.length === 0}
        >
          {t('app.depResolver.scan')}
        </button>

        {showUpdatePanel && updateStatuses.filter(u => u.has_update).length > 0 && (() => {
          const updatable = updateStatuses.filter(u => u.has_update);
          const allSelected = updatable.every(u => selectedUpdateMods.has(u.unique_id));
          const selectedCount = updatable.filter(u => selectedUpdateMods.has(u.unique_id)).length;
          return (
          <div className="svl-update-panel">
            <div className="svl-update-panel-header">
              <span className="svl-update-panel-title">
                {t('app.modList.foundUpdates', { count: updatable.length })}
                {selectedCount > 0 && <span className="svl-update-selected-count"> ({selectedCount})</span>}
              </span>
              <div className="svl-update-panel-header-actions">
                <button className="svl-select-all-btn" onClick={allSelected ? deselectAllUpdateMods : selectAllUpdateMods}>
                  {allSelected ? t('app.modList.deselectAll') : t('app.modList.selectAll')}
                </button>
                <button className="svl-invert-selection-btn" onClick={invertUpdateModSelection}>
                  {t('app.modList.invertSelection')}
                </button>
                <button className="svl-close-update-panel-btn" onClick={() => { setShowUpdatePanel(false); setSelectedUpdateMods(new Set()); }}>✕</button>
              </div>
            </div>
            <div className="svl-update-list">
              {updatable.map((u) => (
                <div
                  key={u.unique_id}
                  className={`svl-update-item ${selectedUpdateMods.has(u.unique_id) ? 'svl-update-item-selected' : ''}`}
                  onClick={() => toggleUpdateModSelect(u.unique_id)}
                >
                  <input
                    type="checkbox"
                    className="svl-update-checkbox"
                    checked={selectedUpdateMods.has(u.unique_id)}
                    onChange={() => toggleUpdateModSelect(u.unique_id)}
                    onClick={e => e.stopPropagation()}
                  />
                  <span className="svl-update-item-name">{u.name}</span>
                  <span className="svl-update-item-version" title={u.changelog || undefined}>
                    {u.current_version} → {u.latest_version
                      || (u.update_source === 'SmapiList' && u.changelog
                          ? t('app.modList.smapiStatus')
                          : u.update_source === 'NexusApi'
                            ? t('app.modList.needsNexusCheck')
                            : '?')}
                  </span>
                  {u.nexus_mod_id ? (
                    <button
                      className="svl-download-update-btn"
                      onClick={(e) => { e.stopPropagation(); handleDownloadModUpdate(u.nexus_mod_id!, u.unique_id); }}
                      disabled={downloadingModIds.has(u.nexus_mod_id)}
                    >
                      {downloadingModIds.has(u.nexus_mod_id) ? t('app.modList.downloading') : t('app.modList.downloadUpdate')}
                    </button>
                  ) : u.download_url ? (
                    <button
                      className="svl-download-update-btn"
                      onClick={(e) => {
                        e.stopPropagation();
                        const match = u.download_url?.match(/mods\/(\d+)/);
                        if (match) {
                          handleDownloadModUpdate(match[1], u.unique_id);
                        } else {
                          message.warning(t('app.modList.cannotAutoDownload'));
                        }
                      }}
                    >
                      {t('app.modList.downloadUpdate')}
                    </button>
                  ) : null}
                </div>
              ))}
            </div>
            <div className="svl-update-panel-footer">
              <span className="svl-update-footer-hint">
                {selectedCount === 0
                  ? t('app.modList.selectModsHint')
                  : t('app.modList.selectedCount', { count: selectedCount, total: updatable.length })}
              </span>
              <button
                className="svl-batch-update-btn"
                onClick={handleBatchUpdate}
                disabled={selectedCount === 0}
              >
                {t('app.modList.batchUpdate', { count: selectedCount })}
              </button>
            </div>
          </div>
        );})()}

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
            <button
              className="svl-batch-toggle-btn svl-batch-enable-btn"
              onClick={handleEnableAllMods}
              disabled={gameRunning || mods.length === 0}
              title={t('app.modList.batchEnable')}
            >
              {t('app.modList.enableAll')}
            </button>
            <button
              className="svl-batch-toggle-btn svl-batch-disable-btn"
              onClick={handleDisableAllMods}
              disabled={gameRunning || mods.length === 0}
              title={t('app.modList.batchDisable')}
            >
              {t('app.modList.disableAll')}
            </button>
          </div>

        </div>
      </div>

      <div
        className="svl-game-status-bar"
        style={{
          backgroundImage: 'linear-gradient(135deg, rgba(26, 21, 16, 0.95) 0%, rgba(45, 36, 24, 0.9) 100%), url(/images/stardew-farm-screenshot.jpg)',
          backgroundSize: 'cover',
          backgroundPosition: 'center',
          backgroundRepeat: 'no-repeat',
        }}
      >
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
            <button className="svl-change-btn" onClick={handleChangeGamePath}>
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
            <img
              src="/images/stardew-hero.jpg"
              alt={t('app.altStardewValley')}
              style={{
                width: '320px',
                height: 'auto',
                borderRadius: '16px',
                marginBottom: '24px',
                boxShadow: '0 8px 32px rgba(0,0,0,0.4)',
                imageRendering: 'auto',
              }}
            />
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
                    onOpenConfigEditor={handleOpenConfigEditor}
                    onOpenBackupManager={handleOpenBackupManager}
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
                    onToggleMod={handleToggleMod}
                    onDeleteMod={handleDeleteMod}
                    onCheckUpdate={handleCheckUpdate}
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
        existingMods={mods}
        gamePath={gamePath}
      />

      <LoadOrderModal
        visible={showLoadOrder}
        onClose={() => setShowLoadOrder(false)}
        mods={mods}
        gamePath={gamePath}
        onOrderApplied={handleRefresh}
      />

      <ModConfigEditor
        visible={showConfigEditor}
        onClose={() => { setShowConfigEditor(false); setSelectedModForConfig(null); }}
        modPath={selectedModForConfig?.folder_path || ''}
        onConfigUpdated={handleRefresh}
      />

      {selectedModForBackup && (
        <ModBackupManager
          visible={showBackupManager}
          onClose={() => { setShowBackupManager(false); setSelectedModForBackup(null); }}
          modPath={selectedModForBackup.folder_path}
          modUniqueId={selectedModForBackup.unique_id}
          modName={selectedModForBackup.name}
          onRestore={handleBackupRestore}
        />
      )}

      <ModBackupConfirmModal
        visible={showBackupConfirm}
        modName={
          pendingBatchUpdate && batchBackupQueue.length > 0
            ? `${batchBackupQueue[0].name} (批量备份 ${batchBackupQueue.length} 个模组)`
            : backupTargetMod
              ? mods.find(m => m.unique_id === backupTargetMod.uniqueId)?.name || ''
              : ''
        }
        modUniqueId={
          pendingBatchUpdate && batchBackupQueue.length > 0
            ? batchBackupQueue[0].uniqueId
            : backupTargetMod?.uniqueId || ''
        }
        modVersion={
          pendingBatchUpdate && batchBackupQueue.length > 0
            ? batchBackupQueue[0].version
            : backupTargetMod
              ? mods.find(m => m.unique_id === backupTargetMod.uniqueId)?.version || ''
              : ''
        }
        _defaultBackupDir={gamePath ? `${gamePath}\\svl-backups` : ''}
        onCancel={() => {
          setShowBackupConfirm(false);
          setPendingDownloadNexusId(null);
          if (pendingBatchUpdate) {
            setBatchBackupQueue([]);
            setPendingBatchUpdate(null);
          }
          setBackupTargetMod(null);
        }}
        onConfirm={handleBackupConfirm}
        onSkipBackup={handleBackupSkip}
      />

      <GameMonitor
        visible={showGameMonitor}
        onClose={() => setShowGameMonitor(false)}
        totalMods={mods.length}
      />

      <SecurityScanner
        visible={showSecurityScanner}
        onClose={() => setShowSecurityScanner(false)}
        mods={mods}
      />

      <LogParser
        isOpen={showLogParser}
        onClose={() => setShowLogParser(false)}
        smapiInstalled={smapiInfo?.installed}
      />

      <DependencyResolver
        open={showDepResolver}
        onClose={() => setShowDepResolver(false)}
        modsPath={modsPath}
        apiKey={localStorage.getItem('svl-nexus-api-key') || ''}
        onInstallComplete={() => handleRefresh()}
      />

    </>
  );
}
