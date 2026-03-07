// src/i18n/index.ts

import { zhHans } from "./locales/zh-Hans";
import { zhHant } from "./locales/zh-Hant";

export type UiLanguage = "zh-Hans" | "zh-Hant";

const STORAGE_KEY = "uiLanguage";

const locales = {
    "zh-Hans": zhHans,
    "zh-Hant": zhHant,
} as const;

let currentLanguage: UiLanguage = "zh-Hans";

export function getUiLanguage(): UiLanguage {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "zh-Hans" || saved === "zh-Hant") {
        currentLanguage = saved;
    }
    return currentLanguage;
}

export function setUiLanguage(lang: UiLanguage): void {
    currentLanguage = lang;
    localStorage.setItem(STORAGE_KEY, lang);
}

export function getLocale() {
    return locales[currentLanguage];
}