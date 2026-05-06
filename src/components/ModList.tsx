import { useMemo, useState, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import type { ModInfo } from '../utils/tauri-api';
import { Tag, Tooltip, Modal, Dropdown, message } from 'antd';
import { FolderOpenOutlined, LinkOutlined, DeleteOutlined, SyncOutlined, CheckOutlined, CloseOutlined } from '@ant-design/icons';
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

const categoryColors: Record<string, string> = {
  visual: 'blue',
  gameplay: 'green',
  expansion: 'purple',
  framework: 'orange',
  ui: 'cyan',
  seasonal: 'gold',
  multiplayer: 'magenta',
  other: 'default',
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
  onAddTag?: (uniqueId: string, tag: string) => void;
  onRemoveTag?: (uniqueId: string, tag: string) => void;
  getTags?: (uniqueId: string) => string[];
}

const ICON_COLORS = [
  'linear-gradient(135deg, var(--svl-warning) 0%, #d97706 100%)',
  'linear-gradient(135deg, var(--svl-error) 0%, #dc2626 100%)',
  'linear-gradient(135deg, #f97316 0%, #ea580c 100%)',
  'linear-gradient(135deg, #eab308 0%, #ca8a04 100%)',
  'linear-gradient(135deg, #84cc16 0%, #65a30d 100%)',
  'linear-gradient(135deg, var(--svl-success) 0%, #16a34a 100%)',
  'linear-gradient(135deg, #14b8a6 0%, #0d9488 100%)',
  'linear-gradient(135deg, #06b6d4 0%, #0891b2 100%)',
];

function getIconColor(uniqueId: string): string {
  let hash = 0;
  for (let i = 0; i < uniqueId.length; i++) {
    hash = uniqueId.charCodeAt(i) + ((hash << 5) - hash);
  }
  return ICON_COLORS[Math.abs(hash) % ICON_COLORS.length];
}

function getModStatus(mod: ModInfo): { icon: string; label: string; color: string } {
  if (mod.has_conflict) {
    return { icon: '❌', label: 'missingDeps', color: 'error' };
  }
  if (mod.has_update) {
    return { icon: '🔄', label: 'updateAvailable', color: 'warning' };
  }
  if (mod.enabled) {
    return { icon: '✅', label: 'enabled', color: 'success' };
  }
  return { icon: '⚠️', label: 'disabled', color: 'default' };
}

export default function ModList({
  mods,
  loading,
  onToggleMod,
  onDeleteMod,
  onSelectMod,
  onOpenModFolder,
  onCheckUpdate,
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
  const parentRef = useRef<HTMLDivElement>(null);

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
                  style={{ background: getIconColor(mod.unique_id) }}
                >
                  {mod.thumbnail_path ? (
                    <img src={`file:///${mod.thumbnail_path.replace(/\\/g, '/')}`} alt={mod.name} />
                  ) : mod.screenshot_path ? (
                    <img src={`file:///${mod.screenshot_path.replace(/\\/g, '/')}`} alt={mod.name} />
                  ) : (
                    '📦'
                  )}
                </div>

                <div className="svl-mod-info">
                  <div className="svl-mod-name">
                    {mod.name}
                    {mod.is_group && mod.sub_mods.length > 0 && (
                      <Tag color="purple" style={{ marginLeft: 6, fontSize: 11 }}>
                        {mod.sub_mods.length}{t('app.modList.subMods')}
                      </Tag>
                    )}
                    <Tooltip title={t(`app.modStatus.${status.label}`)}>
                      <Tag color={status.color} className="svl-status-badge">
                        {status.icon}
                      </Tag>
                    </Tooltip>
                    {mod.has_update && (
                      <Tag color="red" className="svl-update-badge">
                        {t('app.modDetail.updateAvailable')}
                      </Tag>
                    )}
                  </div>
                  <div className="svl-mod-version">v{mod.version}</div>
                  <div className="svl-mod-author">
                    {t('app.modCard.by')} {mod.author}
                  </div>
                  <div className="svl-mod-category">
                    <Tag color={categoryColors[mod.category] || 'default'}>
                      {t(`app.categories.${mod.category}`)}
                    </Tag>
                    {modTags.map((tag) => (
                      <Tag
                        key={tag}
                        color="geekblue"
                        closable
                        onClose={(e) => {
                          e.stopPropagation();
                          onRemoveTag?.(mod.unique_id, tag);
                        }}
                        className="svl-custom-tag"
                      >
                        {tag}
                      </Tag>
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
                      <Tag
                        className="svl-add-tag-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          setTagInputId(mod.unique_id);
                          setTagInputValue('');
                        }}
                      >
                        +
                      </Tag>
                    )}
                  </div>
                </div>

                <div className="svl-mod-actions">
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
