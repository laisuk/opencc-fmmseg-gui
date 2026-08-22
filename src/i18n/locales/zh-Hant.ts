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
        openFileTitle: "開啟檔案",
        compareTitle: "在輸出編輯器中醒目標示轉換後的詞語",
    },

    labels: {
        others: "Others（其它）",
        zhHans: "zh-Hans（簡體）",
        zhHant: "zh-Hant（正體）",
        japanese: "Japanese（日文）",
        chars: "Chars",
    },

    batch: {
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
        overwriteOutput: "覆寫現有輸出檔案（批次模式）",
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
    dialogQuoteValidation: {
        warningTitle: "驗證警告",
        passedTitle: "對話引號驗證通過",
        goToFirstSuspiciousLine: "跳至第一處可疑行",
        close: "關閉",
        more: (count: number) => `...還有 ${count} 項。`,
    },
} as const;
