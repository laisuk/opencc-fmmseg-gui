// src/i18n/locales/zh-Hans.ts

export const zhHans = {
    tabs: {
        convert: "Convert（转换）",
        batchConvert: "Batch Convert（批量转换）",
        settings: "Settings（设置）",
        about: "About（关于）",
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

        // Norm Compat button: text + tooltip separated for future flexibility
        normCompatText: "≡",
        normCompatTitle: "规范化 CJK 兼容汉字 (Compatibility Ideographs)",

        // DeTofu
        detofuText: "豆",
        detofuTitle: "替换豆腐风险字符，并标示所有替换位置 (Tofu block replacement)",

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

        deTofuLevel: "DeTofu Level（去豆腐等级）",
        saveTarget: "Save Target（保存目标）",
    },
} as const;