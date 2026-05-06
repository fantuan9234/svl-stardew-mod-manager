import { useState, useEffect } from 'react';

export type ThemePreset = 'colorful' | 'eyeCare';

const THEME_KEY = 'svl-theme-preset';

const themeStyles: Record<ThemePreset, string> = {
  colorful: '',
  eyeCare: 'eye-care',
};

export function useTheme() {
  const [theme, setTheme] = useState<ThemePreset>(() => {
    const saved = localStorage.getItem(THEME_KEY);
    return (saved as ThemePreset) || 'colorful';
  });

  useEffect(() => {
    localStorage.setItem(THEME_KEY, theme);
    document.documentElement.className = themeStyles[theme];
  }, [theme]);

  const switchTheme = (preset: ThemePreset) => {
    setTheme(preset);
  };

  return { theme, switchTheme, themeName: theme === 'colorful' ? 'app.theme.colorful' : 'app.theme.eyeCare' };
}
