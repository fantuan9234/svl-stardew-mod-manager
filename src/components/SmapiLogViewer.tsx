import React, { useState, useEffect, useRef, useCallback } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';

interface LogLine {
  line: string;
  timestamp: string;
}

interface SmapiLogViewerProps {
  isOpen: boolean;
  onClose: () => void;
}

const LEVEL_REGEX = /^\[(\d{2}:\d{2}:\d{2})\s+(TRACE|DEBUG|INFO|WARN|ERROR|ALERT)\s+([^\]]+)\]\s*/;
const ANSI_ESCAPE = /\x1b\[[0-9;]*m/g;

function stripAnsi(s: string): string {
  return s.replace(ANSI_ESCAPE, '');
}

function getColorClass(line: string): string {
  const raw = stripAnsi(line.trim());
  if (!raw) return 'svl-log-dim';

  const levelMatch = raw.match(LEVEL_REGEX);
  if (levelMatch) {
    const level = levelMatch[2];
    const mod = levelMatch[3];
    const msg = raw.substring(levelMatch[0].length).toLowerCase();

    if (level === 'ERROR') return 'svl-log-red';
    if (level === 'ALERT') return 'svl-log-magenta';
    if (level === 'WARN') return 'svl-log-yellow';

    if (mod === 'SMAPI' || mod.includes('SMAPI')) {
      if (msg.includes('error') || msg.includes('exception') || msg.includes('failed') || msg.includes('crashed') || msg.includes('stack trace')) return 'svl-log-red';
      if (msg.includes('skipped mods') || msg.includes('could not be added') || msg.includes('no longer compatible') || msg.includes("couldn't be loaded")) return 'svl-log-red';
      if (msg.includes('warning') || msg.includes('warn')) return 'svl-log-yellow';
      if (msg.includes('you can update') || msg.includes('update available')) return 'svl-log-magenta';
      if (msg.includes('mods loaded and ready') || msg.includes('everything seems fine') || msg.includes('loaded and ready')) return 'svl-log-green';
      if (msg.includes('type ') && msg.includes(' for help')) return 'svl-log-cyan';
      return 'svl-log-gray';
    }

    if (mod === 'game' && msg.includes('achievements won')) return 'svl-log-yellow';
    if (msg.includes('mods loaded and ready') || msg.includes('everything seems fine')) return 'svl-log-green';
    if (msg.includes('you can update') || msg.includes('update available')) return 'svl-log-magenta';
    if (level === 'TRACE' || level === 'DEBUG') return 'svl-log-gray';
    return 'svl-log-white';
  }

  const rest = raw.slice(raw.indexOf(']') + 1).trim();

  if (raw.indexOf('[SMAPI]') === 0) {
    const msg = rest.toLowerCase();
    if (msg.includes('error') || msg.includes('exception') || msg.includes('failed') || msg.includes('crashed') || msg.includes('stack trace')) return 'svl-log-red';
    if (msg.includes('skipped mods') || msg.includes('could not be added') || msg.includes('no longer compatible') || msg.includes("couldn't be loaded")) return 'svl-log-red';
    if (msg.includes('warning') || msg.includes('warn')) return 'svl-log-yellow';
    if (msg.includes('you can update') || msg.includes('update available')) return 'svl-log-magenta';
    if (msg.includes('mods loaded and ready') || msg.includes('everything seems fine') || msg.includes('loaded and ready')) return 'svl-log-green';
    if (msg.includes('type ') && msg.includes(' for help')) return 'svl-log-cyan';
    return 'svl-log-gray';
  }

  if (raw.indexOf('[game]') === 0) {
    const msg = rest.toLowerCase();
    if (msg.includes('achievements won') || msg.includes('configure your game') || msg.includes('install guide')) return 'svl-log-yellow';
    return 'svl-log-white';
  }

  if (raw.indexOf('[') === 0) {
    const msg = rest.toLowerCase();
    if (msg.includes('error') || msg.includes('exception') || msg.includes('failed') || msg.includes('crashed')) return 'svl-log-red';
    if (msg.includes('warning') || msg.includes('warn')) return 'svl-log-yellow';
    if (msg.includes('you can update') || msg.includes('update available')) return 'svl-log-magenta';
    if (msg.includes('everything seems fine') || msg.includes('installation check completed')) return 'svl-log-green';
    return 'svl-log-white';
  }

  const lower = raw.toLowerCase();
  if (/^[-=]{3,}$/.test(raw)) return 'svl-log-dim';
  if (raw.indexOf('   ') === 0 || raw.indexOf('    ') === 0) return 'svl-log-dim';
  if (raw.indexOf('https://') === 0 || raw.indexOf('http://') === 0) return 'svl-log-cyan';
  if (lower.includes('error') || lower.includes('exception') || lower.includes('stack trace') || lower.includes('unhandled') || lower.includes('fatal')) return 'svl-log-red';
  if (lower.includes('warning') || lower.includes('warn')) return 'svl-log-yellow';
  if (lower.includes('you can update')) return 'svl-log-magenta';
  if (lower.includes('loaded and ready') || lower.includes('everything seems fine')) return 'svl-log-green';
  return 'svl-log-white';
}

const SmapiLogViewer: React.FC<SmapiLogViewerProps> = ({ isOpen, onClose }) => {
  const { t } = useTranslation();
  const [logLines, setLogLines] = useState<LogLine[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [gameStarted, setGameStarted] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const autoScrollRef = useRef(true);
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  const scrollToBottom = useCallback(() => {
    if (autoScrollRef.current && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'instant' as ScrollBehavior });
    }
  }, []);

  useEffect(() => {
    if (!isOpen) return;
    const setupListeners = async () => {
      const unlistenLog = await listen<{ line: string; timestamp: string }>(
        'smapi-log-line',
        (event) => {
          setLogLines((prev) => {
            const newLines = [...prev, event.payload];
            if (newLines.length > 5000) return newLines.slice(-3000);
            return newLines;
          });
          requestAnimationFrame(scrollToBottom);
        }
      );
      const unlistenStart = await listen('smapi-game-started', () => {
        setGameStarted(true);
        setIsRunning(true);
      });
      const unlistenEnd = await listen('smapi-game-ended', () => {
        setIsRunning(false);
      });
      unlistenersRef.current = [unlistenLog, unlistenStart, unlistenEnd];
    };
    setupListeners();
    return () => { unlistenersRef.current.forEach((fn) => fn()); unlistenersRef.current = []; };
  }, [isOpen, scrollToBottom]);

  useEffect(() => {
    if (!isOpen) { setLogLines([]); setGameStarted(false); setIsRunning(false); }
  }, [isOpen]);

  const handleScroll = useCallback(() => {
    if (!containerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
    autoScrollRef.current = scrollHeight - scrollTop - clientHeight < 50;
  }, []);

  const handleClose = () => {
    onClose();
    getCurrentWindow().show().catch(() => {});
  };

  if (!isOpen) return null;

  return (
    <div className="svl-smapi-log-overlay">
      <div className="svl-smapi-log-panel">
        <div className="svl-smapi-log-header">
          <div className="svl-smapi-log-title">
            <span className="svl-smapi-log-icon">📜</span>
            <span>{t('app.smapiLog.title', 'SMAPI 日志')}</span>
            {isRunning && (
              <span className="svl-smapi-log-status-running">
                <span className="svl-status-dot" />
                {t('app.smapiLog.running', '运行中')}
              </span>
            )}
            {!isRunning && gameStarted && (
              <span className="svl-smapi-log-status-ended">
                {t('app.smapiLog.ended', '已结束')}
              </span>
            )}
          </div>
          <div className="svl-smapi-log-actions">
            <button className="svl-smapi-log-btn-minimize" onClick={handleClose} title={t('app.smapiLog.minimize', '关闭面板')}>─</button>
            <button className="svl-smapi-log-btn-close" onClick={handleClose} title={t('app.common.close', '关闭')}>✕</button>
          </div>
        </div>

        <div className="svl-smapi-log-content" ref={containerRef} onScroll={handleScroll}>
          {logLines.length === 0 && !gameStarted && (
            <div className="svl-smapi-log-empty">
              <div className="svl-smapi-log-empty-icon">⏳</div>
              <p>{t('app.smapiLog.waiting', '等待游戏启动...')}</p>
            </div>
          )}

          {logLines.map((log, index) => (
            <div key={index} className="svl-smapi-log-line">
              <span className="svl-smapi-log-time">{log.timestamp}</span>
              <span className={`svl-smapi-log-text ${getColorClass(log.line)}`}>{log.line}</span>
            </div>
          ))}

          <div ref={logEndRef} />
        </div>

        <div className="svl-smapi-log-footer">
          <span className="svl-smapi-log-line-count">
            {t('app.smapiLog.lineCount', '共 {{count}} 行', { count: logLines.length })}
          </span>
          <button
            className="svl-smapi-log-btn-scroll"
            onClick={() => { autoScrollRef.current = true; scrollToBottom(); }}
          >
            {t('app.smapiLog.scrollToBottom', '滚动到底部')}
          </button>
        </div>
      </div>
    </div>
  );
};

export default SmapiLogViewer;
