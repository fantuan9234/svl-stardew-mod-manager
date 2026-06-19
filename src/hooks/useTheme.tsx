import { useState, useEffect, useCallback, createContext, useContext, type ReactNode } from 'react';

export type ThemePreset = 'oceanBlue' | 'parchment' | 'mintGreen' | 'custom';
export type SidebarLogoMode = 'daynight' | 'farm' | 'custom';

export interface CustomThemeColors {
  primary: string;
  accent: string;
  bgPrimary: string;
  bgSecondary: string;
  bgCard: string;
  textPrimary: string;
  textSecondary: string;
  backgroundImage: string;
  backgroundBlur: number;
  backgroundOpacity: number;
  autoColors: boolean;
  customChickenImage: string;
  sidebarLogoMode: SidebarLogoMode;
  customSidebarImage: string;
}

const THEME_KEY = 'svl-theme-preset';
const CUSTOM_COLORS_KEY = 'svl-custom-theme-colors';

const presetClassNames: Record<ThemePreset, string> = {
  oceanBlue: '',
  parchment: 'parchment',
  mintGreen: 'mint-green',
  custom: 'theme-custom',
};

const defaultCustomColors: CustomThemeColors = {
  primary: '#2563EB',
  accent: '#3B82F6',
  bgPrimary: '#0f172a',
  bgSecondary: '#1e293b',
  bgCard: '#1e293b',
  textPrimary: '#f1f5f9',
  textSecondary: '#94a3b8',
  backgroundImage: '',
  backgroundBlur: 20,
  backgroundOpacity: 30,
  autoColors: true,
  customChickenImage: '',
  sidebarLogoMode: 'daynight' as SidebarLogoMode,
  customSidebarImage: '',
};

function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? { r: parseInt(result[1], 16), g: parseInt(result[2], 16), b: parseInt(result[3], 16) }
    : null;
}

function rgbToHsl(r: number, g: number, b: number): { h: number; s: number; l: number } {
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0;
  const l = (max + min) / 2;
  if (max !== min) {
    const d = max - min;
    s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) / 6; break;
      case g: h = ((b - r) / d + 2) / 6; break;
      case b: h = ((r - g) / d + 4) / 6; break;
    }
  }
  return { h: h * 360, s: s * 100, l: l * 100 };
}

function hslToHex(h: number, s: number, l: number): string {
  s /= 100; l /= 100;
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    const color = l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1);
    return Math.round(255 * color).toString(16).padStart(2, '0');
  };
  return `#${f(0)}${f(8)}${f(4)}`;
}

function rgba(hex: string, alpha: number): string {
  const rgb = hexToRgb(hex);
  if (!rgb) return hex;
  return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${alpha})`;
}

function lighten(hex: string, amount: number): string {
  const rgb = hexToRgb(hex);
  if (!rgb) return hex;
  const r = Math.min(255, rgb.r + Math.round((255 - rgb.r) * amount));
  const g = Math.min(255, rgb.g + Math.round((255 - rgb.g) * amount));
  const b = Math.min(255, rgb.b + Math.round((255 - rgb.b) * amount));
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
}

function darken(hex: string, amount: number): string {
  const rgb = hexToRgb(hex);
  if (!rgb) return hex;
  const r = Math.max(0, Math.round(rgb.r * (1 - amount)));
  const g = Math.max(0, Math.round(rgb.g * (1 - amount)));
  const b = Math.max(0, Math.round(rgb.b * (1 - amount)));
  return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
}

export function generateAutoColors(primary: string): Pick<CustomThemeColors, 'accent' | 'bgPrimary' | 'bgSecondary' | 'bgCard' | 'textPrimary' | 'textSecondary'> {
  const rgb = hexToRgb(primary);
  if (!rgb) return {
    accent: '#6b9e3a',
    bgPrimary: '#1f140d',
    bgSecondary: '#2a1d14',
    bgCard: '#2a1d14',
    textPrimary: '#f0e6d3',
    textSecondary: '#c4b89a',
  };

  const hsl = rgbToHsl(rgb.r, rgb.g, rgb.b);

  const accentHue = (hsl.h + 120) % 360;
  const accent = hslToHex(accentHue, Math.min(hsl.s + 10, 80), Math.min(hsl.l + 5, 55));

  const bgPrimary = hslToHex(hsl.h, Math.min(hsl.s * 0.4, 25), 8);
  const bgSecondary = hslToHex(hsl.h, Math.min(hsl.s * 0.45, 28), 12);
  const bgCard = hslToHex(hsl.h, Math.min(hsl.s * 0.45, 28), 13);

  const textPrimary = hslToHex(hsl.h, Math.min(hsl.s * 0.3, 20), 92);
  const textSecondary = hslToHex(hsl.h, Math.min(hsl.s * 0.35, 22), 72);

  return { accent, bgPrimary, bgSecondary, bgCard, textPrimary, textSecondary };
}

function applyCustomThemeCSS(colors: CustomThemeColors) {
  const root = document.documentElement;
  const p = colors.primary;
  const a = colors.accent;
  root.style.setProperty('--svl-primary', p);
  root.style.setProperty('--svl-primary-hover', lighten(p, 0.15));
  root.style.setProperty('--svl-primary-light', lighten(p, 0.3));
  root.style.setProperty('--svl-primary-glow', rgba(p, 0.4));
  root.style.setProperty('--svl-accent', a);
  root.style.setProperty('--svl-accent-hover', lighten(a, 0.15));
  root.style.setProperty('--svl-accent-light', lighten(a, 0.3));
  root.style.setProperty('--svl-accent-glow', rgba(a, 0.4));
  root.style.setProperty('--svl-accent-bg-subtle', rgba(a, 0.03));
  root.style.setProperty('--svl-accent-bg-light', rgba(a, 0.05));
  root.style.setProperty('--svl-accent-bg', rgba(a, 0.1));
  root.style.setProperty('--svl-accent-bg-medium', rgba(a, 0.15));
  root.style.setProperty('--svl-accent-bg-strong', rgba(a, 0.2));
  root.style.setProperty('--svl-accent-border-subtle', rgba(a, 0.1));
  root.style.setProperty('--svl-accent-border-light', rgba(a, 0.15));
  root.style.setProperty('--svl-accent-border', rgba(a, 0.2));
  root.style.setProperty('--svl-accent-border-medium', rgba(a, 0.25));
  root.style.setProperty('--svl-accent-border-strong', rgba(a, 0.3));
  root.style.setProperty('--svl-bg-primary', colors.bgPrimary);
  root.style.setProperty('--svl-bg-secondary', colors.bgSecondary);
  root.style.setProperty('--svl-bg-card', colors.bgCard);
  root.style.setProperty('--svl-bg-card-hover', lighten(colors.bgCard, 0.08));
  root.style.setProperty('--svl-bg-footer', darken(colors.bgSecondary, 0.05));
  root.style.setProperty('--svl-bg-hover', rgba(p, 0.15));
  root.style.setProperty('--svl-text-primary', colors.textPrimary);
  root.style.setProperty('--svl-text-secondary', colors.textSecondary);
  root.style.setProperty('--svl-text-muted', darken(colors.textSecondary, 0.3));
  root.style.setProperty('--svl-text-accent', lighten(p, 0.3));
  root.style.setProperty('--svl-border', lighten(colors.bgCard, 0.12));
  root.style.setProperty('--svl-border-light', lighten(colors.bgCard, 0.18));
  root.style.setProperty('--svl-nexus-border', a);
  root.style.setProperty('--svl-nexus-border-light', lighten(a, 0.3));
  root.style.setProperty('--svl-nexus-text', lighten(a, 0.15));
  root.style.setProperty('--svl-nexus-bg', rgba(a, 0.1));
  root.style.setProperty('--svl-nexus-gradient-from', darken(a, 0.25));
  root.style.setProperty('--svl-nexus-gradient-to', a);
  root.style.setProperty('--svl-nexus-gradient-hover-from', darken(a, 0.15));
  root.style.setProperty('--svl-nexus-gradient-hover-to', lighten(a, 0.1));
  root.style.setProperty('--svl-nexus-glow', rgba(a, 0.4));
  root.style.setProperty('--svl-nexus-focus-glow', rgba(a, 0.2));
  root.style.setProperty('--svl-surface-hover', 'rgba(255, 255, 255, 0.1)');
  root.style.setProperty('--svl-surface-subtle', 'rgba(255, 255, 255, 0.05)');
  root.style.setProperty('--svl-surface-faint', 'rgba(255, 255, 255, 0.04)');

  applyBackgroundImage(colors);
}

function applyBackgroundImage(colors: CustomThemeColors) {
  let bgEl = document.getElementById('svl-custom-bg') as HTMLDivElement | null;
  if (!colors.backgroundImage) {
    if (bgEl) bgEl.remove();
    document.body.classList.remove('svl-has-custom-bg');
    return;
  }
  if (!bgEl) {
    bgEl = document.createElement('div');
    bgEl.id = 'svl-custom-bg';
    document.body.prepend(bgEl);
  }
  document.body.classList.add('svl-has-custom-bg');
  const safeUrl = colors.backgroundImage.replace(/"/g, '%22').replace(/\)/g, '%29');
  bgEl.style.cssText = `
    position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 0;
    background-image: url("${safeUrl}");
    background-size: cover; background-position: center; background-repeat: no-repeat;
    filter: blur(${colors.backgroundBlur}px);
    opacity: ${colors.backgroundOpacity / 100};
    pointer-events: none;
  `;
}

function clearCustomThemeCSS() {
  const root = document.documentElement;
  const props = [
    '--svl-primary', '--svl-primary-hover', '--svl-primary-light', '--svl-primary-glow',
    '--svl-accent', '--svl-accent-hover', '--svl-accent-light', '--svl-accent-glow',
    '--svl-accent-bg-subtle', '--svl-accent-bg-light', '--svl-accent-bg', '--svl-accent-bg-medium', '--svl-accent-bg-strong',
    '--svl-accent-border-subtle', '--svl-accent-border-light', '--svl-accent-border', '--svl-accent-border-medium', '--svl-accent-border-strong',
    '--svl-bg-primary', '--svl-bg-secondary', '--svl-bg-card', '--svl-bg-card-hover', '--svl-bg-footer', '--svl-bg-hover',
    '--svl-text-primary', '--svl-text-secondary', '--svl-text-muted', '--svl-text-accent',
    '--svl-border', '--svl-border-light',
    '--svl-nexus-border', '--svl-nexus-border-light', '--svl-nexus-text', '--svl-nexus-bg',
    '--svl-nexus-gradient-from', '--svl-nexus-gradient-to', '--svl-nexus-gradient-hover-from', '--svl-nexus-gradient-hover-to',
    '--svl-nexus-glow', '--svl-nexus-focus-glow',
    '--svl-surface-hover', '--svl-surface-subtle', '--svl-surface-faint',
  ];
  props.forEach(p => root.style.removeProperty(p));
  const bgEl = document.getElementById('svl-custom-bg');
  if (bgEl) bgEl.remove();
  document.body.classList.remove('svl-has-custom-bg');
}

interface ThemeContextValue {
  theme: ThemePreset;
  switchTheme: (preset: ThemePreset) => void;
  customColors: CustomThemeColors;
  updateCustomColors: (colors: Partial<CustomThemeColors>) => void;
  setBackgroundImage: (dataUrl: string) => void;
  clearBackgroundImage: () => void;
  setCustomChickenImage: (dataUrl: string) => void;
  clearCustomChickenImage: () => void;
  setSidebarLogoMode: (mode: SidebarLogoMode) => void;
  setCustomSidebarImage: (dataUrl: string) => void;
  clearCustomSidebarImage: () => void;
  getAntdThemeConfig: () => {
    primaryColor: string;
    primaryHover: string;
    bgCard: string;
    bgCardHover: string;
    borderColor: string;
    textColor: string;
    textPlaceholder: string;
    headerBg: string;
  };
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setTheme] = useState<ThemePreset>(() => {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === 'colorful') {
      localStorage.setItem(THEME_KEY, 'parchment');
      return 'parchment';
    }
    if (saved === 'eyeCare') {
      localStorage.setItem(THEME_KEY, 'oceanBlue');
      return 'oceanBlue';
    }
    if (saved === 'forestGreen') {
      localStorage.setItem(THEME_KEY, 'mintGreen');
      return 'mintGreen';
    }
    return (saved as ThemePreset) || 'oceanBlue';
  });

  const [customColors, setCustomColorsState] = useState<CustomThemeColors>(() => {
    try {
      const saved = localStorage.getItem(CUSTOM_COLORS_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        return { ...defaultCustomColors, ...parsed };
      }
      return defaultCustomColors;
    } catch {
      return defaultCustomColors;
    }
  });

  useEffect(() => {
    localStorage.setItem(THEME_KEY, theme);
    document.documentElement.className = presetClassNames[theme];

    if (theme === 'custom') {
      applyCustomThemeCSS(customColors);
    } else {
      clearCustomThemeCSS();
      if (customColors.backgroundImage) {
        applyBackgroundImage(customColors);
      }
    }
  }, [theme, customColors]);

  const switchTheme = useCallback((preset: ThemePreset) => {
    setTheme(preset);
  }, []);

  const updateCustomColors = useCallback((colors: Partial<CustomThemeColors>) => {
    setCustomColorsState(prev => {
      const next = { ...prev, ...colors };

      if (colors.primary !== undefined) {
        const auto = generateAutoColors(next.primary);
        Object.assign(next, auto);
      }

      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const setBackgroundImage = useCallback((dataUrl: string) => {
    setCustomColorsState(prev => {
      const next = { ...prev, backgroundImage: dataUrl };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const clearBackgroundImage = useCallback(() => {
    setCustomColorsState(prev => {
      const next = { ...prev, backgroundImage: '' };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const setCustomChickenImage = useCallback((dataUrl: string) => {
    setCustomColorsState(prev => {
      const next = { ...prev, customChickenImage: dataUrl };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const clearCustomChickenImage = useCallback(() => {
    setCustomColorsState(prev => {
      const next = { ...prev, customChickenImage: '' };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const setSidebarLogoMode = useCallback((mode: SidebarLogoMode) => {
    setCustomColorsState(prev => {
      const next = { ...prev, sidebarLogoMode: mode };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const setCustomSidebarImage = useCallback((dataUrl: string) => {
    setCustomColorsState(prev => {
      const next = { ...prev, customSidebarImage: dataUrl };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const clearCustomSidebarImage = useCallback(() => {
    setCustomColorsState(prev => {
      const next = { ...prev, customSidebarImage: '' };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const getAntdThemeConfig = useCallback(() => {
    const isMintGreen = theme === 'mintGreen';
    const isParchment = theme === 'parchment';
    const isCustom = theme === 'custom';

    let primaryColor = '#2563EB';
    let primaryHover = '#3B82F6';
    let bgCard = '#1e293b';
    let bgCardHover = '#334155';
    let borderColor = 'rgba(59, 130, 246, 0.12)';
    let textColor = '#f1f5f9';
    let textPlaceholder = '#64748b';
    let headerBg = '#1e293b';

    if (isParchment) {
      primaryColor = '#8b6914'; primaryHover = '#a67c1a';
      bgCard = '#3d3225'; bgCardHover = '#4a3d2e';
      borderColor = '#4a3d2e'; textColor = '#f0e6d3';
      textPlaceholder = '#8a7d6b'; headerBg = '#2d2418';
    } else if (isMintGreen) {
      primaryColor = '#E8A5B0'; primaryHover = '#F0B8C2';
      bgCard = '#4a2d33'; bgCardHover = '#5a3a41';
      borderColor = 'rgba(232, 165, 176, 0.25)'; textColor = '#faeeec';
      textPlaceholder = '#d8b0b5'; headerBg = '#3a2429';
    } else if (isCustom) {
      primaryColor = customColors.primary;
      primaryHover = lighten(customColors.primary, 0.15);
      bgCard = customColors.bgCard;
      bgCardHover = lighten(customColors.bgCard, 0.08);
      borderColor = lighten(customColors.bgCard, 0.12);
      textColor = customColors.textPrimary;
      textPlaceholder = darken(customColors.textSecondary, 0.3);
      headerBg = customColors.bgSecondary;
    }

    return {
      primaryColor, primaryHover, bgCard, bgCardHover,
      borderColor, textColor, textPlaceholder, headerBg,
    };
  }, [theme, customColors]);

  const value: ThemeContextValue = {
    theme,
    switchTheme,
    customColors,
    updateCustomColors,
    setBackgroundImage,
    clearBackgroundImage,
    setCustomChickenImage,
    clearCustomChickenImage,
    setSidebarLogoMode,
    setCustomSidebarImage,
    clearCustomSidebarImage,
    getAntdThemeConfig,
  };

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return ctx;
}
