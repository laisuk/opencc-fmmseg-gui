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
        compare: "比对",

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
        clearTitle: "清空编辑器内容",
        openFileTitle: "打开文件至编辑器",
        saveFileTitle: "将目标编辑器内容保存到文件（目标在设置中指定）",
        compareTitle: "在输出编辑器中高亮转换后的词语",
    },

    labels: {
        others: "Others（其它）",
        zhHans: "zh-Hans（简体）",
        zhHant: "zh-Hant（繁体）",
        japanese: "Japanese（日文）",
        seal: "Seal（篆书）",
        chars: "字符",
    },

    batch: {
        addTitle: "添加文件到列表",
        removeTitle: "从列表中移除所选文件",
        clearTitle: "清空文件列表",
        browseTitle: "选择输出文件夹",
        openOutDirTitle: "打开输出文件夹",
        clearLogTitle: "清空日志或预览",
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
        ignoreUntrustedPdfText: "忽略不可信的 PDF 文本（高级 PDF）",
        overwriteOutput: "覆盖现有输出文件（批量模式）",
        autoDetectCjkEncoding: "自动检测纯文本的旧式 CJK 编码（批量模式）",
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
        title: "词典工具",
        generationHeading: "📖 词典生成",
        baseDirectory: "基础词典目录",
        baseDirectoryPlaceholder: "选择或输入词典目录...",
        baseDirectoryHint: "必须包含完整所需的 dicts/*.txt 字典文件。",
        outputDirectory: "输出目录",
        outputDirectoryPlaceholder: "选择或输入输出目录...",
        browse: "浏览",
        clearPathHint: "清除路径",
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
        removeHint: "移除当前行",
        add: "添加自定义词典",
        apply: "应用到当前转换器",
        applyStartup: "启动时应用到转换器",
        empty: "未配置自定义词典。",
        defaultRuntime: "默认词典",
        customRuntime: "自定义词典（{count}）",
        customDictionaryEmptyHint: "未配置自定义词典时应用，将把当前转换器重置为默认基础词典。",
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

    runtime: {
        clipboardEmpty: "剪贴板为空",
        clipboardPasted: "已粘贴剪贴板内容",
        noTextCopied: "没有可复制的文本",
        outputCopied: "已复制输出内容",
        errorCopyText: "复制文本失败：{error}",

        converting: "正在转换...",
        conversionComplete: "转换完成（{config}）",
        selectionConverted: "所选文本转换完成（{config}）",
        convertFailed: "转换失败：{error}",

        reflowing: "正在重排...",
        noReflowText: "没有文本需要重排",
        reflowComplete: "重排完成",
        selectionReflowComplete: "所选文本重排完成",
        reflowFailed: "重排失败：{error}",

        normalizingCompat: "正在规范化 CJK 兼容字符...",
        noCompatText: "没有文本需要规范化",
        noCompatFound: "未发现 CJK 兼容字符",
        normalizationComplete: "规范化完成",
        selectionNormalizationComplete: "所选文本规范化完成",
        normalizationFailed: "规范化失败：{error}",

        normalizingDialogQuotes: "正在规范化 CJK 对话引号...",
        noSourceText: "来源文本为空",
        normalizationDialogQuotesComplete: "对话引号规范化完成",
        selectionNormalizationDialogQuotesComplete: "所选文本对话引号规范化完成",

        validatingDialogQuotes: "正在验证 CJK 对话引号...",
        validationCompleted: "验证完成",
        validationFailed: "验证失败：{error}",

        runningDeTofu: "正在执行 DeTofu...",
        noDeTofuNeeded: "未发现豆腐块风险字符",
        deTofuComplete: "DeTofu 完成",
        deTofuFailed: "DeTofu 失败：{error}",

        sourceCleared: "编辑器源文本已清除",
        destinationCleared: "编辑器目标文本已清除",

        opening: "正在打开...",
        noFileSelected: "未选择文件",
        fileOpened: "文件已打开：{filePath}",
        openFailed: "打开失败：{error}",
        fileDropped: "文件已拖放：{filePath}",
        dropFailed: "拖放失败：{error}",

        selectSavePath: "请选择保存路径 {target} ...",
        fileSaved: "文件已保存：{filePath}",
        fileNotSaved: "文件保存失败:{error}",

        noFilesSelected: "未选择文件",
        outputDirectoryNotSet: "未设置输出目录",
        batchRunning: "正在批量转换...",
        batchCompleted: "批量转换完成（{config}）",
        batchWithError: "批量转换完成，但有错误（{config}）",
        batchFailed: "批量转换失败",
        noOutputFolderSelected: "未选择输出文件夹",
        outputFolderOpened: "输出文件夹已打开：{path}",
        outputFolderError: "打开输出文件夹失败：{error}",

        logCleared: "日志已清除",
        addedFiles: "已添加 {count} 个文件",

        comparing: "正在比对源文本和转换后的文本...",
        compareCompleted: "比对完成",
        compareDisabled: "比对已禁用",

        reloadingEncoding: "正在以 {encoding} 编码重新加载...",
        reloadedAutoEncoding: "已使用自动编码检测重新加载文件",
        reloadedEncoding: "已以 {encoding} 编码重新加载文件",
        reloadFailed: "重新加载失败：{error}",
    },

    batchLogs: {
        globalInfo: "批处理全局信息",
        files: "文件",
        output: "输出",
        config: "配置",
        punctuation: "标点符号",
        convertFilename: "转换文件名",
        reflowPdf: "PDF 自动重排",
        autoDetectCjkEncoding: "自动检测 CJK 编码",
        completedSuccessfully: "成功转换完毕",
        completedWithErrors: "转换完毕，但有错误",
        failed: "批处理失败",
    },
} as const;
