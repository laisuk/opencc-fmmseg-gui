// src/i18n/initUiLanguage.ts

import { mustGetEl } from "../dom/refs";
import { applyLocale } from "./applyLocale";
import { getUiLanguage, setUiLanguage, UiLanguage } from "./index";

export function initUiLanguage(): void {
    const selectUiLanguage = mustGetEl<HTMLSelectElement>("select-ui-language");

    const lang = getUiLanguage();
    selectUiLanguage.value = lang;

    applyLocale();

    selectUiLanguage.addEventListener("change", () => {
        const next = selectUiLanguage.value as UiLanguage;
        setUiLanguage(next);
        applyLocale();
    });
}