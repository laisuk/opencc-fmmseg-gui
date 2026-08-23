// src/i18n/locales/zh-Hans.ts

export const zhHans = {
    tabs: {
        convert: "转换",
        batchConvert: "批量转换",
        dictionary: "词典",
        settings: "设置",
        about: "关于",
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
        convertFilename: "转换文件名",
        addPageHeader: "添加页眉（PDF）",
        compactPdf: "紧凑化 PDF 文本",
        autoReflow: "自动重排（PDF）",
        overwriteOutput: "覆盖现有输出文件（批量模式）",
        enableEditorLog: "启用编辑器日志",
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
        extendUnicodeCompat: "扩展 CJK 规范化的 Unicode 兼容范围",
    },
    dictionary: {
        title: "词典生成",
        generationHeading: "词典生成",
        baseDirectory: "基础词典目录",
        outputDirectory: "输出目录",
        browse: "浏览",
        generateZstd: "生成 ZSTD",
        generateCbor: "生成 CBOR",
        generateJson: "生成 JSON",
        generateWithCustom: "应用自定义词典",
        customSlots: "自定义词典槽位",
        slot: "槽位",
        mode: "模式",
        dictionaryFile: "词典文件",
        filePlaceholder: "词典文件路径",
        remove: "移除",
        add: "添加自定义词典",
        apply: "应用到当前转换器",
        applyStartup: "启动时应用到转换器",
        empty: "未配置自定义词典。",
        defaultRuntime: "默认词典",
        customRuntime: "自定义词典（{count}）",
        validating: "正在验证词典源…",
        invalidSource: "请选择有效的基础词典目录以启用生成。",
        generating: "正在生成 {format} 词典…",
        generated: "词典生成成功：{path}",
        applying: "正在应用自定义词典…",
        applied: "自定义词典已应用到当前转换器。",
        reset: "当前已使用默认词典。",
        startupFailed: "无法应用已保存的自定义词典；仍使用默认词典：{error}",
        error: "词典操作失败：{error}",
    },

    dialogQuoteValidation: {
        warningTitle: "验证警告",
        passedTitle: "对话引号验证通过",
        goToFirstSuspiciousLine: "跳转到第一处可疑行",
        close: "关闭",
        more: (count: number) => `...还有 ${count} 项。`,
    },
} as const;
