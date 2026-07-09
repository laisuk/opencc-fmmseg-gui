// src/i18n/index.ts

import {en} from "./locales/en";
import {zhHans} from "./locales/zh-Hans";
import {zhHant} from "./locales/zh-Hant";

export type UiLanguage = "en" | "zh-Hans" | "zh-Hant";

const STORAGE_KEY = "uiLanguage";

const locales = {
    en,
    "zh-Hans": zhHans,
    "zh-Hant": zhHant,
} as const;

let currentLanguage: UiLanguage = "en";

export function getUiLanguage(): UiLanguage {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "en" || saved === "zh-Hans" || saved === "zh-Hant") {
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

export type ThemeMode = "system" | "light" | "dark";

const STORAGE_THEME_MODE = "themeMode";

function applyThemeMode(mode: ThemeMode): void {
    document.documentElement.dataset.theme = mode;
    document.documentElement.style.colorScheme =
        mode === "system" ? "light dark" : mode;

    localStorage.setItem(STORAGE_THEME_MODE, mode);
}

export function initThemeMode(): void {
    const saved = localStorage.getItem(STORAGE_THEME_MODE);

    const mode: ThemeMode =
        saved === "light" || saved === "dark" || saved === "system"
            ? saved
            : "system";

    applyThemeMode(mode);

    const select = document.getElementById("select-theme-mode") as HTMLSelectElement | null;
    if (select) {
        select.value = mode;
        select.addEventListener("change", () => {
            applyThemeMode(select.value as ThemeMode);
        });
    }
}