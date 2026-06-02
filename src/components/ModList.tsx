import { useMemo, useState, useRef, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import type { ModInfo } from '../utils/tauri-api';
import type { ModNameTranslation } from '../utils/tauri-api';
import { getModNameTranslations, translateModName, batchTranslateModNames, deleteModNameTranslation, clearAllModNameTranslations } from '../utils/tauri-api';
import { Tooltip, Modal, Dropdown, message } from 'antd';
import { FolderOpenOutlined, LinkOutlined, DeleteOutlined, SyncOutlined, CheckOutlined, CloseOutlined, SettingOutlined, HistoryOutlined, TranslationOutlined } from '@ant-design/icons';
import { useVirtualizer } from '@tanstack/react-virtual';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '../utils/openUrl';

interface NexusLinkResult {
  url: string;
  method: string;
  mod_id: string | null;
}

type FilterType = 'all' | 'enabled' | 'disabled';
type SortType = 'name-az' | 'name-za' | 'author' | 'version';
type StatusFilter = 'all' | 'hasUpdate' | 'hasConflict' | 'uncategorized';

const categoryClassNames: Record<string, string> = {
  visual: 'svl-cat-visual',
  gameplay: 'svl-cat-gameplay',
  expansion: 'svl-cat-expansion',
  framework: 'svl-cat-framework',
  ui: 'svl-cat-ui',
  seasonal: 'svl-cat-seasonal',
  multiplayer: 'svl-cat-multiplayer',
  other: 'svl-cat-other',
};

interface ModListProps {
  mods: ModInfo[];
  loading: boolean;
  onRefresh: () => void;
  onUninstall: (modId: string) => void;
  searchText: string;
  filterType: FilterType;
  sortBy: SortType;
  statusFilter: StatusFilter;
  onToggleMod?: (modId: string) => void;
  onDeleteMod?: (modId: string) => void;
  onSelectMod?: (mod: ModInfo) => void;
  onOpenModFolder?: (modId: string) => void;
  onCheckUpdate?: (modId: string) => void;
  onOpenConfigEditor?: (modId: string) => void;
  onOpenBackupManager?: (modId: string) => void;
  onAddTag?: (uniqueId: string, tag: string) => void;
  onRemoveTag?: (uniqueId: string, tag: string) => void;
  getTags?: (uniqueId: string) => string[];
}


function getModStatus(mod: ModInfo): { icon: string; label: string; className: string } {
  if (mod.has_conflict) {
    return { icon: '❌', label: 'missingDeps', className: 'svl-tag-error' };
  }
  if (mod.has_update) {
    return { icon: '🔄', label: 'updateAvailable', className: 'svl-tag-warning' };
  }
  if (mod.enabled) {
    return { icon: '✅', label: 'enabled', className: 'svl-tag-success' };
  }
  return { icon: '⚠️', label: 'disabled', className: 'svl-tag-default' };
}

export default function ModList({
  mods,
  loading,
  onRefresh,
  onToggleMod,
  onDeleteMod,
  onSelectMod,
  onOpenModFolder,
  onCheckUpdate,
  onOpenConfigEditor,
  onOpenBackupManager,
  onAddTag,
  onRemoveTag,
  getTags,
}: ModListProps) {
  const { t } = useTranslation();
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [contextMenuMod, setContextMenuMod] = useState<ModInfo | null>(null);
  const [linkLoadingId, setLinkLoadingId] = useState<string | null>(null);
  const [tagInputId, setTagInputId] = useState<string | null>(null);
  const [tagInputValue, setTagInputValue] = useState('');
  const [nameTranslations, setNameTranslations] = useState<Map<string, ModNameTranslation>>(new Map());
  const [translatingId, setTranslatingId] = useState<string | null>(null);
  const [batchTranslating, setBatchTranslating] = useState(false);
  const parentRef = useRef<HTMLDivElement>(null);

  const isModTranslated = useCallback((mod: ModInfo) => {
    const existing = nameTranslations.get(mod.unique_id);
    if (existing && existing.translated_name !== existing.original_name) {
      return true;
    }
    const hasNonAscii = mod.name.split('').some(c => c.charCodeAt(0) > 127);
    if (!hasNonAscii) return false;
    const match = mod.name.match(/\(([^)]+)\)\s*$/);
    if (match && match[1].split('').every(c => c.charCodeAt(0) <= 127)) {
      return true;
    }
    return false;
  }, [nameTranslations]);

  useEffect(() => {
    getModNameTranslations().then(list => {
      const map = new Map<string, ModNameTranslation>();
      list.forEach(t => map.set(t.unique_id, t));
      setNameTranslations(map);
    }).catch(() => {});
  }, []);

  const filteredMods = useMemo(() => {
    return mods;
  }, [mods]);

  const virtualizer = useVirtualizer({
    count: filteredMods.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 80,
    overscan: 5,
  });

  const handleContextMenu = useCallback((mod: ModInfo, e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenuMod(mod);
  }, []);

  const handleSelectMod = useCallback((mod: ModInfo, e: React.MouseEvent) => {
    if (e.ctrlKey || e.metaKey) {
      setSelectedMods(prev => {
        const next = new Set(prev);
        if (next.has(mod.unique_id)) {
          next.delete(mod.unique_id);
        } else {
          next.add(mod.unique_id);
        }
        return next;
      });
    } else {
      onSelectMod?.(mod);
    }
  }, [onSelectMod]);

  const handleBatchEnable = useCallback(async () => {
    for (const modId of selectedMods) {
      await onToggleMod?.(modId);
    }
    message.success(t('app.modList.batchEnableSuccess', { count: selectedMods.size }));
    setSelectedMods(new Set());
  }, [selectedMods, onToggleMod, t]);

  const handleBatchDisable = useCallback(async () => {
    for (const modId of selectedMods) {
      await onToggleMod?.(modId);
    }
    message.success(t('app.modList.batchDisableSuccess', { count: selectedMods.size }));
    setSelectedMods(new Set());
  }, [selectedMods, onToggleMod, t]);

  const handleBatchDelete = useCallback(() => {
    Modal.confirm({
      title: t('app.modList.batchDeleteConfirm', { count: selectedMods.size }),
      content: t('app.modList.batchDeleteWarning'),
      okText: t('app.common.delete'),
      cancelText: t('app.common.cancel'),
      okButtonProps: { danger: true },
      onOk: async () => {
        for (const modId of selectedMods) {
          await onDeleteMod?.(modId);
        }
        message.success(t('app.modList.batchDeleteSuccess', { count: selectedMods.size }));
        setSelectedMods(new Set());
      },
    });
  }, [selectedMods, onDeleteMod, t]);

  const handleOpenNexusLink = useCallback(async (mod: ModInfo) => {
    setLinkLoadingId(mod.unique_id);
    try {
      const result = await invoke<NexusLinkResult>('get_nexus_link', {
        uniqueId: mod.unique_id,
        modName: mod.name,
        nexusModId: mod.nexus_mod_id
      });
      await openUrl(result.url);
    } catch (err) {
      console.error('[ModList] Failed to open mod page:', err);
      message.error(t('app.modCard.openLinkFailed'));
    } finally {
      setLinkLoadingId(null);
    }
  }, [t]);

  const handleTranslateName = useCallback(async (mod: ModInfo) => {
    setTranslatingId(mod.unique_id);
    try {
      const result = await translateModName(mod.unique_id, mod.name, mod.folder_path);
      setNameTranslations(prev => {
        const next = new Map(prev);
        next.set(result.unique_id, result);
        return next;
      });
    } catch (err: any) {
      message.error(err?.toString() || t('app.modNameTranslate.failed'));
    } finally {
      setTranslatingId(null);
    }
  }, [t]);

  const handleBatchTranslateNames = useCallback(async () => {
    setBatchTranslating(true);
    try {
      const modsToTranslate: Array<[string, string, string]> = filteredMods
        .filter(mod => !isModTranslated(mod))
        .map(mod => [mod.unique_id, mod.name, mod.folder_path] as [string, string, string]);
      
      if (modsToTranslate.length === 0) {
        message.info(t('app.modNameTranslate.allTranslated'));
        return;
      }
      const results = await batchTranslateModNames(modsToTranslate);
      const newMap = new Map(nameTranslations);
      results.forEach(r => newMap.set(r.unique_id, r));
      setNameTranslations(newMap);
      message.success(t('app.modNameTranslate.batchSuccess', { count: results.length }));
    } catch (err: any) {
      message.error(err?.toString() || t('app.modNameTranslate.failed'));
    } finally {
      setBatchTranslating(false);
    }
  }, [filteredMods, nameTranslations, t]);

  const handleDeleteTranslation = useCallback(async (mod: ModInfo) => {
    try {
      await deleteModNameTranslation(mod.unique_id, mod.folder_path);
      setNameTranslations(prev => {
        const next = new Map(prev);
        next.delete(mod.unique_id);
        return next;
      });
      onRefresh();
    } catch {}
  }, [onRefresh]);

  const handleClearAllTranslations = useCallback(async () => {
    try {
      const modsToRestore: Array<[string, string]> = filteredMods
        .filter(mod => isModTranslated(mod))
        .map(mod => [mod.unique_id, mod.folder_path] as [string, string]);
      await clearAllModNameTranslations(modsToRestore);
      setNameTranslations(new Map());
      onRefresh();
      message.success(t('app.modNameTranslate.clearSuccess'));
    } catch (err: any) {
      message.error(err?.toString() || t('app.modNameTranslate.failed'));
    }
  }, [filteredMods, isModTranslated, t, onRefresh]);

  const displayNameMap = useMemo(() => {
    const map = new Map<string, string>();
    for (const mod of filteredMods) {
      const translation = nameTranslations.get(mod.unique_id);
      if (translation && translation.translated_name !== translation.original_name) {
        map.set(mod.unique_id, `${translation.translated_name} (${translation.original_name})`);
      } else if (isModTranslated(mod)) {
        map.set(mod.unique_id, mod.name);
      } else {
        map.set(mod.unique_id, mod.name);
      }
    }
    return map;
  }, [filteredMods, nameTranslations, isModTranslated]);

  const getDisplayName = useCallback((mod: ModInfo) => {
    return displayNameMap.get(mod.unique_id) || mod.name;
  }, [displayNameMap]);

  const contextMenuItems = contextMenuMod ? [
    {
      key: 'toggle',
      icon: contextMenuMod.enabled ? <CloseOutlined /> : <CheckOutlined />,
      label: contextMenuMod.is_required 
        ? (contextMenuMod.enabled ? t('app.modCard.requiredMod') : t('app.modCard.enable'))
        : (contextMenuMod.enabled ? t('app.modCard.disable') : t('app.modCard.enable')),
      disabled: contextMenuMod.is_required && contextMenuMod.enabled,
      onClick: () => {
        if (!contextMenuMod.is_required || !contextMenuMod.enabled) {
          onToggleMod?.(contextMenuMod.unique_id);
        }
        setContextMenuMod(null);
      },
    },
    {
      key: 'openPage',
      icon: <LinkOutlined />,
      label: t('app.modCard.openPage'),
      onClick: () => {
        onSelectMod?.(contextMenuMod);
        setContextMenuMod(null);
      },
    },
    {
      key: 'openFolder',
      icon: <FolderOpenOutlined />,
      label: t('app.modCard.openFolder'),
      onClick: () => {
        onOpenModFolder?.(contextMenuMod.unique_id);
        setContextMenuMod(null);
      },
    },
    {
      key: 'checkUpdate',
      icon: <SyncOutlined />,
      label: t('app.modCard.checkUpdate'),
      onClick: () => {
        onCheckUpdate?.(contextMenuMod.unique_id);
        setContextMenuMod(null);
      },
    },
    { type: 'divider' as const },
    {
      key: 'configEditor',
      icon: <SettingOutlined />,
      label: t('features.configEditor.title'),
      onClick: () => {
        onOpenConfigEditor?.(contextMenuMod.unique_id);
        setContextMenuMod(null);
      },
    },
    {
      key: 'backupManager',
      icon: <HistoryOutlined />,
      label: t('features.backupManager.titleShort'),
      onClick: () => {
        onOpenBackupManager?.(contextMenuMod.unique_id);
        setContextMenuMod(null);
      },
    },
    { type: 'divider' as const },
    {
      key: 'translateName',
      icon: <TranslationOutlined />,
      label: isModTranslated(contextMenuMod)
        ? t('app.modNameTranslate.restoreName')
        : t('app.modNameTranslate.translateName'),
      disabled: translatingId === contextMenuMod.unique_id,
      onClick: () => {
        if (isModTranslated(contextMenuMod)) {
          handleDeleteTranslation(contextMenuMod);
        } else {
          handleTranslateName(contextMenuMod);
        }
        setContextMenuMod(null);
      },
    },
    { type: 'divider' as const },
    {
      key: 'delete',
      icon: <DeleteOutlined />,
      label: t('app.modCard.uninstall'),
      danger: true,
      onClick: () => {
        onDeleteMod?.(contextMenuMod.unique_id);
        setContextMenuMod(null);
      },
    },
  ] : [];

  if (loading) {
    return (
      <div className="svl-empty-state">
        <div className="svl-empty-icon"></div>
        <div className="svl-empty-title">{t('app.pages.modManager.refreshing')}</div>
      </div>
    );
  }

  if (filteredMods.length === 0 && mods.length === 0) {
    return (
      <div className="svl-empty-state">
        <div className="svl-empty-icon">📂</div>
        <div className="svl-empty-title">{t('app.pages.modManager.noModsFound')}</div>
        <div className="svl-empty-desc">{t('app.pages.modManager.noModsDesc')}</div>
      </div>
    );
  }

  return (
    <>
      <div style={{ display: 'flex', gap: 8, padding: '4px 8px', borderBottom: '1px solid rgba(255,255,255,0.06)' }}>
        <button
          className="svl-batch-btn"
          onClick={handleBatchTranslateNames}
          disabled={batchTranslating}
          style={{ display: 'flex', alignItems: 'center', gap: 4 }}
        >
          <TranslationOutlined spin={batchTranslating} />
          {batchTranslating ? t('app.modNameTranslate.translating') : t('app.modNameTranslate.batchTranslate')}
        </button>
        {nameTranslations.size > 0 && (
          <button
            className="svl-batch-btn svl-batch-btn-danger"
            onClick={handleClearAllTranslations}
            style={{ display: 'flex', alignItems: 'center', gap: 4 }}
          >
            {t('app.modNameTranslate.clearAll')}
          </button>
        )}
      </div>

      {selectedMods.size > 0 && (
        <div className="svl-batch-actions">
          <span className="svl-batch-count">
            {t('app.modList.selected', { count: selectedMods.size })}
          </span>
          <button className="svl-batch-btn" onClick={handleBatchEnable}>
            {t('app.modCard.enable')}
          </button>
          <button className="svl-batch-btn" onClick={handleBatchDisable}>
            {t('app.modCard.disable')}
          </button>
          <button className="svl-batch-btn svl-batch-btn-danger" onClick={handleBatchDelete}>
            {t('app.modCard.uninstall')}
          </button>
          <button className="svl-batch-btn" onClick={() => setSelectedMods(new Set())}>
            {t('app.common.cancel')}
          </button>
        </div>
      )}

      <div ref={parentRef} className="svl-mods-list" style={{ overflow: 'auto', height: '100%' }}>
        <div
          style={{
            height: `${virtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const mod = filteredMods[virtualRow.index];
            const status = getModStatus(mod);
            const isSelected = selectedMods.has(mod.unique_id);
            const modTags = getTags?.(mod.unique_id) || [];

            return (
              <div
                key={mod.unique_id}
                ref={virtualizer.measureElement}
                data-index={virtualRow.index}
                className={`svl-mod-card ${mod.has_conflict ? 'has-conflict' : ''} ${isSelected ? 'selected' : ''}`}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                onClick={(e) => handleSelectMod(mod, e)}
                onContextMenu={(e) => handleContextMenu(mod, e)}
              >
                <div
                  className="svl-mod-icon"
                >
                  {mod.thumbnail_path ? (
                    <img src={`file:///${mod.thumbnail_path.replace(/\\/g, '/')}`} alt={mod.name} width={40} height={40} style={{ objectFit: 'cover' }} />
                  ) : mod.screenshot_path ? (
                    <img src={`file:///${mod.screenshot_path.replace(/\\/g, '/')}`} alt={mod.name} width={40} height={40} style={{ objectFit: 'cover' }} />
                  ) : (
                    <img
                      src="/mod-icon.png"
                      alt=""
                      width={40}
                      height={40}
                      style={{ objectFit: 'contain' }}
                    />
                  )}
                </div>

                <div className="svl-mod-info">
                  <div className="svl-mod-name">
                    <span>{getDisplayName(mod)}</span>
                    {mod.is_group && mod.sub_mods.length > 0 && (
                      <span className="svl-tag-accent" style={{ marginLeft: 6, fontSize: 11 }}>
                        {mod.sub_mods.length}{t('app.modList.subMods')}
                      </span>
                    )}
                    <Tooltip title={t(`app.modStatus.${status.label}`)}>
                      <span className={`${status.className} svl-status-badge`}>
                        {status.icon}
                      </span>
                    </Tooltip>
                    {mod.has_update && (
                      <span className="svl-tag-danger svl-update-badge">
                        {t('app.modDetail.updateAvailable')}
                      </span>
                    )}
                  </div>
                  <div className="svl-mod-version">v{mod.version}</div>
                  <div className="svl-mod-author">
                    {t('app.modCard.by')} {mod.author}
                  </div>
                  <div className="svl-mod-category">
                    <span className={categoryClassNames[mod.category] || 'svl-cat-other'}>
                      {t(`app.categories.${mod.category}`)}
                    </span>
                    {modTags.map((tag) => (
                      <span
                        key={tag}
                        className="svl-tag-info svl-custom-tag"
                      >
                        {tag}
                        <span
                          className="svl-tag-close"
                          onClick={(e) => {
                            e.stopPropagation();
                            onRemoveTag?.(mod.unique_id, tag);
                          }}
                        >
                          ×
                        </span>
                      </span>
                    ))}
                    {tagInputId === mod.unique_id ? (
                      <input
                        className="svl-tag-input"
                        value={tagInputValue}
                        onChange={(e) => setTagInputValue(e.target.value)}
                        onKeyDown={(e) => {
                          e.stopPropagation();
                          if (e.key === 'Enter' && tagInputValue.trim()) {
                            onAddTag?.(mod.unique_id, tagInputValue.trim());
                            setTagInputValue('');
                            setTagInputId(null);
                          } else if (e.key === 'Escape') {
                            setTagInputValue('');
                            setTagInputId(null);
                          }
                        }}
                        onBlur={() => {
                          if (tagInputValue.trim()) {
                            onAddTag?.(mod.unique_id, tagInputValue.trim());
                          }
                          setTagInputValue('');
                          setTagInputId(null);
                        }}
                        autoFocus
                        placeholder={t('app.tags.inputPlaceholder')}
                      />
                    ) : (
                      <span
                        className="svl-add-tag-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          setTagInputId(mod.unique_id);
                          setTagInputValue('');
                        }}
                      >
                        +
                      </span>
                    )}
                  </div>
                </div>

                <div className="svl-mod-actions">
                  <Tooltip title={isModTranslated(mod)
                    ? t('app.modNameTranslate.restoreName')
                    : t('app.modNameTranslate.translateName')}>
                    <button
                      className="svl-link-btn"
                      onClick={(e) => {
                        e.stopPropagation();
                        if (isModTranslated(mod)) {
                          handleDeleteTranslation(mod);
                        } else {
                          handleTranslateName(mod);
                        }
                      }}
                      disabled={translatingId === mod.unique_id}
                      style={{ fontSize: 14, opacity: translatingId === mod.unique_id ? 0.5 : 1 }}
                    >
                      {translatingId === mod.unique_id ? '⏳' : '🌐'}
                    </button>
                  </Tooltip>
                  <Tooltip title={t('app.modCard.openPage')}>
                    <button
                      className="svl-link-btn"
                      onClick={(e) => { e.stopPropagation(); handleOpenNexusLink(mod); }}
                      disabled={linkLoadingId === mod.unique_id}
                      title={t('app.modCard.openPage')}
                    >
                      {linkLoadingId === mod.unique_id ? '⏳' : '🔗'}
                    </button>
                  </Tooltip>
                  <button
                    className="svl-uninstall-btn"
                    onClick={(e) => { e.stopPropagation(); onDeleteMod?.(mod.unique_id); }}
                    title={t('app.modCard.uninstall')}
                  >
                    🗑️
                  </button>
                  <Tooltip title={mod.is_required && mod.enabled ? t('app.modCard.requiredMod') : ''}>
                    <div
                      className={`svl-switch ${mod.enabled ? 'active' : ''} ${mod.is_required ? 'required' : ''}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        if (!mod.is_required || !mod.enabled) {
                          onToggleMod?.(mod.unique_id);
                        }
                      }}
                    >
                      <div className="svl-switch-thumb" />
                    </div>
                  </Tooltip>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      <Dropdown
        menu={{ items: contextMenuItems }}
        trigger={['contextMenu']}
        open={!!contextMenuMod}
        onOpenChange={(open) => !open && setContextMenuMod(null)}
        overlayClassName="svl-context-menu"
      >
        <div style={{ display: 'none' }} />
      </Dropdown>
    </>
  );
}
