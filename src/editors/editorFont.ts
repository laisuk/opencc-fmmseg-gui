const STORAGE_FONT_FAMILY = "editorFontFamily";
const STORAGE_FONT_SIZE = "editorFontSize";

const DEFAULT_FONT_FAMILY = "default";
const DEFAULT_FONT_SIZE = 17;

function quoteFont(font: string): string {
    // Returns quoted string if spaces exist, otherwise returns original
    return /\s/.test(font) ? `"${font}"` : font;
}

function getEditorFontStack(fontFamily: string): string | null {
    const font = fontFamily.trim();

    if (!font || font === DEFAULT_FONT_FAMILY) {
        return null;
    }

    const monoStack = 'ui-monospace, SFMono-Regular, Consolas, monospace';
    const sansStack = '"PingFang SC", "Sarasa Gothic SC", "Microsoft YaHei", "Noto Sans SC", sans-serif';
    const serifStack = '"Source Han Serif SC", "Noto Serif SC", "Songti SC", "SimSun", serif';

    switch (font) {
        case "monospace":
            return monoStack;

        case "sans-serif":
            return sansStack;

        case "serif":
            return serifStack;

        // --- Monospace Group ---
        case "Consolas":
        case "JetBrains Mono":
        case "Fira Code":
        case "Noto Sans Mono":
        case "Sarasa Mono SC":
            return `${quoteFont(font)}, ${monoStack}`;

        // --- Sans-Serif / UI Group ---
        case "Sarasa Gothic SC":
        case "Noto Sans SC":
        case "Noto Sans TC":
        case "Source Han Sans SC":
        case "Microsoft YaHei":
        case "Microsoft JHengHei":
        case "PingFang SC":
        case "HeiTi":
        case "SimHei":
        case "WenQuanYi Micro Hei":
            return `${quoteFont(font)}, ${sansStack}`;

        // --- Serif / Reading Group ---
        case "Noto Serif SC":
        case "Source Han Serif SC":
        case "Songti SC":
        case "SimSun":
        case "NSimSun":
        case "MingLiU":
        case "PMingLiU":
        case "KaiTi":
        case "FangSong":
            return `${quoteFont(font)}, ${serifStack}`;

        default:
            return `${quoteFont(font)}, ${monoStack}`;
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