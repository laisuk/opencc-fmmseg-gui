const STORAGE_FONT_FAMILY = "editorFontFamily";
const STORAGE_FONT_SIZE = "editorFontSize";

const DEFAULT_FONT_FAMILY = "default";
const DEFAULT_FONT_SIZE = 17;

function quoteFont(font: string): string {
    return /\s/.test(font) ? `"${font}"` : font;
}

function getEditorFontStack(fontFamily: string): string | null {
    const font = fontFamily.trim();

    if (!font || font === DEFAULT_FONT_FAMILY) {
        return null;
    }

    switch (font) {
        case "monospace":
            return "ui-monospace, SFMono-Regular, Consolas, monospace";

        case "sans-serif":
            return `"Noto Sans SC", "Noto Sans TC", "Source Han Sans SC", "Microsoft YaHei", "Microsoft JHengHei", "PingFang SC", sans-serif`;

        case "serif":
            return `"Noto Serif SC", "Source Han Serif SC", "Songti SC", "SimSun", serif`;

        case "Consolas":
        case "JetBrains Mono":
        case "Fira Code":
        case "Noto Sans Mono":
            return `${quoteFont(font)}, ui-monospace, SFMono-Regular, Consolas, monospace`;

        case "Noto Sans SC":
        case "Noto Sans TC":
        case "Source Han Sans SC":
        case "Microsoft YaHei":
        case "Microsoft JHengHei":
        case "PingFang SC":
        case "HeiTi":
        case "KaiTi":
        case "FangSong":
        case "WenQuanYi Micro Hei":
            return `${quoteFont(font)}, "Noto Sans SC", "Noto Sans TC", "Source Han Sans SC", "Microsoft YaHei", "Microsoft JHengHei", "PingFang SC", sans-serif`;

        case "Noto Serif SC":
        case "Source Han Serif SC":
        case "Songti SC":
        case "SimSun":
        case "SimHei":
            return `${quoteFont(font)}, "Noto Serif SC", "Source Han Serif SC", "Songti SC", "SimSun", serif`;

        default:
            return `${quoteFont(font)}, ui-monospace, SFMono-Regular, Consolas, monospace`;
    }
}

function parseFontSize(value: string | null): number {
    const parsed = Number.parseInt(value ?? "", 10);
    if (!Number.isFinite(parsed)) {
        return DEFAULT_FONT_SIZE;
    }
    return Math.min(40, Math.max(10, parsed));
}

export function applyEditorFontSettings(fontFamily: string, fontSizePx: number): void {
    const root = document.documentElement;
    const fontStack = getEditorFontStack(fontFamily);

    if (fontStack) {
        root.style.setProperty("--editor-font", fontStack);
    } else {
        root.style.removeProperty("--editor-font");
    }

    root.style.setProperty("--editor-font-size", `${fontSizePx / 16}rem`);
}

export function loadEditorFontSettings(): {
    fontFamily: string;
    fontSize: number;
} {
    return {
        fontFamily: localStorage.getItem(STORAGE_FONT_FAMILY) || DEFAULT_FONT_FAMILY,
        fontSize: parseFontSize(localStorage.getItem(STORAGE_FONT_SIZE)),
    };
}

export function saveEditorFontSettings(fontFamily: string, fontSizePx: number): void {
    localStorage.setItem(STORAGE_FONT_FAMILY, fontFamily);
    localStorage.setItem(STORAGE_FONT_SIZE, String(fontSizePx));
}

export function initEditorFontControls(
    fontSelect: HTMLSelectElement,
    fontSizeInput: HTMLInputElement,
): void {
    const {fontFamily, fontSize} = loadEditorFontSettings();

    fontSelect.value = fontFamily;
    fontSizeInput.value = String(fontSize);

    applyEditorFontSettings(fontFamily, fontSize);

    fontSelect.addEventListener("change", () => {
        const nextFamily = fontSelect.value || DEFAULT_FONT_FAMILY;
        const nextSize = parseFontSize(fontSizeInput.value);

        saveEditorFontSettings(nextFamily, nextSize);
        applyEditorFontSettings(nextFamily, nextSize);
    });

    fontSizeInput.addEventListener("input", () => {
        const nextFamily = fontSelect.value || DEFAULT_FONT_FAMILY;
        const nextSize = parseFontSize(fontSizeInput.value);

        fontSizeInput.value = String(nextSize);
        saveEditorFontSettings(nextFamily, nextSize);
        applyEditorFontSettings(nextFamily, nextSize);
    });
}