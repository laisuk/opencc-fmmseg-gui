// src/features/compare.ts
import { RangeSetBuilder, StateField } from "@codemirror/state"
import { Decoration, DecorationSet, EditorView } from "@codemirror/view"

export type CompareFeature = {
    compareField: StateField<DecorationSet>
    apply: () => void
    clear: () => void
}

export function createCompareFeature(opts: {
    getSourceText: () => string
    getDestText: () => string
    isEnabled: () => boolean
    onRefreshDest: () => void
    tooltipText?: (original: string, converted: string) => string
}): CompareFeature {
    let compareDecorations: DecorationSet = Decoration.none

    const tooltipText =
        opts.tooltipText ??
        ((orig, conv) => `原文: ${orig} → 转换: ${conv}`)

    const compareField = StateField.define<DecorationSet>({
        create() {
            return Decoration.none
        },
        update() {
            // Always use the latest computed decorations
            return compareDecorations
        },
        provide: (f) => EditorView.decorations.from(f),
    })

    function buildCompareDecorations(src: string, dst: string): DecorationSet {
        const builder = new RangeSetBuilder<Decoration>()
        const len = Math.min(src.length, dst.length)

        // Group consecutive diffs into a phrase block (better tooltip),
        // but still mark each diff range precisely.
        let start = -1
        for (let i = 0; i < len; i++) {
            if (src[i] !== dst[i]) {
                if (start === -1) start = i
            } else if (start !== -1) {
                addBlock(start, i)
                start = -1
            }
        }
        if (start !== -1) addBlock(start, len)

        function addBlock(s: number, e: number) {
            const orig = src.slice(s, e)
            const conv = dst.slice(s, e)

            const mark = Decoration.mark({
                class: "cm-diff-converted",
                attributes: { title: tooltipText(orig, conv) }, // native tooltip
            })

            builder.add(s, e, mark)
        }

        return builder.finish()
    }

    function clear() {
        compareDecorations = Decoration.none
        opts.onRefreshDest()
    }

    function apply() {
        if (!opts.isEnabled()) {
            clear()
            return
        }
        const src = opts.getSourceText()
        const dst = opts.getDestText()
        compareDecorations = buildCompareDecorations(src, dst)
        opts.onRefreshDest()
    }

    return { compareField, apply, clear }
}