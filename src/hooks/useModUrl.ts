import { useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface NexusLinkResult {
  url: string;
  method: string;
  mod_id: string | null;
}

interface UseModUrlResult {
  url: string | null;
  isLoading: boolean;
  resolve: (uniqueId: string, modName?: string, nexusModId?: number | null) => Promise<void>;
}

const urlCache = new Map<string, string>();

export function useModUrl(): UseModUrlResult {
  const [url, setUrl] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const abortRef = useRef(false);

  const resolve = useCallback(async (uniqueId: string, modName?: string, nexusModId?: number | null) => {
    const cacheKey = `${uniqueId}_${nexusModId || ''}`;
    if (urlCache.has(cacheKey)) {
      const cached = urlCache.get(cacheKey)!;
      setUrl(cached);
      return;
    }

    setIsLoading(true);
    abortRef.current = false;

    try {
      const result = await invoke<NexusLinkResult>('get_nexus_link', {
        uniqueId: uniqueId,
        modName: modName || null,
        nexusModId: nexusModId || null,
      });
      
      if (!abortRef.current) {
        urlCache.set(cacheKey, result.url);
        setUrl(result.url);
      }
    } catch (error: any) {
      console.error('[useModUrl] Resolution failed:', error);
      
      if (!abortRef.current) {
        setIsLoading(false);
      }
    } finally {
      if (!abortRef.current) {
        setIsLoading(false);
      }
    }
  }, []);

  return { url, isLoading, resolve };
}
