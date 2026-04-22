// src/i18n/applyLocale.ts

import {mustGetEl} from "../dom/refs";
import {getLocale} from "./index";

export function applyLocale(): void {
    const s = getLocale();

    // ---------- Tabs ----------
    mustGetEl<HTMLElement>("tab-convert")
        .querySelector(".tab-label")!
        .textContent = s.tabs.convert;
    mustGetEl<HTMLElement>("tab-batch")
        .querySelector(".tab-label")!
        .textContent = s.tabs.batchConvert;
    mustGetEl<HTMLElement>("tab-settings")
        .querySelector(".tab-label")!
        .textContent = s.tabs.settings;

    // ---------- Main action buttons ----------
    const btnOpenFile = mustGetEl<HTMLButtonElement>("open-file");
    btnOpenFile.querySelector("span")!.textContent = s.buttons.openFile;

    const btnConvert = mustGetEl<HTMLButtonElement>("convert");
    btnConvert.querySelector("span")!.textContent = s.buttons.convert;

    const btnSaveFile = mustGetEl<HTMLButtonElement>("save-file");
    btnSaveFile.querySelector("span")!.textContent = s.buttons.saveFile;

    mustGetEl<HTMLButtonElement>("clear-source").textContent = s.buttons.clear;
    mustGetEl<HTMLButtonElement>("clear-destination").textContent = s.buttons.clear;

    // ---------- Reflow ----------
    const btnReflow = mustGetEl<HTMLButtonElement>("reflow");
    btnReflow.textContent = s.buttons.reflowText;
    btnReflow.title = s.buttons.reflowTitle;

    // ---------- Paste / Copy ----------
    const btnPaste = mustGetEl<HTMLButtonElement>("paste");
    const btnCopy = mustGetEl<HTMLButtonElement>("copy");

    const pasteSpan = btnPaste.querySelector("span");
    if (pasteSpan) pasteSpan.textContent = s.buttons.paste;
    btnPaste.title = s.buttons.pasteTitle;

    const copySpan = btnCopy.querySelector("span");
    if (copySpan) copySpan.textContent = s.buttons.copy;
    btnCopy.title = s.buttons.copyTitle;

    // ---------- Compare ----------
    const compareLabel = mustGetEl<HTMLElement>("compare-label");
    compareLabel.title = s.buttons.compareTitle;

    const compareText = mustGetEl<HTMLElement>("compare-text");
    compareText.textContent = s.buttons.compare;

    // ---------- Config labels ----------
    mustGetEl<HTMLElement>("rb-st-text").textContent = s.config.s2t;
    mustGetEl<HTMLElement>("rb-ts-text").textContent = s.config.t2s;
    mustGetEl<HTMLElement>("rb-manual-text").textContent = s.config.manual;

    mustGetEl<HTMLElement>("rb-std-text").textContent = s.config.standard;
    mustGetEl<HTMLElement>("rb-zhtw-text").textContent = s.config.zhTw;
    mustGetEl<HTMLElement>("rb-zhhk-text").textContent = s.config.zhHk;

    mustGetEl<HTMLElement>("cb-zhtwp-text").textContent = s.config.zhTwIdioms;
    mustGetEl<HTMLElement>("cb-punctuation-text").textContent = s.config.punctuation;

    // ---------- Settings language selector ----------
    const uiLangLabel = document.getElementById("ui-language-label");
    if (uiLangLabel) uiLangLabel.textContent = s.settings.uiLanguage;

    const optHans = document.getElementById("ui-lang-opt-hans");
    if (optHans) optHans.textContent = s.settings.uiLanguageHans;

    const optHant = document.getElementById("ui-lang-opt-hant");
    if (optHant) optHant.textContent = s.settings.uiLanguageHant;
}