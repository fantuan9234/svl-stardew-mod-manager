import { useState, useEffect, useCallback } from 'react';

export type ThemePreset = 'colorful' | 'eyeCare' | 'custom';

export interface CustomThemeColors {
  primary: string;
  accent: string;
  bgPrimary: string;
  bgSecondary: string;
  bgCard: string;
  textPrimary: string;
  textSecondary: string;
}

const THEME_KEY = 'svl-theme-preset';
const CUSTOM_COLORS_KEY = 'svl-custom-theme-colors';

const presetClassNames: Record<ThemePreset, string> = {
  colorful: '',
  eyeCare: 'eye-care',
  custom: 'theme-custom',
};

const defaultCustomColors: CustomThemeColors = {
  primary: '#8b6914',
  accent: '#6b9e3a',
  bgPrimary: '#1f140d',
  bgSecondary: '#2a1d14',
  bgCard: '#2a1d14',
  textPrimary: '#f0e6d3',
  textSecondary: '#c4b89a',
};

function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? { r: parseInt(result[1], 16), g: parseInt(result[2], 16), b: parseInt(result[3], 16) }
    : null;
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
}

export function useTheme() {
  const [theme, setTheme] = useState<ThemePreset>(() => {
    const saved = localStorage.getItem(THEME_KEY);
    return (saved as ThemePreset) || 'colorful';
  });

  const [customColors, setCustomColorsState] = useState<CustomThemeColors>(() => {
    try {
      const saved = localStorage.getItem(CUSTOM_COLORS_KEY);
      return saved ? JSON.parse(saved) : defaultCustomColors;
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
    }
  }, [theme, customColors]);

  const switchTheme = useCallback((preset: ThemePreset) => {
    setTheme(preset);
  }, []);

  const updateCustomColors = useCallback((colors: Partial<CustomThemeColors>) => {
    setCustomColorsState(prev => {
      const next = { ...prev, ...colors };
      localStorage.setItem(CUSTOM_COLORS_KEY, JSON.stringify(next));
      return next;
    });
  }, []);

  const getAntdThemeConfig = useCallback(() => {
    const isEyeCare = theme === 'eyeCare';
    const isCustom = theme === 'custom';

    let primaryColor = '#8b6914';
    let primaryHover = '#a67c1a';
    let bgCard = '#2a1d14';
    let bgCardHover = '#342618';
    let borderColor = 'rgba(255, 190, 90, 0.12)';
    let textColor = '#f0e6d3';
    let textPlaceholder = '#8a7d6b';
    let headerBg = '#2a1d14';

    if (isEyeCare) {
      primaryColor = '#5b8a72'; primaryHover = '#6b9b82';
      bgCard = '#1c2825'; bgCardHover = '#243230';
      borderColor = 'rgba(140, 210, 170, 0.12)'; textColor = '#d4ddd8';
      textPlaceholder = '#7a8f82'; headerBg = '#182220';
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

  return {
    theme,
    switchTheme,
    customColors,
    updateCustomColors,
    getAntdThemeConfig,
  };
}
