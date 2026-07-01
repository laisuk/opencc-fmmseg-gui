// src/features/compare.ts
//
// Compare overlay for the destination CodeMirror editor.
//
// Design goals:
// 1. Unicode-safe compare (supports astral / non-BMP characters like 𬴂)
// 2. Robust with CodeMirror document replacement
// 3. Effect-based decoration updates (Plan 3)
// 4. Keep output text fully selectable/copyable; compare is only a visual overlay
//
// Notes:
// - Comparison is done by Unicode code points, not UTF-16 code units.
// - CodeMirror decorations still require UTF-16 offsets, so we maintain a
//   code-point-index -> UTF-16-offset mapping.
// - This is intentionally a positional compare, not an LCS diff. That keeps it
//   fast and scalable for very large texts.

import {
    RangeSetBuilder,
    StateEffect,
    StateField,
} from "@codemirror/state";
import {
    Decoration,
    DecorationSet,
    EditorView,
} from "@codemirror/view";

export type CompareFeature = {
    compareField: StateField<DecorationSet>;
    apply: () => void;
    applyTexts: (
        src: string,
        dst: string,
        tooltipText?: (original: string, converted: string) => string
    ) => void;
    clear: () => void;
};

type CodePointInfo = {
    // Unicode-safe character list (code points, not UTF-16 code units).
    chars: string[];

    // offsets[i] = UTF-16 start offset of chars[i]
    // offsets[chars.length] = UTF-16 end offset of the full string
    offsets: number[];
};

/**
 * Convert a string into:
 * - a Unicode-safe code-point array
 * - a code-point-index -> UTF-16-offset table
 *
 * Why not use plain text[i] / text.length?
 * Because JS strings are UTF-16-based:
 * astral characters like 𬴂 occupy 2 code units, but visually are 1 character.
 *
 * Using `for...of` iterates by Unicode code point, which is what we want.
 */
function toCodePointsWithOffsets(text: string): CodePointInfo {
    const chars: string[] = [];
    const offsets: number[] = [];

    let utf16Offset = 0;

    for (const ch of text) {
        offsets.push(utf16Offset);
        chars.push(ch);
        utf16Offset += ch.length; // BMP = 1, astral = 2
    }

    offsets.push(utf16Offset);

    return {chars, offsets};
}

export function createCompareFeature(opts: {
    getSourceText: () => string;
    getDestText: () => string;
    isEnabled: () => boolean;

    // Permanent Plan-3 path:
    // caller provides direct dispatch access to destination editor
    dispatchToDest: (spec: Parameters<EditorView["dispatch"]>[0]) => void;

    tooltipText?: (original: string, converted: string) => string;
}): CompareFeature {
    const tooltipText =
        opts.tooltipText ??
        ((orig, conv) => `Changed: ${orig} → ${conv}`);

    /**
     * Effect used to replace the entire compare decoration set.
     *
     * We intentionally update decorations through an explicit StateEffect
     * instead of mutating outer state + calling editorRight.dispatch({}).
     *
     * This is more robust, especially during dev/hot reload.
     */
    const setCompareDecorations = StateEffect.define<DecorationSet>();

    const compareField = StateField.define<DecorationSet>({
        create() {
            return Decoration.none;
        },

        /**
         * Update gate for compare decorations.
         *
         * Why this matters:
         * When the destination document changes (convert, reflow, clear, etc.),
         * old decoration ranges may become invalid for the new document.
         *
         * Example bad case:
         * - old doc length = 4
         * - decoration ends at position 4
         * - new doc length = 0
         * => stale range now points out of bounds
         *
         * `value.map(tr.changes)` remaps the existing field value through the
         * document transaction, which prevents:
         *   "Position x is out of range for changeset of length y"
         */
        update(value, tr) {
            // Gate 1:
            // Always map existing decorations through document changes first.
            value = value.map(tr.changes);

            // Gate 2:
            // If this transaction carries an explicit compare-decoration update,
            // replace the current value with the effect payload.
            for (const e of tr.effects) {
                if (e.is(setCompareDecorations)) {
                    value = e.value;
                }
            }

            return value;
        },

        provide: (f) => EditorView.decorations.from(f),
    });

    /**
     * Build compare decorations for the destination editor.
     *
     * Strategy:
     * - Compare source/destination by Unicode code point.
     * - Group consecutive differences into one block.
     * - Apply decoration only to the destination range.
     * - Tooltip shows source phrase -> converted phrase.
     *
     * This is intentionally NOT an LCS diff:
     * - much faster
     * - scales better to millions of characters
     * - ideal for OpenCC-style conversion where source/destination are usually
     *   still strongly position-aligned
     */
    function buildCompareDecorations(src: string, dst: string, makeTooltipText = tooltipText): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>();

        const srcInfo = toCodePointsWithOffsets(src);
        const dstInfo = toCodePointsWithOffsets(dst);

        const srcChars = srcInfo.chars;
        const dstChars = dstInfo.chars;

        // Only compare the overlapping region.
        // Extra tail content is ignored by current design.
        const len = Math.min(srcChars.length, dstChars.length);

        let start = -1;

        for (let i = 0; i < len; i++) {
            if (srcChars[i] !== dstChars[i]) {
                if (start === -1) {
                    start = i;
                }
            } else if (start !== -1) {
                addBlock(start, i);
                start = -1;
            }
        }

        if (start !== -1) {
            addBlock(start, len);
        }

        function addBlock(startCp: number, endCp: number) {
            const from = dstInfo.offsets[startCp];
            const to = dstInfo.offsets[endCp];

            // Safety gate: skip empty/invalid ranges
            if (from >= to) return;

            const original = srcChars.slice(startCp, endCp).join("");
            const converted = dstChars.slice(startCp, endCp).join("");

            const mark = Decoration.mark({
                class: "cm-diff-converted",
                attributes: {
                    title: makeTooltipText(original, converted),
                    // "data-tooltip": tooltipText(original, converted),
                },
            });

            builder.add(from, to, mark);
        }

        return builder.finish();
    }

    /**
     * Replace compare decorations with "none".
     *
     * This should be called before:
     * - replacing destination text
     * - clearing destination text
     * - reflow output (if you do not want compare after reflow)
     * - checkbox off
     */
    function clear() {
        opts.dispatchToDest({
            effects: setCompareDecorations.of(Decoration.none),
        });
    }

    /**
     * Recompute compare decorations from current source/destination text.
     *
     * Caller normally does:
     * - compare.clear()
     * - set destination text
     * - if compare enabled => compare.apply()
     */
    function apply() {
        if (!opts.isEnabled()) {
            clear();
            return;
        }

        const src = opts.getSourceText();
        const dst = opts.getDestText();

        // Small fast-exit guards:
        // keep behavior predictable and avoid unnecessary work
        if (!src || !dst) {
            clear();
            return;
        }

        if (src === dst) {
            clear();
            return;
        }

        const decorations = buildCompareDecorations(src, dst);

        opts.dispatchToDest({
            effects: setCompareDecorations.of(decorations),
        });
    }

    /**
     * Build and apply decorations from two arbitrary texts.
     *
     * Unlike {@link apply}, this method does not read editor contents.
     * The caller supplies both source and destination strings directly,
     * making it suitable for temporary comparisons such as DeTofu.
     *
     * If either text is empty or both texts are identical, existing
     * decorations are cleared instead.
     *
     * @param src Source/original text.
     * @param dst Destination/processed text.
     * @param customTooltipText Optional tooltip formatter for each highlighted
     * difference. If omitted, the default compare tooltip is used.
     */
    function applyTexts(
        src: string,
        dst: string,
        customTooltipText?: (original: string, converted: string) => string,
    ) {
        if (!src || !dst || src === dst) {
            clear();
            return;
        }

        const decorations = buildCompareDecorations(
            src,
            dst,
            customTooltipText ?? tooltipText,
        );

        opts.dispatchToDest({
            effects: setCompareDecorations.of(decorations),
        });
    }

    return {
        compareField,
        apply,
        applyTexts,
        clear,
    };
}