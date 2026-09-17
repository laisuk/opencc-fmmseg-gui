// src/i18n/locales/zh-Hant.ts

export const zhHant = {
    tabs: {
        convert: "轉換",
        batchConvert: "批次轉換",
        dictionary: "字典",
        settings: "設定",
        about: "關於",
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
        openFile: "開啟檔案",
        convert: "轉換",
        saveFile: "儲存檔案",
        paste: "貼上",
        copy: "複製",
        clear: "AC",
        compare: "比對",

        reflowText: "↳↰",
        reflowTitle: "重排提取的 CJK 文本",

        // Norm Compat button: text + tooltip separated for future flexibility
        normCompatText: "≡",
        normCompatTitle: "正規化 CJK 相容漢字 (Compatibility Ideographs)",

        // Norm Dialog Quotes button: tooltip separated for future flexibility
        // Text is not needed for SVG icon
        normDialogQuotesTitle: "正規化 CJK 對話引號 (Dialog Quotes)",

        // Validate Dialog Quotes button: tooltip separated for future flexibility
        // Text is not needed for SVG icon
        validateDialogQuotesTitle: "檢查 CJK 對話引號 (Dialog Quotes)",

        // DeTofu
        detofuText: "豆",
        detofuTitle: "取代豆腐風險字元，並標示所有替換位置 (Tofu block replacement)",

        pasteTitle: "將剪貼簿內容貼到編輯器",
        copyTitle: "複製輸出內容",
        clearTitle: "清空編輯器內容",
        openFileTitle: "開啟檔案至編輯器",
        saveFileTitle: "將目標編輯器內容儲存到檔案（目標在設定中指定）",
        compareTitle: "在輸出編輯器中醒目標示轉換後的詞語",
    },

    labels: {
        others: "Others（其它）",
        zhHans: "zh-Hans（簡體）",
        zhHant: "zh-Hant（正體）",
        japanese: "Japanese（日文）",
        chars: "字元",
    },

    batch: {
        addTitle: "新增檔案至清單",
        removeTitle: "從清單中移除所選檔案",
        clearTitle: "清空檔案清單",
        browseTitle: "選擇輸出資料夾",
        openOutDirTitle: "開啟輸出資料夾",
        clearLogTitle: "清空記錄或預覽",
        filesTitle: "檔案",
        logPreviewTitle: "記錄 / 預覽",
        idleStatus: "閒置",
        logPlaceholder: "批次輸出 / 記錄...",
        outputLabel: "輸出",
        outputPlaceholder: "選擇批次輸出資料夾...",
        convertBatch: "批次轉換",
    },

    settings: {
        convertFilename: "轉換檔名",
        addPageHeader: "新增頁首（PDF）",
        compactPdf: "精簡 PDF 文字",
        autoReflow: "自動重排（PDF）",
        ignoreUntrustedPdfText: "忽略不可信的 PDF 文字（進階 PDF）",
        overwriteOutput: "覆寫現有輸出檔案（批次模式）",
        autoDetectCjkEncoding: "自動偵測純文字的舊式 CJK 編碼（批次模式）",
        enableEditorLog: "啟用編輯器記錄",
        uiLanguage: "UI Language（介面語言）",
        uiLanguageEnglish: "English",
        uiLanguageHans: "簡體中文",
        uiLanguageHant: "正體中文",

        themeMode: "Theme Mode（主題模式）",
        themeSystem: "System（跟隨系統）",
        themeLight: "Light（淺色）",
        themeDark: "Dark（深色）",

        editorFont: "Editor Font（編輯器字型）",
        fontSize: "Font Size（字級）",

        deTofuLevel: "DeTofu Level（去豆腐等級）",
        saveTarget: "Save Target（儲存目標）",
        extendUnicodeCompat: "擴展 CJK 正規化的 Unicode 相容範圍",
    },
    dictionary: {
        title: "字典工具",
        generationHeading: "📖 字典產生",
        baseDirectory: "基礎字典目錄",
        baseDirectoryPlaceholder: "選擇或輸入詞典目錄...",
        baseDirectoryHint: "必須包含完整所需的 dicts/*.txt 辭典檔案。",
        outputDirectory: "輸出目錄",
        outputDirectoryPlaceholder: "選擇或輸入輸出目錄...",
        browse: "瀏覽",
        clearPathHint: "清除路徑",
        generateZstd: "產生 ZSTD",
        generateCbor: "產生 CBOR",
        generateJson: "產生 JSON",
        generateWithCustom: "套用自訂字典",
        customSlots: "自訂字典槽位",
        slot: "槽位",
        mode: "模式",
        dictionaryFile: "字典檔案",
        filePlaceholder: "字典檔案路徑",
        remove: "移除",
        removeHint: "移除目前這列",
        add: "新增自訂字典",
        apply: "套用到目前轉換器",
        applyStartup: "啟動時套用到轉換器",
        empty: "尚未設定自訂字典。",
        defaultRuntime: "預設字典",
        customRuntime: "自訂字典（{count}）",
        validating: "正在驗證字典來源…",
        invalidSource: "請選擇有效的基礎字典目錄以啟用產生。",
        generating: "正在產生 {format} 字典…",
        generated: "字典產生成功：{path}",
        applying: "正在套用自訂字典…",
        applied: "自訂字典已套用到目前轉換器。",
        reset: "目前已使用預設字典。",
        startupFailed: "無法套用已儲存的自訂字典；仍使用預設字典：{error}",
        error: "字典操作失敗：{error}",
    },

    dialogQuoteValidation: {
        warningTitle: "驗證警告",
        passedTitle: "對話引號驗證通過",
        goToFirstSuspiciousLine: "跳至第一處可疑行",
        close: "關閉",
        more: (count: number) => `...還有 ${count} 項。`,
    },

    runtime: {
        clipboardEmpty: "剪貼簿為空",
        clipboardPasted: "已貼上剪貼簿內容",
        noTextCopied: "沒有可複製的文字",
        outputCopied: "已複製輸出內容",
        errorCopyText: "複製文字失敗：{error}",

        converting: "正在轉換...",
        conversionComplete: "轉換完成（{config}）",
        selectionConverted: "所選文字轉換完成（{config}）",
        convertFailed: "轉換失敗：{error}",

        reflowing: "正在重排...",
        noReflowText: "沒有文本需要重排",
        reflowComplete: "重排完成",
        selectionReflowComplete: "所選文字重排完成",
        reflowFailed: "重排失敗：{error}",

        normalizingCompat: "正在正規化 CJK 相容字元...",
        noCompatText: "沒有文本需要正規化",
        noCompatFound: "未發現 CJK 相容字元",
        normalizationComplete: "正規化完成",
        selectionNormalizationComplete: "所選文字正規化完成",
        normalizationFailed: "正規化失敗：{error}",

        normalizingDialogQuotes: "正在正規化 CJK 對話引號...",
        noText: "編輯器文本為空",

        validatingDialogQuotes: "正在驗證 CJK 對話引號...",
        validationCompleted: "驗證完成",
        validationFailed: "驗證失敗：{error}",

        runningDeTofu: "正在執行 DeTofu...",
        noDeTofuNeeded: "未發現豆腐塊風險字元",
        deTofuComplete: "DeTofu 完成",
        deTofuFailed: "DeTofu 失敗：{error}",

        sourceCleared: "編輯器來源文字已清除",
        destinationCleared: "編輯器目標文字已清除",

        opening: "正在開啟...",
        noFileSelected: "未選擇檔案",
        fileOpened: "檔案已開啟：{filePath}",
        openFailed: "開啟失敗：{error}",
        fileDropped: "檔案已拖放：{filePath}",
        dropFailed: "拖放失敗：{error}",

        selectSavePath: "請選擇儲存路徑 {target} ...",
        fileSaved: "檔案已儲存：{filePath}",
        fileNotSaved: "檔案儲存失敗:{error}",

        noFilesSelected: "未選擇檔案",
        outputDirectoryNotSet: "未設定輸出目錄",
        batchRunning: "正在批次轉換...",
        batchCompleted: "批次轉換完成（{config}）",
        batchWithError: "批次轉換完成，但有錯誤（{config}）",
        batchFailed: "批次轉換失敗",
        noOutputFolderSelected: "未選擇輸出資料夾",
        outputFolderOpened: "輸出資料夾已開啟：{path}",
        outputFolderError: "開啟輸出資料夾失敗：{error}",

        logCleared: "日誌已清除",
        addedFiles: "已新增 {count} 個檔案",

        comparing: "正在比對來源文字和轉換後的文字...",
        compareCompleted: "比對完成",
        compareDisabled: "比對已停用",

        reloadingEncoding: "正在以 {encoding} 編碼重新載入...",
        reloadedAutoEncoding: "已使用自動編碼偵測重新載入檔案",
        reloadedEncoding: "已以 {encoding} 編碼重新載入檔案",
        reloadFailed: "重新載入失敗：{error}",
    },

    batchLogs: {
        globalInfo: "批次處理全域資訊",
        files: "檔案",
        output: "輸出",
        config: "設定",
        punctuation: "標點符號",
        convertFilename: "轉換檔案名稱",
        reflowPdf: "PDF 自動重排",
        autoDetectCjkEncoding: "自動偵測 CJK 編碼",
        completedSuccessfully: "成功轉換完畢",
        completedWithErrors: "轉換完畢，但有錯誤",
        failed: "批次處理失敗",
    },
} as const;
