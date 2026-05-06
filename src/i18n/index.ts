import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import zh from './zh.json';
import en from './en.json';

const savedLanguage = localStorage.getItem('svl-language');

i18n
  .use(initReactI18next)
  .init({
    resources: {
      zh: { translation: zh },
      en: { translation: en },
    },
    lng: savedLanguage || 'zh',
    fallbackLng: 'zh',
    interpolation: {
      escapeValue: false,
    },
  })
  .catch((err) => {
    console.error('[i18n] Failed to initialize i18n:', err);
  });

export default i18n;
