import { useState, useEffect, useRef } from 'react';
import { useLocation } from 'react-router-dom';
import { Typography, Button, Space, Divider, Progress, Tag, message, Slider } from 'antd';
import { useTranslation } from 'react-i18next';
import { CloudDownloadOutlined, CheckCircleOutlined, SyncOutlined, LinkOutlined, UploadOutlined, DeleteOutlined } from '@ant-design/icons';
import { listen } from '@tauri-apps/api/event';
import i18n from '../i18n';
import { useTheme } from '../hooks/useTheme';
import NexusApiConfig from '../components/NexusApiConfig';
import { openUrl } from '../utils/openUrl';
import {
  getCurrentAppVersion,
  checkAppUpdateFromServer,
  downloadAppUpdateFromServer,
  runInstaller,
  AppUpdateInfo,
  AppUpdateProgress,
} from '../utils/tauri-api';

const { Title, Text, Paragraph } = Typography;

export default function Settings() {
  const { t } = useTranslation();
  const location = useLocation();
  const { theme, switchTheme, customColors, updateCustomColors, setBackgroundImage, clearBackgroundImage, setCustomChickenImage, clearCustomChickenImage, setSidebarLogoMode, setCustomSidebarImage, clearCustomSidebarImage } = useTheme();
  const bgInputRef = useRef<HTMLInputElement>(null);
  const chickenInputRef = useRef<HTMLInputElement>(null);
  const sidebarImgInputRef = useRef<HTMLInputElement>(null);

  const [appVersion, setAppVersion] = useState('');
  const [pendingColor, setPendingColor] = useState<string | null>(null);
  const [checking, setChecking] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<AppUpdateInfo | null>(null);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [downloadedBytes, setDownloadedBytes] = useState(0);
  const [totalBytes, setTotalBytes] = useState(0);
  const [downloaded, setDownloaded] = useState(false);
  const [installerPath, setInstallerPath] = useState<string | null>(null);
  const [checkError, setCheckError] = useState<string | null>(null);

  const progressUnlistenRef = useRef<(() => void) | null>(null);
  const updateSectionRef = useRef<HTMLDivElement>(null);

  // 接收从 AppLayout 导航传递的更新信息
  useEffect(() => {
    const state = location.state as { updateInfo?: AppUpdateInfo } | null;
    if (state?.updateInfo) {
      setUpdateInfo(state.updateInfo);
      // 清除导航 state，避免刷新时重复触发
      window.history.replaceState({}, '');
      // 滚动到更新区域
      setTimeout(() => {
        updateSectionRef.current?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }, 300);
    }
  }, [location.state]);

  useEffect(() => {
    getCurrentAppVersion().then(v => setAppVersion(v)).catch(() => setAppVersion('?.?.?'));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    listen<AppUpdateProgress>('app-update-progress', (event) => {
      setDownloadedBytes(event.payload.downloaded);
      setTotalBytes(event.payload.total);
      setProgress(Math.min(Math.round(event.payload.percent), 100));
    }).then(fn => { unlisten = fn; progressUnlistenRef.current = fn; });
    return () => { unlisten?.(); };
  }, []);

  const handleLanguageChange = (lang: string) => {
    i18n.changeLanguage(lang);
    localStorage.setItem('svl-language', lang);
  };

  const handleCheckUpdate = async () => {
    setChecking(true);
    setUpdateInfo(null);
    setCheckError(null);
    setDownloaded(false);
    setDownloading(false);
    setProgress(0);
    try {
      const info = await checkAppUpdateFromServer();
      setUpdateInfo(info);
    } catch (err: any) {
      const errMsg = err?.message || err?.toString() || t('features.updater.checkFailed');
      setCheckError(errMsg);
    } finally {
      setChecking(false);
    }
  };

  const handleDownload = async () => {
    if (!updateInfo) return;
    setDownloading(true);
    setProgress(0);
    setDownloadedBytes(0);
    setTotalBytes(0);
    try {
      const result = await downloadAppUpdateFromServer(updateInfo.download_url);
      if (result.success) {
        setDownloading(false);
        setDownloaded(true);
        if (result.file_path) {
          setInstallerPath(result.file_path);
        }
      } else {
        setDownloading(false);
        message.error(result.message || t('features.updater.downloadFailed'), 5);
      }
    } catch (err: any) {
      setDownloading(false);
      const errMsg = err?.message || err?.toString() || t('features.updater.downloadFailed');
      message.error(errMsg, 5);
    }
  };

  const handleOpenDownloadPage = () => {
    if (!updateInfo?.download_url) return;
    openUrl(updateInfo.download_url, t('app.openUrlFailed'));
  };

  const handleRestart = async () => {
    if (!installerPath) return;
    try {
      await runInstaller(installerPath);
    } catch {
      message.error('Failed to start installer');
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  const formatDate = (dateStr: string | null | undefined): string => {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString();
    } catch {
      return dateStr;
    }
  };

  return (
    <div style={{ padding: 24 }}>
      <Title level={2}>{t('app.pages.settings.title')}</Title>

      <div style={{ marginTop: 24 }}>
        <Text strong style={{ display: 'block', marginBottom: 12 }}>
          {t('app.theme.title')}
        </Text>
        <div className="svl-theme-grid">
          {([
            { key: 'oceanBlue', color: '#2563EB', label: t('app.theme.oceanBlue') },
            { key: 'parchment', color: '#8b6914', label: t('app.theme.parchment') },
            { key: 'mintGreen', color: '#E8A5B0', label: t('app.theme.mintGreen') },
            { key: 'custom', color: customColors.primary, label: t('app.theme.custom') },
          ] as const).map(item => (
            <div
              key={item.key}
              className={`svl-theme-card${theme === item.key ? ' svl-theme-card--active' : ''}`}
              onClick={() => switchTheme(item.key)}
            >
              <div
                className="svl-theme-card-dot"
                style={{ background: item.color }}
              />
              <span className="svl-theme-card-label">{item.label}</span>
            </div>
          ))}
        </div>
        {theme === 'custom' && (
          <div className="svl-theme-custom-panel">
            <div className="svl-theme-color-row">
              <span className="svl-theme-color-label">{t('app.theme.customPrimary')}</span>
              <input
                type="color"
                className="svl-theme-color-picker"
                value={pendingColor ?? customColors.primary}
                onChange={(e) => setPendingColor(e.target.value)}
                onPointerUp={() => {
                  if (pendingColor) {
                    updateCustomColors({ primary: pendingColor });
                    setPendingColor(null);
                  }
                }}
                onBlur={() => {
                  if (pendingColor) {
                    updateCustomColors({ primary: pendingColor });
                    setPendingColor(null);
                  }
                }}
              />
            </div>

            <div className="svl-theme-section-title" style={{ marginTop: 16 }}>
              {t('app.theme.backgroundImage')}
            </div>
            <div style={{ display: 'flex', gap: 8, marginBottom: 12, flexWrap: 'wrap' }}>
              <Button
                size="small"
                icon={<UploadOutlined />}
                onClick={() => bgInputRef.current?.click()}
              >
                {t('app.theme.uploadImage')}
              </Button>
              {customColors.backgroundImage && (
                <Button
                  size="small"
                  danger
                  icon={<DeleteOutlined />}
                  onClick={clearBackgroundImage}
                >
                  {t('app.theme.removeImage')}
                </Button>
              )}
            </div>
            <input
              ref={bgInputRef}
              type="file"
              accept="image/*"
              style={{ display: 'none' }}
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (!file) return;
                if (file.size > 5 * 1024 * 1024) {
                  message.warning(t('app.theme.imageTooLarge'));
                  return;
                }
                const reader = new FileReader();
                reader.onload = (ev) => {
                  const dataUrl = ev.target?.result as string;
                  if (dataUrl) setBackgroundImage(dataUrl);
                };
                reader.readAsDataURL(file);
                e.target.value = '';
              }}
            />
            {customColors.backgroundImage && (
              <div style={{ marginTop: 8 }}>
                <div className="svl-theme-slider-row">
                  <span className="svl-theme-color-label">{t('app.theme.blur')}</span>
                  <Slider
                    min={0}
                    max={50}
                    value={customColors.backgroundBlur}
                    onChange={(v) => updateCustomColors({ backgroundBlur: v })}
                    style={{ flex: 1, margin: '0 12px' }}
                  />
                  <span style={{ fontSize: 12, color: 'var(--svl-text-secondary)', minWidth: 32, textAlign: 'right' }}>
                    {customColors.backgroundBlur}px
                  </span>
                </div>
                <div className="svl-theme-slider-row">
                  <span className="svl-theme-color-label">{t('app.theme.opacity')}</span>
                  <Slider
                    min={5}
                    max={100}
                    value={customColors.backgroundOpacity}
                    onChange={(v) => updateCustomColors({ backgroundOpacity: v })}
                    style={{ flex: 1, margin: '0 12px' }}
                  />
                  <span style={{ fontSize: 12, color: 'var(--svl-text-secondary)', minWidth: 32, textAlign: 'right' }}>
                    {customColors.backgroundOpacity}%
                  </span>
                </div>
              </div>
            )}
            {customColors.backgroundImage && (
              <div style={{
                marginTop: 12,
                borderRadius: 8,
                overflow: 'hidden',
                height: 100,
                position: 'relative',
                border: '1px solid var(--svl-border)',
              }}>
                <img
                  src={customColors.backgroundImage}
                  alt="bg preview"
                  style={{
                    width: '100%',
                    height: '100%',
                    objectFit: 'cover',
                    filter: `blur(${customColors.backgroundBlur}px)`,
                    opacity: customColors.backgroundOpacity / 100,
                  }}
                />
              </div>
            )}

            <div className="svl-theme-section-title" style={{ marginTop: 16 }}>
              {t('app.theme.customImages')}
            </div>

            <div className="svl-theme-color-row" style={{ alignItems: 'center' }}>
              <span className="svl-theme-color-label">{t('app.theme.chickenImage')}</span>
              <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                {customColors.customChickenImage && (
                  <img
                    src={customColors.customChickenImage}
                    alt="chicken preview"
                    style={{
                      width: 32,
                      height: 32,
                      objectFit: 'contain',
                      imageRendering: 'pixelated',
                      borderRadius: 4,
                      border: '1px solid var(--svl-border)',
                    }}
                  />
                )}
                <Button
                  size="small"
                  icon={<UploadOutlined />}
                  onClick={() => chickenInputRef.current?.click()}
                >
                  {t('app.theme.uploadImage')}
                </Button>
                {customColors.customChickenImage && (
                  <Button
                    size="small"
                    danger
                    icon={<DeleteOutlined />}
                    onClick={clearCustomChickenImage}
                  >
                    {t('app.theme.removeImage')}
                  </Button>
                )}
              </div>
            </div>
            <input
              ref={chickenInputRef}
              type="file"
              accept="image/*"
              style={{ display: 'none' }}
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (!file) return;
                if (file.size > 5 * 1024 * 1024) {
                  message.warning(t('app.theme.imageTooLarge'));
                  return;
                }
                const reader = new FileReader();
                reader.onload = (ev) => {
                  const dataUrl = ev.target?.result as string;
                  if (dataUrl) setCustomChickenImage(dataUrl);
                };
                reader.readAsDataURL(file);
                e.target.value = '';
              }}
            />

            <div className="svl-theme-section-title" style={{ marginTop: 16 }}>
              {t('app.theme.sidebarLogo')}
            </div>

            <div className="svl-theme-color-row" style={{ alignItems: 'center' }}>
              <span className="svl-theme-color-label">{t('app.theme.sidebarLogoMode')}</span>
              <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                {([
                  { key: 'daynight', label: t('app.theme.logoDaynight') },
                  { key: 'farm', label: t('app.theme.logoFarm') },
                  { key: 'custom', label: t('app.theme.logoCustom') },
                ] as const).map(item => (
                  <Button
                    key={item.key}
                    size="small"
                    type={customColors.sidebarLogoMode === item.key ? 'primary' : 'default'}
                    onClick={() => setSidebarLogoMode(item.key)}
                  >
                    {item.label}
                  </Button>
                ))}
              </div>
            </div>

            {customColors.sidebarLogoMode === 'custom' && (
              <div className="svl-theme-color-row" style={{ marginTop: 8, alignItems: 'center' }}>
                <span className="svl-theme-color-label">{t('app.theme.sidebarLogoImage')}</span>
                <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                  {customColors.customSidebarImage && (
                    <img
                      src={customColors.customSidebarImage}
                      alt="sidebar logo preview"
                      style={{
                        width: 48,
                        height: 32,
                        objectFit: 'cover',
                        borderRadius: 4,
                        border: '1px solid var(--svl-border)',
                      }}
                    />
                  )}
                  <Button
                    size="small"
                    icon={<UploadOutlined />}
                    onClick={() => sidebarImgInputRef.current?.click()}
                  >
                    {t('app.theme.uploadImage')}
                  </Button>
                  {customColors.customSidebarImage && (
                    <Button
                      size="small"
                      danger
                      icon={<DeleteOutlined />}
                      onClick={clearCustomSidebarImage}
                    >
                      {t('app.theme.removeImage')}
                    </Button>
                  )}
                </div>
                <input
                  ref={sidebarImgInputRef}
                  type="file"
                  accept="image/*"
                  style={{ display: 'none' }}
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (!file) return;
                    if (file.size > 5 * 1024 * 1024) {
                      message.warning(t('app.theme.imageTooLarge'));
                      return;
                    }
                    const reader = new FileReader();
                    reader.onload = (ev) => {
                      const dataUrl = ev.target?.result as string;
                      if (dataUrl) setCustomSidebarImage(dataUrl);
                    };
                    reader.readAsDataURL(file);
                    e.target.value = '';
                  }}
                />
              </div>
            )}

          </div>
        )}
      </div>

      <div style={{ marginTop: 24 }}>
        <Text strong style={{ display: 'block', marginBottom: 12 }}>
          {t('app.language.switch')}
        </Text>
        <Space>
          <Button
            type={i18n.language === 'zh' ? 'primary' : 'default'}
            onClick={() => handleLanguageChange('zh')}
          >
            中文
          </Button>
          <Button
            type={i18n.language === 'zh-TW' ? 'primary' : 'default'}
            onClick={() => handleLanguageChange('zh-TW')}
          >
            繁體中文
          </Button>
          <Button
            type={i18n.language === 'en' ? 'primary' : 'default'}
            onClick={() => handleLanguageChange('en')}
          >
            English
          </Button>
        </Space>
      </div>

      <Divider style={{ marginTop: 32, marginBottom: 24 }} />

      <NexusApiConfig />

      <Divider style={{ marginTop: 32, marginBottom: 24 }} />

      <div ref={updateSectionRef}>
        <Text strong style={{ display: 'block', marginBottom: 16, fontSize: 15 }}>
          {t('app.pages.settings.about')}
        </Text>
        <div style={{
          padding: 20,
          borderRadius: 10,
          background: 'var(--svl-bg-secondary, rgba(255,255,255,0.04))',
          border: '1px solid var(--svl-border, rgba(255,255,255,0.08))',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 12 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <div style={{
                width: 40,
                height: 40,
                borderRadius: 10,
                background: 'linear-gradient(135deg, #6b9e3a, #8fd45a)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                color: '#fff',
                fontWeight: 700,
                fontSize: 16,
              }}>
                S
              </div>
              <div>
                <div style={{ fontWeight: 600, fontSize: 16 }}>SVL</div>
                <Text style={{ fontSize: 12, color: 'var(--svl-text-secondary, rgba(255,255,255,0.5))' }}>
                  v{appVersion}
                </Text>
              </div>
            </div>
            <Button
              type="primary"
              size="small"
              loading={checking}
              onClick={handleCheckUpdate}
              icon={!checking ? <SyncOutlined /> : undefined}
            >
              {t('features.updater.checkButton')}
            </Button>
          </div>

          {checkError && (
            <div style={{
              marginTop: 16,
              padding: '10px 14px',
              borderRadius: 8,
              background: 'rgba(199, 80, 80, 0.1)',
              border: '1px solid rgba(199, 80, 80, 0.25)',
            }}>
              <Text style={{ color: '#c75050', fontSize: 13 }}>{checkError}</Text>
            </div>
          )}

          {updateInfo && !updateInfo.has_update && (
            <div style={{
              marginTop: 16,
              padding: '10px 14px',
              borderRadius: 8,
              background: 'rgba(107, 158, 58, 0.1)',
              border: '1px solid rgba(107, 158, 58, 0.25)',
              display: 'flex',
              alignItems: 'center',
              gap: 8,
            }}>
              <CheckCircleOutlined style={{ color: '#6b9e3a', fontSize: 16 }} />
              <Text style={{ color: '#6b9e3a', fontSize: 13 }}>
                {t('features.updater.upToDate', { version: updateInfo.current_version })}
              </Text>
            </div>
          )}

          {updateInfo && updateInfo.has_update && (
            <div style={{
              marginTop: 16,
              paddingTop: 16,
              borderTop: '1px solid var(--svl-border, rgba(255,255,255,0.08))',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12, flexWrap: 'wrap' }}>
                <Tag color="blue" style={{ margin: 0 }}>
                  {t('features.updater.currentVersion')}: v{updateInfo.current_version}
                </Tag>
                <Tag color="green" style={{ margin: 0 }}>
                  {t('features.updater.latestVersion')}: v{updateInfo.latest_version}
                </Tag>
                {updateInfo.source && (
                  <Tag style={{ margin: 0, background: 'var(--svl-border)', borderColor: 'var(--svl-border-light)' }}>
                    {updateInfo.source === 'github' ? 'GitHub' : updateInfo.source}
                  </Tag>
                )}
              </div>

              {updateInfo.release_date && (
                <div style={{ marginBottom: 8 }}>
                  <Text style={{ fontSize: 12, color: 'var(--svl-text-secondary, rgba(255,255,255,0.5))' }}>
                    {t('features.updater.releaseDate')}: {formatDate(updateInfo.release_date)}
                  </Text>
                </div>
              )}

              {updateInfo.file_size && updateInfo.file_size > 0 && (
                <div style={{ marginBottom: 8 }}>
                  <Text style={{ fontSize: 12, color: 'var(--svl-text-secondary, rgba(255,255,255,0.5))' }}>
                    {t('app.pages.settings.fileSize')}: {formatBytes(updateInfo.file_size)}
                  </Text>
                </div>
              )}

              {updateInfo.release_notes && (
                <div style={{
                  marginTop: 10,
                  padding: 12,
                  borderRadius: 8,
                  background: 'var(--svl-surface-faint)',
                  border: '1px solid var(--svl-border)',
                  maxHeight: 180,
                  overflowY: 'auto',
                }}>
                  <Text strong style={{ fontSize: 12, color: 'var(--svl-text-secondary, rgba(255,255,255,0.6))', display: 'block', marginBottom: 6 }}>
                    {t('features.updater.releaseNotes')}
                  </Text>
                  <Paragraph style={{ whiteSpace: 'pre-wrap', fontSize: 13, margin: 0, color: 'var(--svl-text-primary, rgba(255,255,255,0.85))' }}>
                    {updateInfo.release_notes}
                  </Paragraph>
                </div>
              )}

              {!downloading && !downloaded && (
                <div style={{ marginTop: 14, display: 'flex', gap: 10, flexWrap: 'wrap', alignItems: 'center' }}>
                  <Button
                    type="primary"
                    onClick={handleDownload}
                    icon={<CloudDownloadOutlined />}
                  >
                    {t('features.updater.downloadButton')}
                  </Button>
                  <Button
                    onClick={handleOpenDownloadPage}
                    icon={<LinkOutlined />}
                  >
                    {t('app.pages.settings.openDownloadPage')}
                  </Button>
                </div>
              )}

              {downloading && (
                <div style={{ marginTop: 14 }}>
                  <Progress
                    percent={progress}
                    status="active"
                    strokeColor={{ '0%': 'var(--svl-accent)', '100%': 'var(--svl-accent-light)' }}
                    size="small"
                  />
                  {totalBytes > 0 && (
                    <Text style={{ fontSize: 12, color: 'var(--svl-text-secondary, rgba(255,255,255,0.5))' }}>
                      {formatBytes(downloadedBytes)} / {formatBytes(totalBytes)}
                    </Text>
                  )}
                </div>
              )}

              {downloaded && (
                <div style={{ marginTop: 14 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
                    <CheckCircleOutlined style={{ color: '#6b9e3a', fontSize: 16 }} />
                    <Text style={{ color: '#6b9e3a', fontSize: 13 }}>
                      {t('features.serverUpdater.downloadCompleteRestart')}
                    </Text>
                  </div>
                  <Button type="primary" danger onClick={handleRestart}>
                    {t('features.updater.restartButton')}
                  </Button>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
