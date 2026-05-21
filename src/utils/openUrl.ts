import { openUrl as tauriOpenUrl } from '@tauri-apps/plugin-opener';
import { message } from 'antd';
import i18n from '../i18n';

export async function openUrl(url: string, fallbackMessage?: string): Promise<void> {
  if (!url) {
    message.error(fallbackMessage || i18n.t('app.urlEmpty'));
    return;
  }

  let normalizedUrl = url.trim();

  if (!normalizedUrl.startsWith('http://') && !normalizedUrl.startsWith('https://')) {
    normalizedUrl = 'https://' + normalizedUrl;
  }

  try {
    await tauriOpenUrl(normalizedUrl);
  } catch (error) {
    console.error('Failed to open URL:', normalizedUrl, error);
    message.error(fallbackMessage || i18n.t('app.openUrlFailed'));
  }
}
