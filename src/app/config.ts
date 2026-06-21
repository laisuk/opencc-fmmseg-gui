export type ZhoConfig =
    | "s2t"
    | "s2tw"
    | "s2twp"
    | "s2hk"
    | "s2hkp"
    | "t2s"
    | "t2tw"
    | "t2twp"
    | "t2hk"
    | "tw2s"
    | "tw2sp"
    | "tw2t"
    | "tw2tp"
    | "hk2s"
    | "hk2sp"
    | "hk2t"
    | "jp2t"
    | "t2jp";

export type TextCode = 0 | 1 | 2; // 1: zh-Hant, 2: zh-Hans, else: others

const MANUAL_CONFIGS: readonly ZhoConfig[] = [
    "s2t", "s2tw", "s2twp", "s2hk", "s2hkp",
    "t2s", "t2tw", "t2twp", "t2hk",
    "tw2s", "tw2sp", "tw2t", "tw2tp",
    "hk2s", "hk2sp", "hk2t",
    "jp2t", "t2jp",
] as const;

function asZhoConfig(v: string): ZhoConfig {
    if ((MANUAL_CONFIGS as readonly string[]).includes(v)) return v as ZhoConfig;
    return "s2t"; // fallback
}

// Matrix:
// S→T  Std  -    => s2t
// S→T  TW   Off  => s2tw
// S→T  TW   On   => s2twp
// S→T  HK   Off  => s2hk
// S→T  HK   On   => s2hkp
// T→S  Std  -    => t2s
// T→S  TW   Off  => tw2s
// T→S  TW   On   => tw2sp
// T→S  HK   Off  => hk2s
// T→S  HK   On   => hk2sp
const CONFIG_MATRIX = {
    s2t: {
        std: {off: "s2t", on: "s2t"},
        tw: {off: "s2tw", on: "s2twp"},
        hk: {off: "s2hk", on: "s2hkp"},
    },
    t2s: {
        std: {off: "t2s", on: "t2s"},
        tw: {off: "tw2s", on: "tw2sp"},
        hk: {off: "hk2s", on: "hk2sp"},
    },
} as const satisfies Record<
    "s2t" | "t2s",
    Record<"std" | "tw" | "hk", Record<"off" | "on", ZhoConfig>>
>;

export function getCurrentConfigFromUi(ui: {
    rbManual: HTMLInputElement
    selectConfig: HTMLSelectElement
    rbT2s: HTMLInputElement
    rbStd: HTMLInputElement
    rbZhHk: HTMLInputElement
    rbZhTw: HTMLInputElement
    cbRegionalTerms: HTMLInputElement
}): ZhoConfig {
    if (ui.rbManual.checked)
        return asZhoConfig(ui.selectConfig.value);

    const direction = ui.rbT2s.checked ? "t2s" : "s2t";
    const region = ui.rbStd.checked ? "std" : ui.rbZhHk.checked ? "hk" : "tw";
    const phrase = ui.cbRegionalTerms.checked ? "on" : "off";

    return CONFIG_MATRIX[direction][region][phrase];
}