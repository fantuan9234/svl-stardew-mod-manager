import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Tag, Button, Tabs, Tooltip, message } from 'antd';
import { CloseOutlined, LinkOutlined, LoadingOutlined, FolderOpenOutlined, HeartOutlined, SyncOutlined, DeleteOutlined } from '@ant-design/icons';
import type { ModInfo } from '../utils/tauri-api';
import { openUrl } from '../utils/openUrl';
import { useModUrl } from '../hooks/useModUrl';
import { invoke } from '@tauri-apps/api/core';
import { revealItemInDir } from '@tauri-apps/plugin-opener';
import { useImageUrl } from '../hooks/useImageUrl';

interface ModDetailProps {
  mod: ModInfo;
  installedMods: ModInfo[];
  onClose: () => void;
  onToggleMod?: (modId: string) => void;
  onDeleteMod?: (modId: string) => void;
  onCheckUpdate?: (modId: string) => void;
  onEndorse?: (modId: string) => void;
  onAddTag?: (uniqueId: string, tag: string) => void;
  onRemoveTag?: (uniqueId: string, tag: string) => void;
  getTags?: (uniqueId: string) => string[];
  allTags?: string[];
}

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

const categoryLabels: Record<string, string> = {
  visual: 'app.categories.visual',
  gameplay: 'app.categories.gameplay',
  expansion: 'app.categories.expansion',
  framework: 'app.categories.framework',
  ui: 'app.categories.ui',
  seasonal: 'app.categories.seasonal',
  multiplayer: 'app.categories.multiplayer',
  other: 'app.categories.other',
};

export default function ModDetail({ mod, installedMods, onClose, onDeleteMod, onCheckUpdate, onEndorse, onAddTag, onRemoveTag, getTags, allTags }: ModDetailProps) {
  const { t } = useTranslation();
  const { url: resolvedUrl, isLoading, resolve } = useModUrl();
  const [endorseLoading, setEndorseLoading] = useState(false);
  const [updateLoading, setUpdateLoading] = useState(false);
  const [tagInputValue, setTagInputValue] = useState('');
  const detailImageUrl = useImageUrl(mod.thumbnail_path || mod.screenshot_path);
  useEffect(() => {
    const validModUrl = mod.url && !mod.url.includes('/search?');
    console.log('[ModDetail] mod:', mod.name, 'url:', mod.url, 'nexus_mod_id:', mod.nexus_mod_id, 'valid:', validModUrl);
    if (validModUrl) {
      return;
    }
    resolve(mod.unique_id || mod.name, mod.name, mod.nexus_mod_id);
  }, [mod.unique_id, mod.name, mod.nexus_mod_id]);

  const handleOpenLink = async () => {
    const validModUrl = mod.url && !mod.url.includes('/search?');
    const targetUrl = validModUrl ? mod.url : (resolvedUrl || mod.url);
    if (targetUrl) {
      await openUrl(targetUrl, t('app.modCard.openLinkFailed'));
    }
  };

  const handleOpenFolder = async () => {
    try {
      await revealItemInDir(mod.folder_path);
    } catch (err) {
      console.error('[handleOpenFolder] failed:', err);
      message.error(t('app.smapiInstaller.openPathFailed'));
    }
  };

  const handleEndorse = async () => {
    if (!mod.nexus_id) {
      message.warning(t('app.modDetail.noNexusId'));
      return;
    }
    try {
      setEndorseLoading(true);
      await invoke('endorse_mod', { gameId: 'stardewvalley', modId: mod.nexus_id });
      message.success(t('app.modDetail.endorseSuccess'));
      onEndorse?.(mod.unique_id);
    } catch (err) {
      console.error('[handleEndorse] failed:', err);
      message.error(t('app.modDetail.endorseFailed'));
    } finally {
      setEndorseLoading(false);
    }
  };

  const handleCheckUpdate = async () => {
    try {
      setUpdateLoading(true);
      const updates = await invoke('check_mod_updates', { uniqueId: mod.unique_id });
      if (updates && (updates as any).has_update) {
        message.info(t('app.modDetail.updateAvailable'));
      } else {
        message.success(t('app.modCard.upToDate'));
      }
      onCheckUpdate?.(mod.unique_id);
    } catch (err) {
      console.error('[handleCheckUpdate] failed:', err);
      message.error(t('app.modCard.checkUpdateFailed'));
    } finally {
      setUpdateLoading(false);
    }
  };

  const handleDeleteMod = () => {
    onDeleteMod?.(mod.unique_id);
  };

  const getCategoryClassName = () => categoryClassNames[mod.category] || 'svl-cat-other';
  const getCategoryLabel = () => categoryLabels[mod.category] || 'app.categories.other';

  const installedIds = new Set(installedMods.map(m => m.unique_id.toLowerCase()));

  const depsWithStatus = mod.dependencies.map(dep => ({
    ...dep,
    isInstalled: installedIds.has(dep.unique_id.toLowerCase()),
  }));

  const requiredDeps = depsWithStatus.filter(d => d.is_required);

  const missingRequiredCount = requiredDeps.filter(d => !d.isInstalled).length;

  const tabItems = [
    {
      key: 'basic',
      label: t('app.modDetail.basicInfo'),
      children: (
        <div className="svl-mod-detail-meta">
          <div className="svl-mod-meta-row">
            <span className="svl-meta-label">{t('app.modDetail.name')}</span>
            <span className="svl-meta-value">{mod.name}</span>
          </div>
          <div className="svl-mod-meta-row">
            <span className="svl-meta-label">{t('app.modDetail.version')}</span>
            <span className="svl-meta-value">{mod.version}</span>
          </div>
          <div className="svl-mod-meta-row">
            <span className="svl-meta-label">{t('app.modDetail.author')}</span>
            <span className="svl-meta-value">{mod.author}</span>
          </div>
          <div className="svl-mod-meta-row">
            <span className="svl-meta-label">{t('app.modDetail.uniqueId')}</span>
            <span className="svl-meta-value svl-meta-code">{mod.unique_id}</span>
          </div>
          <div className="svl-mod-meta-row">
            <span className="svl-meta-label">{t('app.modDetail.category')}</span>
            <span className="svl-meta-value">
              <Tag className={getCategoryClassName()}>{t(getCategoryLabel())}</Tag>
            </span>
          </div>
          {mod.nexus_id && (
            <div className="svl-mod-meta-row">
              <span className="svl-meta-label">{t('app.modDetail.nexusId')}</span>
              <span className="svl-meta-value svl-meta-code">{mod.nexus_id}</span>
            </div>
          )}
          <div className="svl-mod-meta-row svl-meta-description">
            <span className="svl-meta-label">{t('app.modDetail.description')}</span>
            <div className="svl-mod-description">
              {mod.description || t('app.modDetail.noDescription')}
            </div>
          </div>
          <div className="svl-mod-meta-row">
            <span className="svl-meta-label">{t('app.tags.label')}</span>
            <div className="svl-tags-container">
              {(getTags?.(mod.unique_id) || []).map((tag) => (
                <Tag
                  key={tag}
                  className="svl-tag-info svl-custom-tag"
                  closable
                  onClose={() => onRemoveTag?.(mod.unique_id, tag)}
                >
                  {tag}
                </Tag>
              ))}
              <div className="svl-tag-add-wrapper">
                <input
                  className="svl-tag-input"
                  value={tagInputValue}
                  onChange={(e) => setTagInputValue(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && tagInputValue.trim()) {
                      onAddTag?.(mod.unique_id, tagInputValue.trim());
                      setTagInputValue('');
                    } else if (e.key === 'Escape') {
                      setTagInputValue('');
                    }
                  }}
                  placeholder={t('app.tags.inputPlaceholder')}
                />
                {tagInputValue.trim() && (
                  <div className="svl-tag-suggestions">
                    {allTags
                      ?.filter(
                        (t) =>
                          t.toLowerCase().includes(tagInputValue.toLowerCase()) &&
                          !(getTags?.(mod.unique_id) || []).includes(t),
                      )
                      .slice(0, 5)
                      .map((suggestion) => (
                        <div
                          key={suggestion}
                          className="svl-tag-suggestion-item"
                          onClick={() => {
                            onAddTag?.(mod.unique_id, suggestion);
                            setTagInputValue('');
                          }}
                        >
                          {suggestion}
                        </div>
                      ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      ),
    },
    {
      key: 'dependencies',
      label: t('app.modDetail.dependencies'),
      children: (
        <div className="svl-deps-container">
          {requiredDeps.length > 0 && (
            <div className="svl-deps-section">
              <h4 className="svl-deps-section-title">
                {t('app.modDetail.requiredDeps')}
                {missingRequiredCount > 0 && (
                  <Tag className="svl-tag-error">{t('app.modDetail.missingCount', { count: missingRequiredCount })}</Tag>
                )}
              </h4>
              <div className="svl-deps-list">
                {requiredDeps.map((dep) => (
                  <div key={dep.unique_id} className={`svl-dep-item ${dep.isInstalled ? 'installed' : 'missing'}`}>
                    <span className="svl-dep-status">
                      {dep.isInstalled ? '✅' : '❌'}
                    </span>
                    <span className="svl-dep-id">{dep.unique_id}</span>
                    {dep.minimum_version && (
                      <Tag>{dep.minimum_version}+</Tag>
                    )}
                    <span className="svl-dep-name">
                      {dep.isInstalled ? t('app.modDetail.installed') : t('app.modDetail.missing')}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {mod.dependencies.length === 0 && (
            <div className="svl-no-deps">{t('app.modDetail.noDependencies')}</div>
          )}
        </div>
      ),
    },
    {
      key: 'changelog',
      label: t('app.modDetail.changelog'),
      children: (
        <div className="svl-changelog">
          {mod.update_notes ? (
            <div className="svl-changelog-content">
              <pre>{mod.update_notes}</pre>
            </div>
          ) : (
            <div className="svl-no-changelog">{t('app.modDetail.noChangelog')}</div>
          )}
        </div>
      ),
    },
  ];

  return (
    <div className="svl-mod-detail">
      <div className="svl-mod-detail-header">
        <div className="svl-mod-detail-title">
          <h2>{mod.name}</h2>
          <Tag className={getCategoryClassName()}>{t(getCategoryLabel())}</Tag>
          {mod.has_update && (
            <Tag className="svl-tag-danger">{t('app.modDetail.updateAvailable')}</Tag>
          )}
          {mod.has_conflict && (
            <Tag className="svl-tag-error">{t('app.modCard.conflictDetected')}</Tag>
          )}
        </div>
        <Button
          type="text"
          icon={<CloseOutlined />}
          onClick={onClose}
          className="svl-mod-detail-close"
        />
      </div>

      {(mod.thumbnail_path || mod.screenshot_path) && detailImageUrl && (
        <div className="svl-mod-detail-screenshot">
          <img
            src={detailImageUrl}
            alt={mod.name}
          />
        </div>
      )}

      <div className="svl-mod-detail-actions">
        <Tooltip title={t('app.modCard.openPage')}>
          <Button
            icon={isLoading ? <LoadingOutlined /> : <LinkOutlined />}
            onClick={handleOpenLink}
            disabled={isLoading || !(resolvedUrl || mod.url)}
          >
            {t('app.modCard.openPage')}
          </Button>
        </Tooltip>
        <Tooltip title={t('app.modCard.openFolder')}>
          <Button
            icon={<FolderOpenOutlined />}
            onClick={handleOpenFolder}
          >
            {t('app.modCard.openFolder')}
          </Button>
        </Tooltip>
        {mod.nexus_id && (
          <Tooltip title={t('app.modDetail.endorse')}>
            <Button
              icon={<HeartOutlined />}
              onClick={handleEndorse}
              loading={endorseLoading}
            >
              {t('app.modDetail.endorse')}
            </Button>
          </Tooltip>
        )}
        <Tooltip title={t('app.modCard.checkUpdate')}>
          <Button
            icon={<SyncOutlined />}
            onClick={handleCheckUpdate}
            loading={updateLoading}
          >
            {t('app.modCard.checkUpdate')}
          </Button>
        </Tooltip>
        <Tooltip title={t('app.modCard.uninstall')}>
          <Button
            danger
            icon={<DeleteOutlined />}
            onClick={handleDeleteMod}
          >
            {t('app.modCard.uninstall')}
          </Button>
        </Tooltip>
      </div>

      <Tabs
        defaultActiveKey="basic"
        items={tabItems}
        className="svl-mod-detail-tabs"
      />
    </div>
  );
}
