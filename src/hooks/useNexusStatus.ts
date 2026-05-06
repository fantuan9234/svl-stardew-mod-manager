import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'antd';

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

export async function verifyNexusConnection(apiKey: string): Promise<void> {
  setNexusStatus({ status: 'checking', hasApiKey: true });

  try {
    const result = await invoke<any>('verify_nexus_api_key', { apiKey });

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
  } catch {
    setNexusStatus({
      status: 'disconnected',
      isPremium: false,
      lastChecked: Date.now(),
    });
  }
}

export function useNexusStatus() {
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
      message.warning('请先配置 API Key');
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
