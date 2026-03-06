import {getCurrentWebview} from "@tauri-apps/api/webview";

type DragDropDeps = {
    editorHost: HTMLElement
    batchList: HTMLSelectElement | null
    setStatus: (msg: string) => void
    onOpenEditor: (path: string) => Promise<void>
    onAddBatch: (paths: string[]) => void
}

function isOver(el: HTMLElement | null, x: number, y: number): boolean {
    if (!el) return false;
    const hit = document.elementFromPoint(x, y);
    return !!hit && (hit === el || el.contains(hit));
}

export async function setupUnifiedDrop(deps: DragDropDeps) {

    let lastOverEditor = false;
    let lastOverBatch = false;

    const webview = getCurrentWebview();

    await webview.onDragDropEvent(async (event) => {

        const p = event.payload;

        if (p.type === "over") {

            lastOverEditor = isOver(deps.editorHost, p.position.x, p.position.y);
            lastOverBatch = isOver(deps.batchList, p.position.x, p.position.y);

            deps.editorHost?.classList.toggle("dragover", lastOverEditor);
            deps.batchList?.classList.toggle("dragover", lastOverBatch);

            return;
        }

        if (p.type === "drop") {

            deps.editorHost?.classList.remove("dragover");
            deps.batchList?.classList.remove("dragover");

            const paths = p.paths ?? [];
            if (paths.length === 0) return;

            if (lastOverEditor) {
                await deps.onOpenEditor(paths[0]);
                return;
            }

            if (lastOverBatch) {
                deps.onAddBatch(paths);
                return;
            }

            return;
        }

        deps.editorHost?.classList.remove("dragover");
        deps.batchList?.classList.remove("dragover");

        lastOverEditor = false;
        lastOverBatch = false;

    });
}