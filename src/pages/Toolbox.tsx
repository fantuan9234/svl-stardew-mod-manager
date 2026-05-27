import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { PieChartOutlined, ControlOutlined, SaveOutlined, FileTextOutlined } from '@ant-design/icons';
import StorageAnalyzerView from '../components/StorageAnalyzerView';
import ConfigManager from '../components/ConfigManager';
import SnapshotManager from '../components/SnapshotManager';
import AppLogViewer from '../components/AppLogViewer';

type ToolView = 'home' | 'storage' | 'config' | 'snapshot' | 'logs';

const tools: {
  key: ToolView;
  icon: React.ReactNode;
  color: string;
}[] = [
  { key: 'storage', icon: <PieChartOutlined />, color: '#52c41a' },
  { key: 'config', icon: <ControlOutlined />, color: '#faad14' },
  { key: 'snapshot', icon: <SaveOutlined />, color: '#1890ff' },
  { key: 'logs', icon: <FileTextOutlined />, color: '#ff4d4f' },
];

export default function Toolbox() {
  const { t } = useTranslation();
  const [view, setView] = useState<ToolView>('home');

  if (view === 'storage') return <StorageAnalyzerView onBack={() => setView('home')} />;
  if (view === 'config') return <ConfigManager onBack={() => setView('home')} />;
  if (view === 'snapshot') return <SnapshotManager onBack={() => setView('home')} />;
  if (view === 'logs') return <AppLogViewer onBack={() => setView('home')} />;

  return (
    <div className="svl-toolbox-home">
      <div className="svl-toolbox-header">
        <h2 className="svl-toolbox-title">{t('app.toolbox.title')}</h2>
        <p className="svl-toolbox-subtitle">{t('app.toolbox.subtitle')}</p>
      </div>
      <div className="svl-toolbox-grid">
        {tools.map(tool => (
            <div
              key={tool.key}
              className="svl-toolbox-card"
              style={{ '--tool-color': tool.color } as React.CSSProperties}
              onClick={() => setView(tool.key)}
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
  );
}
