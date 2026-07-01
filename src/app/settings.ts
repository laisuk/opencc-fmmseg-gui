import {mustGetEl} from "../dom/refs";

export type SaveTarget = "source" | "destination";

export type AppSettingsSnapshot = {
    convertFilename: boolean;
    addPageHeader: boolean;
    compactPdf: boolean;
    autoReflow: boolean;
    overwriteOutput: boolean;
    enableEditorLog: boolean;
    punctuation: boolean;
    customHeadingRegexText: string;
    customHeadingRegex: string | null;
    deTofuLevel: string;
    saveTarget: SaveTarget;
};

type SettingsElements = {
    cbConvertFilename: HTMLInputElement;
    cbAddPageHeader: HTMLInputElement;
    cbCompactPdf: HTMLInputElement;
    cbAutoReflow: HTMLInputElement;
    cbOverwriteOutput: HTMLInputElement;
    cbEnableEditorLog: HTMLInputElement;
    cbPunctuation: HTMLInputElement;
    tbHeadingRegex: HTMLInputElement | null;
    headingRegexStatus: HTMLSpanElement | null;
    headingRegexHint: HTMLSpanElement | null;
    selectDeTofuLevel: HTMLSelectElement;
    selectSaveTarget: HTMLSelectElement;
};

const STORAGE_KEYS = {
    convertFilename: "convertFilename",
    addPageHeader: "addPageHeader",
    compactPdf: "compactPdf",
    autoReflow: "autoReflow",
    overwriteOutput: "cbOverwriteOutput",
    enableEditorLog: "cbEnableEditorLog",
    punctuation: "cbPunctuation",
    customHeadingRegex: "custom_heading_regex",
    deTofuLevel: "deToFuLevel",
    saveTarget: "saveTarget",
} as const;

const DEFAULT_HINT =
    "Used to detect headings (optional). Invalid patterns will be highlighted.";

const state = {
    convertFilename: false,
    addPageHeader: false,
    compactPdf: false,
    autoReflow: true,
    overwriteOutput: true,
    enableEditorLog: true,
    punctuation: true,
    customHeadingRegexText: "",
    customHeadingRegexRe: null as RegExp | null,
    deTofuLevel: "ExtB",
    saveTarget: "destination" as SaveTarget,
};

let elements: SettingsElements | null = null;

function readBoolean(key: string, defaultValue: boolean): boolean {
    const raw = localStorage.getItem(key);
    if (raw === null) return defaultValue;
    return raw === "true";
}

function readSaveTarget(key: string, defaultValue: SaveTarget): SaveTarget {
    const raw = localStorage.getItem(key);
    return raw === "source" || raw === "destination" ? raw : defaultValue;
}

function debounce<T extends (...args: any[]) => void>(fn: T, ms = 120) {
    let t: number | undefined;
    return (...args: Parameters<T>) => {
        if (t) window.clearTimeout(t);
        t = window.setTimeout(() => fn(...args), ms);
    };
}

function simplifyRegexError(msg: string): string {
    const i = msg.lastIndexOf(":");
    return i >= 0 ? msg.slice(i + 1).trim() : msg;
}

function syncStateFromCheckboxes(): void {
    if (!elements) return;
    state.convertFilename = elements.cbConvertFilename.checked;
    state.addPageHeader = elements.cbAddPageHeader.checked;
    state.compactPdf = elements.cbCompactPdf.checked;
    state.autoReflow = elements.cbAutoReflow.checked;
    state.overwriteOutput = elements.cbOverwriteOutput.checked;
    state.enableEditorLog = elements.cbEnableEditorLog.checked;
    state.punctuation = elements.cbPunctuation.checked;
}

function applyStateToCheckboxes(): void {
    if (!elements) return;
    elements.cbConvertFilename.checked = state.convertFilename;
    elements.cbAddPageHeader.checked = state.addPageHeader;
    elements.cbCompactPdf.checked = state.compactPdf;
    elements.cbAutoReflow.checked = state.autoReflow;
    elements.cbOverwriteOutput.checked = state.overwriteOutput;
    elements.cbEnableEditorLog.checked = state.enableEditorLog;
    elements.cbPunctuation.checked = state.punctuation;
}

function persistCheckboxState(): void {
    localStorage.setItem(STORAGE_KEYS.convertFilename, String(state.convertFilename));
    localStorage.setItem(STORAGE_KEYS.addPageHeader, String(state.addPageHeader));
    localStorage.setItem(STORAGE_KEYS.compactPdf, String(state.compactPdf));
    localStorage.setItem(STORAGE_KEYS.autoReflow, String(state.autoReflow));
    localStorage.setItem(STORAGE_KEYS.overwriteOutput, String(state.overwriteOutput));
    localStorage.setItem(STORAGE_KEYS.enableEditorLog, String(state.enableEditorLog));
    localStorage.setItem(STORAGE_KEYS.punctuation, String(state.punctuation));
}

function renderHeadingRegexState(): void {
    if (!elements) return;
    const tb = elements.tbHeadingRegex;
    const status = elements.headingRegexStatus;
    const hint = elements.headingRegexHint;
    if (!tb || !status || !hint) return;

    if (!state.customHeadingRegexText) {
        tb.classList.remove("is-invalid");
        status.textContent = "Off";
        status.classList.remove("ok", "bad");
        hint.textContent = DEFAULT_HINT;
        return;
    }

    if (state.customHeadingRegexRe) {
        tb.classList.remove("is-invalid");
        status.textContent = "OK";
        hint.textContent = DEFAULT_HINT;
        status.classList.remove("bad");
        status.classList.add("ok");
        return;
    }

    tb.classList.add("is-invalid");
    status.textContent = "Invalid";
    status.classList.remove("ok");
    status.classList.add("bad");
}

function setHeadingRegex(text: string, persist = true): void {
    state.customHeadingRegexText = text.trim();

    if (!state.customHeadingRegexText) {
        state.customHeadingRegexRe = null;
        if (persist) localStorage.setItem(STORAGE_KEYS.customHeadingRegex, "");
        renderHeadingRegexState();
        return;
    }

    try {
        state.customHeadingRegexRe = new RegExp(state.customHeadingRegexText, "u");
        if (persist) {
            localStorage.setItem(STORAGE_KEYS.customHeadingRegex, state.customHeadingRegexText);
        }
        renderHeadingRegexState();
    } catch (err) {
        state.customHeadingRegexRe = null;
        if (persist) {
            localStorage.setItem(STORAGE_KEYS.customHeadingRegex, state.customHeadingRegexText);
        }

        if (!elements?.headingRegexHint) return;
        const raw = err instanceof Error ? err.message : "Invalid";
        const msg = simplifyRegexError(raw);
        elements.headingRegexHint.textContent = `Regex error: ${msg}`;
        renderHeadingRegexState();
    }
}

export function initAppSettings(): void {
    elements = {
        cbConvertFilename: mustGetEl<HTMLInputElement>("cbConvertFilename"),
        cbAddPageHeader: mustGetEl<HTMLInputElement>("cbAddPageHeader"),
        cbCompactPdf: mustGetEl<HTMLInputElement>("cbCompactPdf"),
        cbAutoReflow: mustGetEl<HTMLInputElement>("cbAutoReflow"),
        cbOverwriteOutput: mustGetEl<HTMLInputElement>("cbOverwriteOutput"),
        cbEnableEditorLog: mustGetEl<HTMLInputElement>("cbEnableEditorLog"),
        cbPunctuation: mustGetEl<HTMLInputElement>("cb_punctuation"),
        tbHeadingRegex: document.getElementById("tbHeadingRegex") as HTMLInputElement | null,
        headingRegexStatus: document.getElementById("headingRegexStatus") as HTMLSpanElement | null,
        headingRegexHint: document.getElementById("headingRegexHint") as HTMLSpanElement | null,
        selectDeTofuLevel: mustGetEl<HTMLSelectElement>("select-detofu-level"),
        selectSaveTarget: mustGetEl<HTMLSelectElement>("select-save-target"),
    };

    state.convertFilename = readBoolean(STORAGE_KEYS.convertFilename, false);
    state.addPageHeader = readBoolean(STORAGE_KEYS.addPageHeader, false);
    state.compactPdf = readBoolean(STORAGE_KEYS.compactPdf, false);
    state.autoReflow = readBoolean(STORAGE_KEYS.autoReflow, true);
    state.overwriteOutput = readBoolean(STORAGE_KEYS.overwriteOutput, true);
    state.enableEditorLog = readBoolean(STORAGE_KEYS.enableEditorLog, true);
    state.punctuation = readBoolean(STORAGE_KEYS.punctuation, true);

    applyStateToCheckboxes();

    [
        elements.cbConvertFilename,
        elements.cbAddPageHeader,
        elements.cbCompactPdf,
        elements.cbAutoReflow,
        elements.cbOverwriteOutput,
        elements.cbEnableEditorLog,
        elements.cbPunctuation,
    ].forEach((cb) => {
        cb.addEventListener("change", () => {
            syncStateFromCheckboxes();
            persistCheckboxState();
        });
    });

    state.deTofuLevel =
        localStorage.getItem(STORAGE_KEYS.deTofuLevel) ?? "ExtB";

    elements.selectDeTofuLevel.value = state.deTofuLevel;

    elements.selectDeTofuLevel.addEventListener("change", () => {
        state.deTofuLevel = elements!.selectDeTofuLevel.value;
        localStorage.setItem(STORAGE_KEYS.deTofuLevel, state.deTofuLevel);
    });

    state.saveTarget = readSaveTarget(STORAGE_KEYS.saveTarget, "destination");
    elements.selectSaveTarget.value = state.saveTarget;

    elements.selectSaveTarget.addEventListener("change", () => {
        state.saveTarget =
            elements!.selectSaveTarget.value === "source" ? "source" : "destination";
        localStorage.setItem(STORAGE_KEYS.saveTarget, state.saveTarget);
    });

    const savedHeadingRegex = localStorage.getItem(STORAGE_KEYS.customHeadingRegex) ?? "";
    if (elements.tbHeadingRegex) {
        elements.tbHeadingRegex.value = savedHeadingRegex;
        elements.tbHeadingRegex.addEventListener(
            "input",
            debounce(() => {
                if (!elements?.tbHeadingRegex) return;
                setHeadingRegex(elements.tbHeadingRegex.value, true);
            }, 120),
        );
    }

    setHeadingRegex(savedHeadingRegex, false);
}

export function getCustomHeadingRegex(): string | null {
    const text = state.customHeadingRegexText.trim();
    if (!text || !state.customHeadingRegexRe) return null;
    return text;
}

export function isEditorLogEnabled(): boolean {
    return state.enableEditorLog;
}

export function getAppSettings(): AppSettingsSnapshot {
    return {
        convertFilename: state.convertFilename,
        addPageHeader: state.addPageHeader,
        compactPdf: state.compactPdf,
        autoReflow: state.autoReflow,
        overwriteOutput: state.overwriteOutput,
        enableEditorLog: state.enableEditorLog,
        punctuation: state.punctuation,
        customHeadingRegexText: state.customHeadingRegexText,
        customHeadingRegex: getCustomHeadingRegex(),
        deTofuLevel: state.deTofuLevel,
        saveTarget: state.saveTarget,
    };
}
