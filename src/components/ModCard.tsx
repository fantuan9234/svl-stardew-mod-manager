import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import type { ModInfo } from '../utils/tauri-api';
import { Tag, Tooltip } from 'antd';
import { LoadingOutlined } from '@ant-design/icons';
import { openUrl } from '../utils/openUrl';
import { useModUrl } from '../hooks/useModUrl';
import { useImageUrl } from '../hooks/useImageUrl';

interface ModCardProps {
  mod: ModInfo;
  icon: string;
  iconColor: string;
  isSelected?: boolean;
  onToggle: () => void;
  onUninstall: () => void;
  onSelect?: () => void;
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

export default function ModCard({ mod, icon, iconColor, isSelected, onToggle, onUninstall, onSelect }: ModCardProps) {
  const { t } = useTranslation();
  const [enabled, setEnabled] = useState(mod.enabled);
  const { url: resolvedUrl, isLoading, resolve } = useModUrl();
  const thumbnailUrl = useImageUrl(mod.thumbnail_path);
  const screenshotUrl = useImageUrl(mod.screenshot_path);

  useEffect(() => {
    setEnabled(mod.enabled);
  }, [mod.enabled]);

  useEffect(() => {
    resolve(mod.unique_id || mod.name, mod.name, mod.nexus_mod_id);
  }, [mod.unique_id, mod.name, mod.nexus_mod_id]);

  const handleToggle = () => {
    onToggle();
  };

  const handleOpenLink = async () => {
    const targetUrl = resolvedUrl || mod.url;
    if (targetUrl) {
      await openUrl(targetUrl, t('app.modCard.openLinkFailed'));
    }
  };

  return (
    <div className={`svl-mod-card ${mod.has_conflict ? 'has-conflict' : ''} ${isSelected ? 'selected' : ''}`}>
      {mod.thumbnail_path && thumbnailUrl ? (
        <div className="svl-mod-screenshot" onClick={onSelect}>
          <img src={thumbnailUrl} alt={mod.name} />
        </div>
      ) : mod.screenshot_path && screenshotUrl ? (
        <div className="svl-mod-screenshot" onClick={onSelect}>
          <img src={screenshotUrl} alt={mod.name} />
        </div>
      ) : (
        <div
          className="svl-mod-icon"
          style={{ background: iconColor }}
          onClick={onSelect}
        >
          {icon}
        </div>
      )}

      <div className="svl-mod-info" onClick={onSelect}>
        <div className="svl-mod-name">
          {mod.name}
          {mod.has_conflict && (
            <span className="svl-conflict-badge" title={mod.conflict_warning || t('app.modCard.conflictDetected')}>
              ⚠️
            </span>
          )}
          {mod.has_update && (
            <Tag className="svl-tag-danger svl-update-badge">
              {t('app.modDetail.updateAvailable')}
            </Tag>
          )}
        </div>
        <div className="svl-mod-version">v{mod.version}</div>
        <div className="svl-mod-author">
          {t('app.modCard.by')} {mod.author}
        </div>
        <div className="svl-mod-category">
          <Tag className={categoryClassNames[mod.category] || 'svl-cat-other'}>
            {t(`app.categories.${mod.category}`, mod.category)}
          </Tag>
        </div>
      </div>

      <div className="svl-mod-actions">
        <Tooltip title={isLoading ? t('app.modCard.resolvingUrl') : t('app.modCard.openPage')}>
          <span
            className={`svl-mod-link ${(resolvedUrl || mod.url) ? '' : 'disabled'}`}
            style={{ cursor: (resolvedUrl || mod.url) ? 'pointer' : 'default', position: 'relative' }}
            onClick={handleOpenLink}
          >
            {isLoading ? (
              <LoadingOutlined style={{ fontSize: 14 }} />
            ) : (
              '🔗'
            )}
          </span>
        </Tooltip>
        <button
          className="svl-uninstall-btn"
          onClick={onUninstall}
          title={t('app.modCard.uninstall')}
        >
          🗑️
        </button>
        <div
          className={`svl-switch ${enabled ? 'active' : ''}`}
          onClick={handleToggle}
        >
          <div className="svl-switch-thumb" />
        </div>
      </div>
    </div>
  );
}
