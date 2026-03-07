// src/i18n/runtimeLabels.ts

import { getLocale } from "./index";

export type RuntimeLabelKind =
    | "others"
    | "zhHans"
    | "zhHant"
    | "japanese";

export function getRuntimeLabel(kind: RuntimeLabelKind): string {
    const s = getLocale().labels;

    switch (kind) {
        case "others":
            return s.others;
        case "zhHans":
            return s.zhHans;
        case "zhHant":
            return s.zhHant;
        case "japanese":
            return s.japanese;
    }
}

export function formatCharCount(count: number): string {
    const s = getLocale().labels;
    return `[ ${count.toLocaleString()} ${s.chars} ]`;
}