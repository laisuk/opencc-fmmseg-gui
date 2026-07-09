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
        openFile: "打开文件",
        convert: "转换",
        saveFile: "保存文件",
        paste: "粘贴",
        copy: "复制",
        clear: "AC",
        compare: "对比",

        // Reflow button: text + tooltip separated for future flexibility
        reflowText: "↳↰",
        reflowTitle: "重排提取的 CJK 文本",

        // Norm Compat button: text + tooltip separated for future flexibility
        normCompatText: "≡",
        normCompatTitle: "规范化 CJK 兼容汉字 (Compatibility Ideographs)",

        // Norm Dialog Quotes button: tooltip separated for future flexibility
        // Text is not needed for SVG icon
        normDialogQuotesTitle: "规范化 CJK 对话引号 (Dialog Quotes)",

        // Validate Dialog Quotes button: tooltip separated for future flexibility
        // Text is not needed for SVG icon
        validateDialogQuotesTitle: "检查 CJK 对话引号 (Dialog Quotes)",

        // DeTofu
        detofuText: "豆",
        detofuTitle: "替换豆腐风险字符，并标示所有替换位置 (Tofu block replacement)",

        pasteTitle: "将剪贴板内容粘贴到编辑器",
        copyTitle: "复制输出内容",
        openFileTitle: "打开文件",
        compareTitle: "在输出编辑器中高亮转换后的词语",
    },

    labels: {
        others: "Others（其它）",
        zhHans: "zh-Hans（简体）",
        zhHant: "zh-Hant（繁体）",
        japanese: "Japanese（日文）",
        chars: "Chars",
    },

    batch: {
        filesTitle: "文件",
        logPreviewTitle: "日志 / 预览",
        idleStatus: "空闲",
        logPlaceholder: "批量输出 / 日志...",
        outputLabel: "输出",
        outputPlaceholder: "选择批量输出文件夹...",
        convertBatch: "批量转换",
    },

    settings: {
        uiLanguage: "UI Language（界面语言）",
        uiLanguageEnglish: "English",
        uiLanguageHans: "简体中文",
        uiLanguageHant: "繁體中文",

        themeMode: "Theme Mode（主题模式）",
        themeSystem: "System（跟随系统）",
        themeLight: "Light（浅色）",
        themeDark: "Dark（深色）",

        editorFont: "Editor Font（编辑器字体）",
        fontSize: "Font Size（字号）",

        deTofuLevel: "DeTofu Level（去豆腐等级）",
        saveTarget: "Save Target（保存目标）",
    },
    dialogQuoteValidation: {
        warningTitle: "验证警告",
        passedTitle: "对话引号验证通过",
        goToFirstSuspiciousLine: "跳转到第一处可疑行",
        close: "关闭",
        more: (count: number) => `...还有 ${count} 项。`,
    },
} as const;
