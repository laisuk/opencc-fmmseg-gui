import {getCurrentWebview} from "@tauri-apps/api/webview";

type DragDropDeps = {
    editorHost: HTMLElement
    batchList: HTMLSelectElement | null
    onOpenEditor: (path: string) => Promise<void>
    onAddBatch: (paths: string[]) => void
}

function isOver(el: HTMLElement | null, x: number, y: number): boolean {
    if (!el) return false;
    const rect = el.getBoundingClientRect();
    // Half-open bounds exclude empty/hidden targets and avoid shared-edge ambiguity.
    return x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom;
}

export async function setupUnifiedDrop(deps: DragDropDeps) {
    // Native event callbacks are not awaited. Keep editor opens in arrival order,
    // including progress-listener setup/cleanup, without blocking hover or batch.
    let pendingOpen = Promise.resolve();

    await getCurrentWebview().onDragDropEvent((event) => {
        const p = event.payload;

        if (p.type === "over" || p.type === "enter") {
            // Tauri positions are physical pixels; DOM rectangles use CSS pixels.
            const x = p.position.x / window.devicePixelRatio;
            const y = p.position.y / window.devicePixelRatio;
            deps.editorHost.classList.toggle("dragover", isOver(deps.editorHost, x, y));
            deps.batchList?.classList.toggle("dragover", isOver(deps.batchList, x, y));
            return;
        }

        deps.editorHost.classList.remove("dragover");
        deps.batchList?.classList.remove("dragover");

        if (p.type !== "drop" || p.paths.length === 0) return;

        // Resolve the target before async work or subsequent layout changes.
        const x = p.position.x / window.devicePixelRatio;
        const y = p.position.y / window.devicePixelRatio;
        if (isOver(deps.editorHost, x, y)) {
            const path = p.paths[0];
            pendingOpen = pendingOpen
                .then(() => deps.onOpenEditor(path))
                .catch((error) => console.error("Dropped file open failed:", error));
        } else if (isOver(deps.batchList, x, y)) {
            deps.onAddBatch(p.paths);
        }
    });
}
