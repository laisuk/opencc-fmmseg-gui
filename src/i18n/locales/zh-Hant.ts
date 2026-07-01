// src/i18n/locales/zh-Hant.ts

export const zhHant = {
    tabs: {
        convert: "Convert（轉換）",
        batchConvert: "Batch Convert（批次轉換）",
        settings: "Settings（設定）",
        about: "About（關於）",
    },

    config: {
        s2t: "zh-Hans（簡體） to zh-Hant（正體）",
        t2s: "zh-Hant（正體） to zh-Hans（簡體）",
        manual: "Manual（自定義）",
        standard: "General（通用簡繁）",
        zhTw: "ZH-TW（中台簡繁）",
        zhHk: "ZH-HK（中港簡繁）",
        regionalTerms: "Regional Terms（地區用語）",
        punctuation: "Punctuation（標點）",
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

        // Norm Compat button: text + tooltip separated for future flexibility
        normCompatText: "≡",
        normCompatTitle: "正規化 CJK 相容漢字 (Compatibility Ideographs)",

        // DeTofu
        detofuText: "豆",
        detofuTitle: "取代豆腐風險字元，並標示所有替換位置 (Tofu block replacement)",

        pasteTitle: "Paste clipboard to editor",
        copyTitle: "Copy output",
        compareTitle: "Highlight converted phrases in the output editor",
    },

    labels: {
        others: "Others（其它）",
        zhHans: "zh-Hans（簡體）",
        zhHant: "zh-Hant（正體）",
        japanese: "Japanese（日文）",
        chars: "Chars",
    },

    settings: {
        uiLanguage: "UI Language（介面語言）",
        uiLanguageHans: "簡體中文",
        uiLanguageHant: "正體中文",

        themeMode: "Theme Mode（主題模式）",
        themeSystem: "System（跟隨系統）",
        themeLight: "Light（淺色）",
        themeDark: "Dark（深色）",

        deTofuLevel: "DeTofu Level（去豆腐等級）",
        saveTarget: "Save Target（儲存目標）",
    },
} as const;