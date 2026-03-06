// src/features/compare.ts
import { RangeSetBuilder, StateField } from "@codemirror/state";
import { Decoration, DecorationSet, EditorView } from "@codemirror/view";

export type CompareFeature = {
    compareField: StateField<DecorationSet>;
    apply: () => void;
    clear: () => void;
};

type CodePointInfo = {
    chars: string[];
    offsets: number[]; // code point index -> UTF-16 offset
};

function toCodePointsWithOffsets(text: string): CodePointInfo {
    const chars = Array.from(text);
    const offsets: number[] = new Array(chars.length + 1);

    let utf16Offset = 0;
    for (let i = 0; i < chars.length; i++) {
        offsets[i] = utf16Offset;
        utf16Offset += chars[i].length; // 1 for BMP, 2 for astral
    }
    offsets[chars.length] = utf16Offset;

    return { chars, offsets };
}

export function createCompareFeature(opts: {
    getSourceText: () => string;
    getDestText: () => string;
    isEnabled: () => boolean;
    onRefreshDest: () => void;
    tooltipText?: (original: string, converted: string) => string;
}): CompareFeature {
    let compareDecorations: DecorationSet = Decoration.none;

    const tooltipText =
        opts.tooltipText ??
        ((orig, conv) => `原文: ${orig} → 转换: ${conv}`);

    const compareField = StateField.define<DecorationSet>({
        create() {
            return Decoration.none;
        },
        update() {
            return compareDecorations;
        },
        provide: (f) => EditorView.decorations.from(f),
    });

    function buildCompareDecorations(src: string, dst: string): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>();

        const srcInfo = toCodePointsWithOffsets(src);
        const dstInfo = toCodePointsWithOffsets(dst);

        const srcChars = srcInfo.chars;
        const dstChars = dstInfo.chars;

        const len = Math.min(srcChars.length, dstChars.length);

        let start = -1;

        for (let i = 0; i < len; i++) {
            if (srcChars[i] !== dstChars[i]) {
                if (start === -1) start = i;
            } else if (start !== -1) {
                addBlock(start, i);
                start = -1;
            }
        }

        if (start !== -1) {
            addBlock(start, len);
        }

        function addBlock(s: number, e: number) {
            const orig = srcChars.slice(s, e).join("");
            const conv = dstChars.slice(s, e).join("");

            // Decorations apply to destination editor,
            // so use destination UTF-16 offsets.
            const from = dstInfo.offsets[s];
            const to = dstInfo.offsets[e];

            if (from >= to) return;

            const mark = Decoration.mark({
                class: "cm-diff-converted",
                attributes: { title: tooltipText(orig, conv) },
            });

            builder.add(from, to, mark);
        }

        return builder.finish();
    }

    function clear() {
        compareDecorations = Decoration.none;
        opts.onRefreshDest();
    }

    function apply() {
        if (!opts.isEnabled()) {
            clear();
            return;
        }

        const src = opts.getSourceText();
        const dst = opts.getDestText();

        compareDecorations = buildCompareDecorations(src, dst);
        opts.onRefreshDest();
    }

    return { compareField, apply, clear };
}