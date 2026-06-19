import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from 'antd';
import { PieChartOutlined, ControlOutlined, SaveOutlined, FileTextOutlined, TranslationOutlined, BugOutlined, EditOutlined, HistoryOutlined } from '@ant-design/icons';
import StorageAnalyzerView from '../components/StorageAnalyzerView';
import ConfigManager from '../components/ConfigManager';
import SnapshotManager from '../components/SnapshotManager';
import AppLogViewer from '../components/AppLogViewer';
import ModTranslatorView from '../components/ModTranslatorView';
import SmapiLogAnalyzer from '../components/SmapiLogAnalyzer';
import SaveEditorView from '../components/SaveEditorView';
import SaveBackupManager from '../components/SaveBackupManager';

type ToolView = 'home' | 'storage' | 'config' | 'snapshot' | 'saveBackup' | 'translate' | 'logs' | 'smapiAnalyzer' | 'saveEditor';

interface ToolItem {
  key: ToolView;
  icon: React.ReactNode;
  color: string;
}

interface ToolGroup {
  labelKey: string;
  tools: ToolItem[];
}

const toolGroups: ToolGroup[] = [
  {
    labelKey: 'app.toolbox.groupMod',
    tools: [
      { key: 'storage', icon: <PieChartOutlined />, color: '#1890ff' },
      { key: 'config', icon: <ControlOutlined />, color: '#13c2c2' },
      { key: 'snapshot', icon: <SaveOutlined />, color: '#096dd9' },
      { key: 'translate', icon: <TranslationOutlined />, color: '#36cfc9' },
    ],
  },
  {
    labelKey: 'app.toolbox.groupSave',
    tools: [
      { key: 'saveBackup', icon: <HistoryOutlined />, color: '#52c41a' },
      { key: 'saveEditor', icon: <EditOutlined />, color: '#389e0d' },
    ],
  },
  {
    labelKey: 'app.toolbox.groupDiag',
    tools: [
      { key: 'smapiAnalyzer', icon: <BugOutlined />, color: '#fa8c16' },
      { key: 'logs', icon: <FileTextOutlined />, color: '#cf1322' },
    ],
  },
];

const RECENT_KEY = 'svl_toolbox_recent';
const MAX_RECENT = 2;

function getRecentTools(): ToolView[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch { return []; }
}

function saveRecentTools(list: ToolView[]) {
  localStorage.setItem(RECENT_KEY, JSON.stringify(list));
}

// 扁平化所有工具，用于按 key 查找
const allTools: Partial<Record<ToolView, ToolItem>> = {};
toolGroups.forEach(g => g.tools.forEach(t => { allTools[t.key] = t; }));

export default function Toolbox() {
  const { t } = useTranslation();
  const [view, setView] = useState<ToolView>('home');
  const [recent, setRecent] = useState<ToolView[]>(getRecentTools);

  const navigateTo = useCallback((key: ToolView) => {
    const updated = [key, ...recent.filter(r => r !== key)].slice(0, MAX_RECENT);
    setRecent(updated);
    saveRecentTools(updated);
    setView(key);
  }, [recent]);

  // 从子视图返回时也记录
  const goHome = useCallback(() => {
    setView('home');
  }, []);

  if (view === 'storage') return <StorageAnalyzerView onBack={goHome} />;
  if (view === 'config') return <ConfigManager onBack={goHome} />;
  if (view === 'snapshot') return <SnapshotManager onBack={goHome} />;
  if (view === 'logs') return <AppLogViewer onBack={goHome} />;
  if (view === 'translate') return <ModTranslatorView onBack={goHome} />;
  if (view === 'smapiAnalyzer') return <SmapiLogAnalyzer onBack={goHome} />;
  if (view === 'saveEditor') return <SaveEditorView onBack={goHome} />;
  if (view === 'saveBackup') return <SaveBackupView onBack={goHome} onOpenSaveEditor={() => setView('saveEditor')} />;

  return (
    <div className="svl-toolbox-home">
      <div className="svl-toolbox-header">
        <h2 className="svl-toolbox-title">{t('app.toolbox.title')}</h2>
        <p className="svl-toolbox-subtitle">{t('app.toolbox.subtitle')}</p>
        {recent.length > 0 && (
          <div className="svl-toolbox-recent">
            <span className="svl-toolbox-recent-label">{t('app.toolbox.recent', '最近使用')}</span>
            {recent.map(key => {
              const tool = allTools[key];
              if (!tool) return null;
              return (
                <span
                  key={key}
                  className="svl-toolbox-recent-pill"
                  style={{ '--tool-color': tool.color } as React.CSSProperties}
                  onClick={() => navigateTo(key)}
                >
                  {tool.icon} {t(`app.toolbox.${key}Title`)}
                </span>
              );
            })}
          </div>
        )}
      </div>

      {toolGroups.map((group) => (
        <div key={group.labelKey} className="svl-toolbox-group">
          <h3 className="svl-toolbox-group-title">{t(group.labelKey)}</h3>
          <div className="svl-toolbox-grid">
            {group.tools.map(tool => (
              <div
                key={tool.key}
                className="svl-toolbox-card"
                style={{ '--tool-color': tool.color } as React.CSSProperties}
                onClick={() => navigateTo(tool.key)}
              >
                <div className="svl-toolbox-card-icon">
                  {tool.icon}
                </div>
                <div className="svl-toolbox-card-body">
                  <span className="svl-toolbox-card-name">
                    {t(`app.toolbox.${tool.key}Title`)}
                  </span>
                  <span className="svl-toolbox-card-desc">
                    {t(`app.toolbox.${tool.key}Desc`)}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function SaveBackupView({ onBack, onOpenSaveEditor }: { onBack: () => void; onOpenSaveEditor: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="svl-toolbox-detail">
      <Button onClick={onBack} style={{ marginBottom: 16 }}>← {t('common.back', '返回')}</Button>
      <SaveBackupManager visible={true} onClose={onBack} onSaveEditorOpen={onOpenSaveEditor} />
    </div>
  );
}
