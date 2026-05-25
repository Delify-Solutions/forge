// SPDX-License-Identifier: AGPL-3.0-or-later
//
// i18next bootstrap. Two languages on day one (en, vi). Default is English;
// we deliberately do not auto-detect from `navigator` so a Vietnamese-locale
// macOS still lands on English on first run. The wizard's welcome step lets
// the user switch, and the choice persists in localStorage from then on.

import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';
import vi from './locales/vi.json';

export const SUPPORTED_LANGUAGES = [
    { code: 'en', label: 'English' },
    { code: 'vi', label: 'Tiếng Việt' },
] as const;

export type LanguageCode = (typeof SUPPORTED_LANGUAGES)[number]['code'];

export const STORAGE_KEY = 'delify-forge.language';

void i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources: {
            en: { translation: en },
            vi: { translation: vi },
        },
        fallbackLng: 'en',
        supportedLngs: SUPPORTED_LANGUAGES.map((l) => l.code),
        interpolation: {
            escapeValue: false,
        },
        detection: {
            order: ['localStorage'],
            lookupLocalStorage: STORAGE_KEY,
            caches: ['localStorage'],
        },
    });

export function setLanguage(code: LanguageCode) {
    void i18n.changeLanguage(code);
}

export default i18n;
