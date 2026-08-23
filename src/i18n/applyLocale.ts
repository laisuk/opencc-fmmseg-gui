// src/i18n/applyLocale.ts

import {mustGetEl} from "../dom/refs";
import {applyQuoteValidationDialogLocale} from "../dialog";
import {applyDictionaryLocale} from "../features/dictionary/dictionary";
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
    mustGetEl<HTMLElement>("tab-dictionary")
        .querySelector(".tab-label")!
        .textContent = s.tabs.dictionary;
    mustGetEl<HTMLElement>("tab-about")
        .querySelector(".tab-label")!
        .textContent = s.tabs.about;

    // ---------- Main action buttons ----------
    const btnOpenFile = mustGetEl<HTMLButtonElement>("open-file");
    btnOpenFile.querySelector("span")!.textContent = s.buttons.openFile;
    btnOpenFile.title = s.buttons.openFileTitle;

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

    // ---------- Norm Dialog Quotes ----------
    const btnNormDialogQuotes = mustGetEl<HTMLButtonElement>("norm-dialog-quotes");
    btnNormDialogQuotes.title = s.buttons.normDialogQuotesTitle;

    // ---------- Validate Dialog Quotes ----------
    const btnValidateSourceDialogQuotes = mustGetEl<HTMLButtonElement>("validate-source-dialog-quotes");
    btnValidateSourceDialogQuotes.title = s.buttons.validateDialogQuotesTitle;
    const btnValidateDestinationDialogQuotes = mustGetEl<HTMLButtonElement>("validate-destination-dialog-quotes");
    btnValidateDestinationDialogQuotes.title = s.buttons.validateDialogQuotesTitle;

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

    // ---------- Batch Convert ----------
    const batchFilesTitle = document.getElementById("batch-files-title");
    if (batchFilesTitle) batchFilesTitle.textContent = s.batch.filesTitle;

    const batchLogTitle = document.getElementById("batch-log-title");
    if (batchLogTitle) batchLogTitle.textContent = s.batch.logPreviewTitle;

    const batchStatus = document.getElementById("batch-status");
    if (batchStatus?.dataset.statusKey === "idle") {
        batchStatus.textContent = s.batch.idleStatus;
    }

    const batchLog = document.getElementById("batch-log") as HTMLTextAreaElement | null;
    if (batchLog) batchLog.placeholder = s.batch.logPlaceholder;

    const batchOutputLabel = document.getElementById("batch-output-label");
    if (batchOutputLabel) batchOutputLabel.textContent = s.batch.outputLabel;

    const batchOutDir = document.getElementById("batch-outdir") as HTMLInputElement | null;
    if (batchOutDir) batchOutDir.placeholder = s.batch.outputPlaceholder;

    const batchRunText = document.getElementById("batch-run-text");
    if (batchRunText) batchRunText.textContent = s.batch.convertBatch;
    // ---------- Config labels ----------

    // ---------- Settings checkboxes ----------
    mustGetEl<HTMLElement>("convert-filename-text").textContent = s.settings.convertFilename;
    mustGetEl<HTMLElement>("add-page-header-text").textContent = s.settings.addPageHeader;
    mustGetEl<HTMLElement>("compact-pdf-text").textContent = s.settings.compactPdf;
    mustGetEl<HTMLElement>("auto-reflow-text").textContent = s.settings.autoReflow;
    mustGetEl<HTMLElement>("overwrite-output-text").textContent = s.settings.overwriteOutput;
    mustGetEl<HTMLElement>("enable-editor-log-text").textContent = s.settings.enableEditorLog;

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

    const optEn = document.getElementById("ui-lang-opt-en");
    if (optEn) optEn.textContent = s.settings.uiLanguageEnglish;

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

    // ---------- Editor Font ----------
    const editorFontLabel = document.getElementById("editor-font-label-text");
    if (editorFontLabel) editorFontLabel.textContent = s.settings.editorFont;

    const editorFontSizeLabel = document.getElementById("editor-font-size-label-text");
    if (editorFontSizeLabel) editorFontSizeLabel.textContent = s.settings.fontSize;
    // ---------- DeTofu Level ----------
    const deTofuLabel = document.getElementById("detofu-level-label-text");
    if (deTofuLabel) deTofuLabel.textContent = s.settings.deTofuLevel;

    // ---------- Save Target ----------
    const saveTargetLabel = document.getElementById("save-target-label-text");
    if (saveTargetLabel) saveTargetLabel.textContent = s.settings.saveTarget;

    const extendUnicodeCompatLabel = document.getElementById("extend-unicode-compat-text");
    if (extendUnicodeCompatLabel) {
        extendUnicodeCompatLabel.textContent = s.settings.extendUnicodeCompat;
    }

    applyQuoteValidationDialogLocale();
    applyDictionaryLocale();
}
