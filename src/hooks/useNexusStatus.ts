import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'antd';
import { useTranslation } from 'react-i18next';

export type NexusConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'checking';

interface NexusStatusState {
  status: NexusConnectionStatus;
  isPremium: boolean;
  hasApiKey: boolean;
  lastChecked: number | null;
}

let globalState: NexusStatusState = {
  status: 'disconnected',
  isPremium: false,
  hasApiKey: false,
  lastChecked: null,
};

const listeners = new Set<(state: NexusStatusState) => void>();

function notifyListeners() {
  listeners.forEach(listener => listener({ ...globalState }));
}

export function getNexusStatus(): NexusStatusState {
  return { ...globalState };
}

export function setNexusStatus(state: Partial<NexusStatusState>) {
  globalState = { ...globalState, ...state };
  notifyListeners();
}

const VERIFY_TIMEOUT_MS = 15000;

let verifyTimeoutId: ReturnType<typeof setTimeout> | null = null;

export async function verifyNexusConnection(apiKey: string): Promise<void> {
  setNexusStatus({ status: 'checking', hasApiKey: true });

  if (verifyTimeoutId) {
    clearTimeout(verifyTimeoutId);
    verifyTimeoutId = null;
  }

  try {
    const result = await Promise.race([
      invoke<any>('verify_nexus_api_key', { apiKey }),
      new Promise<never>((_, reject) => {
        verifyTimeoutId = setTimeout(() => reject(new Error('timeout')), VERIFY_TIMEOUT_MS);
      }),
    ]);

    if (verifyTimeoutId) {
      clearTimeout(verifyTimeoutId);
      verifyTimeoutId = null;
    }

    if (result.success) {
      setNexusStatus({
        status: 'connected',
        isPremium: result.is_premium || false,
        lastChecked: Date.now(),
      });
    } else {
      setNexusStatus({
        status: 'disconnected',
        isPremium: false,
        lastChecked: Date.now(),
      });
    }
  } catch (error: any) {
    if (verifyTimeoutId) {
      clearTimeout(verifyTimeoutId);
      verifyTimeoutId = null;
    }

    const isTimeout = error?.message === 'timeout';
    const isNetworkError = String(error || '').includes('请求失败');

    setNexusStatus({
      status: 'disconnected',
      isPremium: false,
      lastChecked: Date.now(),
    });

    if (isTimeout) {
      message.warning('连接超时，请稍后重试');
    } else if (isNetworkError) {
      message.warning('网络连接失败，请检查网络设置');
    }
  }
}

export function useNexusStatus() {
  const { t } = useTranslation();
  const [state, setState] = useState<NexusStatusState>(globalState);
  const stateRef = useRef(globalState);

  useEffect(() => {
    const listener = (newState: NexusStatusState) => {
      stateRef.current = newState;
      setState(newState);
    };

    listeners.add(listener);
    setState({ ...globalState });

    return () => {
      listeners.delete(listener);
    };
  }, []);

  const reconnect = useCallback(async () => {
    const apiKey = localStorage.getItem('svl-nexus-api-key');
    if (!apiKey) {
      message.warning(t('app.configureApiKeyFirst'));
      return;
    }
    await verifyNexusConnection(apiKey);
  }, []);

  const disconnect = useCallback(() => {
    localStorage.removeItem('svl-nexus-api-key');
    setNexusStatus({
      status: 'disconnected',
      isPremium: false,
      hasApiKey: false,
      lastChecked: null,
    });
  }, []);

  return {
    ...state,
    reconnect,
    disconnect,
  };
}
