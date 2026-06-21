// src/i18n/locales/zh-Hans.ts

export const zhHans = {
    tabs: {
        convert: "Convert（轉換）",
        batchConvert: "Batch Convert（批次轉換）",
        settings: "Settings（設定）",
        about: "About（關於）",
    },

    config: {
        s2t: "zh-Hans（简体） to zh-Hant（繁体）",
        t2s: "zh-Hant（繁体） to zh-Hans（简体）",
        manual: "Manual（自定义）",
        standard: "General（通用简繁）",
        zhTw: "ZH-TW（中台简繁）",
        zhHk: "ZH-HK（中港简繁）",
        regionalTerms: "Regional Terms（地区用语）",
        punctuation: "Punctuation（标点）",
    },

    buttons: {
        openFile: "Open File",
        convert: "Convert",
        saveFile: "Save File",
        paste: "Paste",
        copy: "Copy",
        clear: "AC",
        compare: "Compare",

        // Reflow button: text + tooltip separated for future flexibility
        reflowText: "↳↰",
        reflowTitle: "Reflow extracted CJK text",

        pasteTitle: "Paste clipboard to editor",
        copyTitle: "Copy output",
        compareTitle: "Highlight converted phrases in the output editor",
    },

    labels: {
        others: "Others（其它）",
        zhHans: "zh-Hans（简体）",
        zhHant: "zh-Hant（繁体）",
        japanese: "Japanese（日文）",
        chars: "Chars",
    },

    settings: {
        uiLanguage: "UI Language（界面语言）",
        uiLanguageHans: "简体中文",
        uiLanguageHant: "繁體中文",

        themeMode: "Theme Mode（主题模式）",
        themeSystem: "System（跟随系统）",
        themeLight: "Light（浅色）",
        themeDark: "Dark（深色）",
    },
} as const;