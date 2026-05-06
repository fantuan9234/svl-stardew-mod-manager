import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  exportSyncEnvironment,
  importSyncEnvironment,
  applySyncEnvironment,
  openSaveDialog,
  openOpenDialog,
  type SyncDiff,
  type SyncApplyResult,
} from '../utils/tauri-api';

interface SyncManagerProps {
  isOpen: boolean;
  onClose: () => void;
  gamePath: string;
  mods: Array<{
    name: string;
    unique_id: string;
    version: string;
    author: string;
    enabled: boolean;
  }>;
}

export default function SyncManager({ isOpen, onClose, gamePath, mods }: SyncManagerProps) {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<'export' | 'import'>('export');
  const [hostName, setHostName] = useState('');
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState(false);
  const [applying, setApplying] = useState(false);
  const [syncDiff, setSyncDiff] = useState<SyncDiff | null>(null);
  const [applyResult, setApplyResult] = useState<SyncApplyResult | null>(null);
  const [syncFilePath, setSyncFilePath] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleExport = async () => {
    try {
      setExporting(true);
      setError(null);
      setSuccessMessage(null);

      if (!hostName.trim()) {
        setError(t('app.sync.export.hostNameRequired'));
        return;
      }

      const savePath = await openSaveDialog();

      if (!savePath) {
        setExporting(false);
        return;
      }

      const finalPath = savePath.endsWith('.svl_sync') ? savePath : `${savePath}.svl_sync`;
      
      await exportSyncEnvironment(gamePath, hostName, finalPath);
      setSuccessMessage(t('app.sync.export.success'));
    } catch (err: any) {
      setError(err?.message || t('app.sync.export.failed'));
    } finally {
      setExporting(false);
    }
  };

  const handleImport = async () => {
    try {
      setImporting(true);
      setError(null);
      setSuccessMessage(null);
      setSyncDiff(null);
      setApplyResult(null);

      const filePath = await openOpenDialog();

      if (!filePath) {
        setImporting(false);
        return;
      }

      setSyncFilePath(filePath);
      const diff = await importSyncEnvironment(filePath, gamePath);
      setSyncDiff(diff);
    } catch (err: any) {
      setError(err?.message || t('app.sync.import.failed'));
    } finally {
      setImporting(false);
    }
  };

  const handleApplySync = async () => {
    try {
      setApplying(true);
      setError(null);

      const result = await applySyncEnvironment(syncFilePath, gamePath);
      setApplyResult(result);
    } catch (err: any) {
      setError(err?.message || t('app.sync.apply.failed'));
    } finally {
      setApplying(false);
    }
  };

  const handleClose = () => {
    setSyncDiff(null);
    setApplyResult(null);
    setError(null);
    setSuccessMessage(null);
    setSyncFilePath('');
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="svl-sync-overlay" onClick={handleClose}>
      <div className="svl-sync-modal" onClick={(e) => e.stopPropagation()}>
        <div className="svl-sync-header">
          <h2>{t('app.sync.title')}</h2>
          <button className="svl-sync-close" onClick={handleClose}>×</button>
        </div>

        <div className="svl-sync-tabs">
          <button
            className={`svl-sync-tab ${activeTab === 'export' ? 'active' : ''}`}
            onClick={() => setActiveTab('export')}
          >
            {t('app.sync.export.tab')}
          </button>
          <button
            className={`svl-sync-tab ${activeTab === 'import' ? 'active' : ''}`}
            onClick={() => setActiveTab('import')}
          >
            {t('app.sync.import.tab')}
          </button>
        </div>

        <div className="svl-sync-content">
          {error && (
            <div className="svl-sync-error">
              <span className="svl-sync-error-icon">⚠️</span>
              <span>{error}</span>
            </div>
          )}

          {successMessage && (
            <div className="svl-sync-success">
              <span className="svl-sync-success-icon">✅</span>
              <span>{successMessage}</span>
            </div>
          )}

          {activeTab === 'export' && (
            <div className="svl-sync-export">
              <div className="svl-sync-export-info">
                <p>{t('app.sync.export.description')}</p>
              </div>

              <div className="svl-sync-form-group">
                <label>{t('app.sync.export.hostNameLabel')}</label>
                <input
                  type="text"
                  value={hostName}
                  onChange={(e) => setHostName(e.target.value)}
                  placeholder={t('app.sync.export.hostNamePlaceholder')}
                />
              </div>

              <div className="svl-sync-mod-list">
                <h3>{t('app.sync.export.modCount', { count: mods.length })}</h3>
                <div className="svl-sync-mod-list-scroll">
                  {mods.map((mod) => (
                    <div key={mod.unique_id} className="svl-sync-mod-item">
                      <span className="svl-sync-mod-name">{mod.name}</span>
                      <span className="svl-sync-mod-version">v{mod.version}</span>
                    </div>
                  ))}
                </div>
              </div>

              <button
                className="svl-sync-export-btn"
                onClick={handleExport}
                disabled={exporting || !hostName.trim()}
              >
                {exporting ? t('app.sync.export.exporting') : t('app.sync.export.exportButton')}
              </button>
            </div>
          )}

          {activeTab === 'import' && (
            <div className="svl-sync-import">
              {!syncDiff && !applyResult && (
                <>
                  <div className="svl-sync-import-info">
                    <p>{t('app.sync.import.description')}</p>
                  </div>

                  <button
                    className="svl-sync-import-btn"
                    onClick={handleImport}
                    disabled={importing}
                  >
                    {importing
                      ? t('app.sync.import.importing')
                      : t('app.sync.import.importButton')}
                  </button>
                </>
              )}

              {syncDiff && !applyResult && (
                <div className="svl-sync-diff">
                  <div className="svl-sync-diff-summary">
                    <p>{syncDiff.summary}</p>
                  </div>

                  {syncDiff.missing_mods.length > 0 && (
                    <div className="svl-sync-diff-section">
                      <h3 className="svl-sync-diff-title svl-sync-diff-title--missing">
                        {t('app.sync.diff.missing', { count: syncDiff.missing_mods.length })}
                      </h3>
                      {syncDiff.missing_mods.map((mod) => (
                        <div key={mod.unique_id} className="svl-sync-diff-item svl-sync-diff-item--missing">
                          <span className="svl-sync-diff-item-name">{mod.name}</span>
                          <span className="svl-sync-diff-item-version">v{mod.version}</span>
                        </div>
                      ))}
                    </div>
                  )}

                  {syncDiff.version_mismatch.length > 0 && (
                    <div className="svl-sync-diff-section">
                      <h3 className="svl-sync-diff-title svl-sync-diff-title--mismatch">
                        {t('app.sync.diff.mismatch', { count: syncDiff.version_mismatch.length })}
                      </h3>
                      {syncDiff.version_mismatch.map((mismatch) => (
                        <div key={mismatch.mod_entry.unique_id} className="svl-sync-diff-item svl-sync-diff-item--mismatch">
                          <span className="svl-sync-diff-item-name">{mismatch.mod_entry.name}</span>
                          <span className="svl-sync-diff-item-versions">
                            <span className="svl-sync-diff-item-current">{mismatch.current_version}</span>
                            <span className="svl-sync-diff-item-arrow">→</span>
                            <span className="svl-sync-diff-item-required">{mismatch.required_version}</span>
                          </span>
                        </div>
                      ))}
                    </div>
                  )}

                  {syncDiff.config_diffs.filter((c) => c.status !== 'matched').length > 0 && (
                    <div className="svl-sync-diff-section">
                      <h3 className="svl-sync-diff-title svl-sync-diff-title--config">
                        {t('app.sync.diff.configs', {
                          count: syncDiff.config_diffs.filter((c) => c.status !== 'matched').length,
                        })}
                      </h3>
                      {syncDiff.config_diffs
                        .filter((c) => c.status !== 'matched')
                        .map((config) => (
                          <div key={config.config_file} className="svl-sync-diff-item svl-sync-diff-item--config">
                            <span className="svl-sync-diff-item-name">{config.mod_name}</span>
                            <span className={`svl-sync-diff-item-status svl-sync-diff-item-status--${config.status}`}>
                              {config.status === 'missing'
                                ? t('app.sync.diff.configMissing')
                                : t('app.sync.diff.configMismatch')}
                            </span>
                          </div>
                        ))}
                    </div>
                  )}

                  {syncDiff.total_changes === 0 && (
                    <div className="svl-sync-perfect">
                      <span className="svl-sync-perfect-icon">✅</span>
                      <span>{t('app.sync.diff.perfect')}</span>
                    </div>
                  )}

                  {syncDiff.total_changes > 0 && (
                    <button
                      className="svl-sync-apply-btn"
                      onClick={handleApplySync}
                      disabled={applying}
                    >
                      {applying
                        ? t('app.sync.apply.applying')
                        : t('app.sync.apply.applyButton')}
                    </button>
                  )}
                </div>
              )}

              {applyResult && (
                <div className="svl-sync-apply-result">
                  <div className={`svl-sync-apply-result-header ${applyResult.success ? 'svl-sync-apply-result--success' : 'svl-sync-apply-result--failed'}`}>
                    <span className="svl-sync-apply-result-icon">
                      {applyResult.success ? '✅' : '⚠️'}
                    </span>
                    <p>{applyResult.message}</p>
                  </div>

                  {applyResult.applied_mods.length > 0 && (
                    <div className="svl-sync-apply-result-section">
                      <h4>{t('app.sync.apply.appliedMods')}</h4>
                      <ul>
                        {applyResult.applied_mods.map((name) => (
                          <li key={name}>{name}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {applyResult.configs_applied.length > 0 && (
                    <div className="svl-sync-apply-result-section">
                      <h4>{t('app.sync.apply.configsApplied')}</h4>
                      <ul>
                        {applyResult.configs_applied.map((name) => (
                          <li key={name}>{name}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  <div className="svl-sync-apply-result-notice">
                    <p>{t('app.sync.apply.restartNotice')}</p>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
