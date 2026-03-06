export type ZhoConfig =
    | "s2t"
    | "s2tw"
    | "s2twp"
    | "s2hk"
    | "t2s"
    | "t2tw"
    | "t2twp"
    | "t2hk"
    | "tw2s"
    | "tw2sp"
    | "tw2t"
    | "tw2tp"
    | "hk2s"
    | "hk2t"
    | "jp2t"
    | "t2jp";

export type TextCode = 0 | 1 | 2; // 1: zh-Hant, 2: zh-Hans, else: others

const MANUAL_CONFIGS: readonly ZhoConfig[] = [
    "s2t", "s2tw", "s2twp", "s2hk", "t2s", "t2tw", "t2twp", "t2hk",
    "tw2s", "tw2sp", "tw2t", "tw2tp", "hk2s", "hk2t", "jp2t", "t2jp",
] as const;

function asZhoConfig(v: string): ZhoConfig {
    if ((MANUAL_CONFIGS as readonly string[]).includes(v)) return v as ZhoConfig;
    return "s2t"; // fallback
}

export function getCurrentConfigFromUi(ui: {
    rbManual: HTMLInputElement
    selectConfig: HTMLSelectElement
    rbT2s: HTMLInputElement
    rbStd: HTMLInputElement
    rbZhHk: HTMLInputElement
    rbZhTw: HTMLInputElement
    cbZhTwP: HTMLInputElement
}): ZhoConfig {
    if (ui.rbManual.checked) return asZhoConfig(ui.selectConfig.value);

    // Your original nested logic kept as-is (just typed)
    return ui.rbT2s.checked
        ? (ui.rbStd.checked
            ? "t2s"
            : ui.rbZhHk.checked
                ? "hk2s"
                : ui.rbZhTw.checked
                    ? (ui.cbZhTwP.checked ? "tw2sp" : "tw2s")
                    : "t2s")
        : (ui.rbStd.checked
            ? "s2t"
            : ui.rbZhHk.checked
                ? "s2hk"
                : ui.rbZhTw.checked
                    ? (ui.cbZhTwP.checked ? "s2twp" : "s2tw")
                    : "s2t");
}
