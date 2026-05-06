import { openUrl as tauriOpenUrl } from '@tauri-apps/plugin-opener';
import { message } from 'antd';

export async function openUrl(url: string, fallbackMessage?: string): Promise<void> {
  if (!url) {
    message.error(fallbackMessage || '链接地址为空');
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
    message.error(fallbackMessage || '打开链接失败，请手动复制链接访问');
  }
}
