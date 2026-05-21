import { useState, useEffect } from 'react';
import { toAssetUrl } from '../utils/tauri-api';

export function useImageUrl(filePath: string | null | undefined): string {
  const [url, setUrl] = useState('');
  useEffect(() => {
    if (!filePath) {
      setUrl('');
      return;
    }
    let cancelled = false;
    toAssetUrl(filePath).then(result => {
      if (!cancelled) setUrl(result);
    });
    return () => { cancelled = true; };
  }, [filePath]);
  return url;
}
