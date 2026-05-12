import {EditorView} from "@codemirror/view";

export function getEditorText(view: EditorView): string {
    return view.state.doc.toString();
}

export function setEditorText(view: EditorView, value: string, cursorAtEnd = false): void {
    view.dispatch({
        changes: {
            from: 0,
            to: view.state.doc.length,
            insert: value,
        },
    });

    const pos = cursorAtEnd ? view.state.doc.length : 0;

    view.dispatch({
        selection: {anchor: pos},
        scrollIntoView: true,
    });
}

export function getSelectedText(view: EditorView): string {
    const ranges = view.state.selection.ranges;
    const parts: string[] = [];

    for (const r of ranges) {
        if (r.empty) continue;
        parts.push(view.state.doc.sliceString(r.from, r.to));
    }

    return parts.join("\n");
}

export function hasSelection(view: EditorView): boolean {
    return view.state.selection.ranges.some(r => !r.empty);
}

// Fast and correct for CodeMirror: doc.length is character count.
export function updateCharCount(view: EditorView, lblCharCount: HTMLElement): void {
    const charCount = view.state.doc.length;
    lblCharCount.textContent = `[ ${charCount.toLocaleString()} Chars ]`;
}

export function focusInput(view: EditorView): void {
    view.focus();
}

export function clearEditor(view: EditorView): boolean {
    if (view.state.doc.length === 0) return false;

    view.dispatch({
        changes: {from: 0, to: view.state.doc.length, insert: ""}
    });

    return true; // tells caller something changed
}