// src/i18n/locales/en.ts

export const en = {
    tabs: {
        convert: "Convert",
        batchConvert: "Batch Convert",
        settings: "Settings",
        about: "About",
    },

    config: {
        s2t: "zh-Hans to zh-Hant",
        t2s: "zh-Hant to zh-Hans",
        manual: "Manual",
        standard: "General",
        zhTw: "ZH-TW",
        zhHk: "ZH-HK",
        regionalTerms: "Regional Terms",
        punctuation: "Punctuation",
    },

    buttons: {
        openFile: "Open File",
        convert: "Convert",
        saveFile: "Save File",
        paste: "Paste",
        copy: "Copy",
        clear: "AC",
        compare: "Compare",

        reflowText: "↳↰",
        reflowTitle: "Reflow extracted CJK text",

        normCompatText: "≡",
        normCompatTitle: "Normalize CJK compatibility ideographs",

        normDialogQuotesTitle: "Normalize CJK dialog quotes",
        validateDialogQuotesTitle: "Validate CJK dialog quotes",

        detofuText: "豆",
        detofuTitle: "Replace tofu-risk characters and highlight the replacements",

        pasteTitle: "Paste clipboard to editor",
        copyTitle: "Copy output",
        openFileTitle: "Open file",
        compareTitle: "Highlight converted phrases in the output editor",
    },

    labels: {
        others: "Others",
        zhHans: "zh-Hans",
        zhHant: "zh-Hant",
        japanese: "Japanese",
        chars: "Chars",
    },

    batch: {
        filesTitle: "Files",
        logPreviewTitle: "Log / Preview",
        idleStatus: "Idle",
        logPlaceholder: "Batch output / log...",
        outputLabel: "Output",
        outputPlaceholder: "Select batch output folder...",
        convertBatch: "Convert Batch",
    },

    settings: {
        uiLanguage: "UI Language",
        uiLanguageEnglish: "English",
        uiLanguageHans: "简体中文",
        uiLanguageHant: "繁體中文",

        themeMode: "Theme Mode",
        themeSystem: "System",
        themeLight: "Light",
        themeDark: "Dark",

        editorFont: "Editor Font",
        fontSize: "Font Size",

        deTofuLevel: "DeTofu Level",
        saveTarget: "Save Target",
    },

    dialogQuoteValidation: {
        warningTitle: "Validation Warning",
        passedTitle: "Dialog Quote Validation Passed",
        goToFirstSuspiciousLine: "Go To First Suspicious Line",
        close: "Close",
        more: (count: number) => `...and ${count} more.`,
    },
} as const;
