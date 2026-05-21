import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { I18nextProvider } from 'react-i18next';
import i18n from './i18n';
import App from './App';
import ModManager from './pages/ModManager';
import NexusModBrowser from './pages/NexusModBrowser';
import SyncPage from './pages/SyncPage';
import ProfilesPage from './pages/ProfilesPage';
import Settings from './pages/Settings';
import SavesManager from './pages/SavesManager';
import DonatePage from './pages/DonatePage';
import LogViewer from './pages/LogViewer';
import './App.css';

document.addEventListener('contextmenu', (e) => {
  e.preventDefault();
});

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <I18nextProvider i18n={i18n}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<App />}>
            <Route index element={<Navigate to="/mod-manager" replace />} />
            <Route path="mod-manager" element={<ModManager />} />
            <Route path="nexus-browser" element={<NexusModBrowser />} />
            <Route path="sync" element={<SyncPage />} />
            <Route path="profiles" element={<ProfilesPage />} />
            <Route path="saves" element={<SavesManager />} />
            <Route path="settings" element={<Settings />} />
            <Route path="donate" element={<DonatePage />} />
            <Route path="log-viewer" element={<LogViewer />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </I18nextProvider>
  </React.StrictMode>,
);
