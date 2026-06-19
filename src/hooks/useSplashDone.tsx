import { createContext, useContext, type ReactNode } from 'react';

const SplashContext = createContext<boolean>(true);

export function SplashProvider({ splashDone, children }: { splashDone: boolean; children: ReactNode }) {
  return <SplashContext.Provider value={splashDone}>{children}</SplashContext.Provider>;
}

export function useSplashDone(): boolean {
  return useContext(SplashContext);
}
