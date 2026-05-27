import { useState, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, Space, Spin, Empty, Typography, message, Input } from 'antd';
import {
  ArrowLeftOutlined,
  ReloadOutlined,
  DownloadOutlined,
  FolderOpenOutlined,
  DeleteOutlined,
  SearchOutlined,
} from '@ant-design/icons';
import {
  getAppLogs,
  exportAppLogs,
  clearOldAppLogs,
  getLogDirPath,
  type AppLogResult,
} from '../utils/tauri-api';

const { Text, Title } = Typography;

const LogIconSvg = ({ color, size = 20 }: { color: string; size?: number }) => (
  <svg width={size} height={size} viewBox="0 0 32 32" fill="none">
    <rect x="5" y="4" width="22" height="24" rx="3" fill={color} opacity="0.08" stroke={color} strokeWidth="1.5" />
    <path d="M5 9h22" stroke={color} strokeWidth="1" opacity="0.2" />
    <line x1="10" y1="14" x2="22" y2="14" stroke={color} strokeWidth="1.5" strokeLinecap="round" opacity="0.4" />
    <line x1="10" y1="18" x2="20" y2="18" stroke={color} strokeWidth="1.5" strokeLinecap="round" opacity="0.3" />
    <line x1="10" y1="22" x2="18" y2="22" stroke={color} strokeWidth="1.5" strokeLinecap="round" opacity="0.2" />
    <circle cx="9" cy="6.5" r="0.9" fill={color} opacity="0.5" />
  </svg>
);

const levelColors: Record<string, string> = {
  INFO: '#6b9e3a',
  WARN: '#c49a3b',
  ERROR: '#c75050',
  DEBUG: '#6b9ec4',
};

export default function AppLogViewer({ onBack }: { onBack: () => void }) {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<AppLogResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState('');
  const logEndRef = useRef<HTMLDivElement>(null);

  const loadLogs = async () => {
    setLoading(true);
    try {
      const result = await getAppLogs(500);
      setLogs(result);
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.logLoadFailed'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadLogs();
    const interval = setInterval(loadLogs, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleExport = async () => {
    try {
      const path = await exportAppLogs();
      message.success(t('app.toolbox.logExportSuccess', { path }));
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.logExportFailed'));
    }
  };

  const handleOpenDir = async () => {
    try {
      const path = await getLogDirPath();
      window.open('file://' + path, '_blank');
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.logOpenDirFailed'));
    }
  };

  const handleClearOld = async () => {
    try {
      await clearOldAppLogs(7);
      message.success(t('app.toolbox.logClearSuccess'));
      await loadLogs();
    } catch (e: any) {
      message.error(e?.toString() || t('app.toolbox.logClearFailed'));
    }
  };

  const filteredLines = logs?.lines.filter(line => {
    if (!filter) return true;
    const lower = filter.toLowerCase();
    return line.toLowerCase().includes(lower);
  }) || [];

  return (
    <div style={{ padding: '24px 28px', maxWidth: 1200, margin: '0 auto', display: 'flex', flexDirection: 'column', height: 'calc(100vh - 48px)' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 14, marginBottom: 20, flexShrink: 0 }}>
        <button
          onClick={onBack}
          style={{
            width: 36, height: 36, borderRadius: 10,
            border: '1px solid rgba(139,115,85,0.2)',
            background: 'rgba(61,50,37,0.5)',
            color: 'var(--svl-text-secondary)',
            cursor: 'pointer',
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            transition: 'all 0.2s',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'rgba(61,50,37,0.8)';
            e.currentTarget.style.borderColor = 'rgba(139,115,85,0.4)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'rgba(61,50,37,0.5)';
            e.currentTarget.style.borderColor = 'rgba(139,115,85,0.2)';
          }}
        >
          <ArrowLeftOutlined />
        </button>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <LogIconSvg color="#c7856b" size={22} />
          <Title level={4} style={{ margin: 0, fontWeight: 600 }}>{t('app.toolbox.logsTitle')}</Title>
        </div>
        <Space style={{ marginLeft: 'auto' }}>
          <Input
            placeholder={t('app.toolbox.logFilter')}
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            prefix={<SearchOutlined style={{ color: 'var(--svl-text-muted)' }} />}
            style={{
              width: 200,
              borderRadius: 10,
              background: 'rgba(45,36,24,0.4)',
              borderColor: 'rgba(139,115,85,0.2)',
            }}
          />
          <Button
            icon={<ReloadOutlined />}
            onClick={loadLogs}
            loading={loading}
            style={{ borderRadius: 10, borderColor: 'rgba(139,115,85,0.2)' }}
          />
          <Button
            icon={<DownloadOutlined />}
            onClick={handleExport}
            style={{ borderRadius: 10, borderColor: 'rgba(139,115,85,0.2)' }}
          >
            {t('app.toolbox.logExport')}
          </Button>
          <Button
            icon={<FolderOpenOutlined />}
            onClick={handleOpenDir}
            style={{ borderRadius: 10, borderColor: 'rgba(139,115,85,0.2)' }}
          >
            {t('app.toolbox.logOpenDir')}
          </Button>
          <Button
            icon={<DeleteOutlined />}
            onClick={handleClearOld}
            style={{ borderRadius: 10, borderColor: 'rgba(139,115,85,0.2)' }}
          >
            {t('app.toolbox.logClearOld')}
          </Button>
        </Space>
      </div>

      <div style={{
        flex: 1,
        borderRadius: 14,
        background: 'linear-gradient(145deg, rgba(61,50,37,0.6), rgba(45,36,24,0.4))',
        border: '1px solid rgba(139,115,85,0.12)',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}>
        <div style={{
          padding: '10px 16px',
          borderBottom: '1px solid rgba(139,115,85,0.12)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexShrink: 0,
        }}>
          <Text type="secondary" style={{ fontSize: 12 }}>
            {t('app.toolbox.logLines', { count: filteredLines.length })}
          </Text>
          {logs?.log_dir && (
            <Text type="secondary" style={{ fontSize: 12 }}>
              {logs.log_dir}
            </Text>
          )}
        </div>

        <div style={{
          flex: 1,
          overflow: 'auto',
          padding: '8px 0',
          fontFamily: "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
          fontSize: 12,
          lineHeight: 1.7,
        }}>
          {loading && !logs && (
            <div style={{ textAlign: 'center', padding: '60px 0' }}>
              <Spin size="large" />
            </div>
          )}

          {!loading && filteredLines.length === 0 && (
            <Empty description={t('app.toolbox.logEmpty')} image={Empty.PRESENTED_IMAGE_SIMPLE} />
          )}

          {filteredLines.map((line, idx) => {
            const levelMatch = line.match(/\[(INFO|WARN|ERROR|DEBUG)\]/);
            const level = levelMatch ? levelMatch[1] : 'INFO';
            const color = levelColors[level] || levelColors.INFO;

            return (
              <div
                key={idx}
                style={{
                  padding: '2px 16px',
                  whiteSpace: 'pre-wrap',
                  wordBreak: 'break-all',
                  borderLeft: `2px solid transparent`,
                  transition: 'border-color 0.15s, background 0.15s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderLeftColor = color;
                  e.currentTarget.style.background = `${color}08`;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderLeftColor = 'transparent';
                  e.currentTarget.style.background = 'transparent';
                }}
              >
                <span style={{ color }}>{line.substring(0, line.indexOf(']') + 1)}</span>
                <span style={{ color: 'var(--svl-text-secondary)' }}>{line.substring(line.indexOf(']') + 1)}</span>
              </div>
            );
          })}
          <div ref={logEndRef} />
        </div>
      </div>
    </div>
  );
}
