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
    mustGetEl<HTMLElement>("tab-about")
        .querySelector(".tab-label")!
        .textContent = s.tabs.about;

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

    // ---------- Norm Compat ----------
    const btnNormCompat = mustGetEl<HTMLButtonElement>("norm-compat");
    btnNormCompat.textContent = s.buttons.normCompatText;
    btnNormCompat.title = s.buttons.normCompatTitle;

    // ---------- DeTofu ----------
    const btnDeTofu = mustGetEl<HTMLButtonElement>("detofu");
    btnDeTofu.textContent = s.buttons.detofuText;
    btnDeTofu.title = s.buttons.detofuTitle;

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

    mustGetEl<HTMLElement>("cb-regional-text").textContent = s.config.regionalTerms;
    mustGetEl<HTMLElement>("cb-punctuation-text").textContent = s.config.punctuation;

    // ---------- Settings language selector ----------
    const uiLangLabelText = document.getElementById("ui-language-label-text");
    if (uiLangLabelText) uiLangLabelText.textContent = s.settings.uiLanguage;

    const optHans = document.getElementById("ui-lang-opt-hans");
    if (optHans) optHans.textContent = s.settings.uiLanguageHans;

    const optHant = document.getElementById("ui-lang-opt-hant");
    if (optHant) optHant.textContent = s.settings.uiLanguageHant;

    // ---------- Theme mode ----------
    const themeLabel = document.getElementById("theme-mode-label-text");
    if (themeLabel) themeLabel.textContent = s.settings.themeMode;

    const optSystem = document.getElementById("theme-opt-system");
    if (optSystem) optSystem.textContent = s.settings.themeSystem;

    const optLight = document.getElementById("theme-opt-light");
    if (optLight) optLight.textContent = s.settings.themeLight;

    const optDark = document.getElementById("theme-opt-dark");
    if (optDark) optDark.textContent = s.settings.themeDark;

    // ---------- DeTofu Level ----------
    const deTofuLabel = document.getElementById("detofu-level-label-text");
    if (deTofuLabel) deTofuLabel.textContent = s.settings.deTofuLevel;

    // ---------- Save Target ----------
    const saveTargetLabel = document.getElementById("save-target-label-text");
    if (saveTargetLabel) saveTargetLabel.textContent = s.settings.saveTarget;
}