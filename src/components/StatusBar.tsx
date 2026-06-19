import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip, message, Popover } from 'antd';
import { AppstoreOutlined } from '@ant-design/icons';

interface StatusBarProps {
  smapiConnected: boolean;
  smapiVersion: string | null;
  modsCount: number;
  gamePath: string;
}

export default function StatusBar({ smapiConnected, smapiVersion, modsCount, gamePath }: StatusBarProps) {
  const { t } = useTranslation();
  const [pathDetailOpen, setPathDetailOpen] = useState(false);

  const handleOpenLog = async () => {
    try {
      const appData = await invoke<string>('get_appdata_path');
      const logPath = `${appData}\\StardewValley\\ErrorLogs\\SMAPI-latest.txt`;
      await invoke('open_path', { path: logPath });
    } catch {
      message.error(t('app.log.openLogFolder'));
    }
  };

  const handleOpenGameFolder = async () => {
    if (!gamePath) return;
    try {
      await invoke('open_path', { path: gamePath });
    } catch {
      message.error(t('app.smapiInstaller.openGamePathFailed'));
    }
  };

  const pathShort = gamePath
    ? '...' + gamePath.replace(/[/\\]$/, '').split(/[/\\]/).pop()
    : t('app.pages.modManager.gameNotFound', '未检测到');

  const smapiDisplay = smapiVersion && smapiVersion !== 'Installed'
    ? `SMAPI v${smapiVersion}`
    : (smapiConnected ? t('app.statusbar.smapiInstalled', 'SMAPI 已安装') : t('app.statusbar.smapiDisconnected'));

  return (
    <div className="svl-statusbar">
      <div className="svl-statusbar-left">
        <Popover
          open={pathDetailOpen}
          onOpenChange={setPathDetailOpen}
          content={
            <div style={{ maxWidth: 360 }}>
              <div style={{ fontSize: 11, color: 'var(--svl-text-muted)', marginBottom: 4 }}>
                {t('app.statusbar.gamePath', '游戏路径')}
              </div>
              <code
                style={{
                  display: 'block',
                  wordBreak: 'break-all',
                  fontSize: 12,
                  padding: '6px 8px',
                  background: 'var(--svl-bg-secondary)',
                  borderRadius: 4,
                  cursor: 'pointer',
                }}
                onClick={() => { setPathDetailOpen(false); handleOpenGameFolder(); }}
              >
                {gamePath || t('app.statusbar.noPath', '未设置')}
              </code>
            </div>
          }
          trigger="click"
          placement="top"
        >
          <div className="svl-status-segment" title={gamePath || ''}>
            <span className="svl-status-segment-icon">📁</span>
            <span className="svl-status-segment-text">{pathShort}</span>
          </div>
        </Popover>

        <div className="svl-status-separator" />

        <Tooltip title={smapiDisplay}>
          <div className={`svl-status-segment ${!smapiConnected ? 'svl-status-segment--warn' : ''}`}>
            <div className={`svl-status-dot ${!smapiConnected ? 'disconnected' : ''}`} />
            <span className="svl-status-segment-text">{smapiDisplay}</span>
          </div>
        </Tooltip>

        <div className="svl-status-separator" />

        <Tooltip title={t('app.statusbar.modsLoadedTip', '已加载模组数量')}>
          <div className="svl-status-segment">
            <span className="svl-status-segment-icon"><AppstoreOutlined /></span>
            <span className="svl-status-segment-text">
              {modsCount} {t('app.statusbar.modsLoaded')}
            </span>
          </div>
        </Tooltip>
      </div>

      <div className="svl-statusbar-right">
        <button className="svl-log-btn" onClick={handleOpenLog}>
          <span>{t('app.statusbar.openLog')}</span>
          <span style={{ fontSize: 10 }}>↗</span>
        </button>
      </div>
    </div>
  );
}
