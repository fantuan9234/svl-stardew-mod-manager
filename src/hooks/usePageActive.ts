import { createContext, useContext } from 'react';

const PageActiveContext = createContext<string>('/mod-manager');

export const PageActiveProvider = PageActiveContext.Provider;

export function usePageActive(path: string): boolean {
  const activePath = useContext(PageActiveContext);
  return activePath === path;
}

export function useActivePath(): string {
  return useContext(PageActiveContext);
}
