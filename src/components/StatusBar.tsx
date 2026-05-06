import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'antd';

interface StatusBarProps {
  smapiConnected: boolean;
  modsCount: number;
}

export default function StatusBar({ smapiConnected, modsCount }: StatusBarProps) {
  const { t } = useTranslation();

  const handleOpenLog = async () => {
    try {
      const appData = await invoke<string>('get_appdata_path');
      const logPath = `${appData}\\StardewValley\\ErrorLogs\\SMAPI-latest.txt`;
      await invoke('open_path', { path: logPath });
    } catch {
      message.error(t('app.log.openLogFolder'));
    }
  };

  return (
    <div className="svl-statusbar">
      <div className="svl-statusbar-left" />

      <div className="svl-statusbar-right">
        <div className="svl-status-indicator">
          <div className={`svl-status-dot ${!smapiConnected ? 'disconnected' : ''}`} />
          <span>
            {smapiConnected
              ? t('app.statusbar.smapiConnected')
              : t('app.statusbar.smapiDisconnected')}
          </span>
          <span style={{ color: 'var(--svl-text-muted)' }}>·</span>
          <span>{modsCount} {t('app.statusbar.modsLoaded')}</span>
        </div>
        <button className="svl-log-btn" onClick={handleOpenLog}>
          <span>{t('app.statusbar.openLog')}</span>
          <span style={{ fontSize: 10 }}>↗</span>
        </button>
      </div>
    </div>
  );
}
