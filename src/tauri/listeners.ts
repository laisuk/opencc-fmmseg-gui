import {listen} from "@tauri-apps/api/event";

type BatchProgress = {
    index: number;
    total: number;
    input?: string | null;
    output?: string | null;
    ok: boolean;
    message: string;
    progress?: string | null;
};

type ListenerDeps = {
    setStatusBar: (text: string) => void;
    appendLog: (line: string) => void;
    appendEditorLog: (line: string) => void;
};

let deps: ListenerDeps | null = null;
let unlistenBatch: null | (() => void) = null;
let unlistenOpen: null | (() => void) = null;
let batchHadError = false;

export function setupListeners(nextDeps: ListenerDeps): void {
    deps = nextDeps;
}

export function getBatchHadError(): boolean {
    return batchHadError;
}

function requireDeps(): ListenerDeps {
    if (!deps) {
        throw new Error("Listeners are not configured. Call setupListeners() first.");
    }
    return deps;
}

export async function startBatchListener(): Promise<void> {
    if (unlistenBatch) return;

    const d = requireDeps();
    batchHadError = false;

    unlistenBatch = await listen<BatchProgress>("batch-progress", (event) => {
        const p = event.payload;

        if (!p.ok) batchHadError = true;

        d.setStatusBar(
            p.progress && p.progress.trim().length > 0
                ? p.progress
                : p.total > 0
                    ? (!p.ok ? `Batch failed (${p.index}/${p.total})` : `Batch: ${p.index}/${p.total}`)
                    : p.message
        );

        const isBatchMarker = !p.input;

        const isFileDoneMessage =
            p.message === "converted" ||
            p.message === "office converted" ||
            p.message === "pdf converted" ||
            p.message === "skipped (output exists)" ||
            p.message === "overwriting existing file" ||
            p.message.startsWith("Batch done");

        const isRegexWarning =
            p.message.startsWith("Custom heading regex rejected by Rust:");

        const shouldLog =
            isBatchMarker || !p.ok || isFileDoneMessage || isRegexWarning;

        if (!shouldLog) return;

        if (p.input) {
            let symbol = "✔";

            if (!p.ok) symbol = "✖";
            else if (p.message === "skipped (output exists)") symbol = "⏭";
            else if (p.message === "overwriting existing file") symbol = "♻";

            d.appendLog(
                `${symbol} [${p.index}/${p.total}] ${p.message}\n` +
                `  in:  ${p.input}\n` +
                `  out: ${p.output}`
            );
        } else {
            const marker = isRegexWarning ? "⚠" : "==";
            d.appendLog(
                isRegexWarning
                    ? `${marker} ${p.message}`
                    : `== ${p.message} ==`
            );
        }
    });

    d.appendLog(`[${new Date().toLocaleString()}] Listener register: OK (batch-progress)`);
}

export function stopBatchListener(): void {
    if (!unlistenBatch) return;

    const d = requireDeps();
    unlistenBatch();
    unlistenBatch = null;
    d.appendLog("Listener stopped (batch-progress)");
}

export async function startOpenFileListener(): Promise<void> {
    if (unlistenOpen) return;

    const d = requireDeps();

    unlistenOpen = await listen<BatchProgress>("open-progress", (event) => {
        const p = event.payload;

        d.setStatusBar(
            (p.progress && p.progress.trim().length > 0)
                ? p.progress
                : (p.total > 0
                    ? (!p.ok ? `Open failed (${p.index}/${p.total})` : `Open: ${p.index}/${p.total}`)
                    : p.message)
        );

        const isOpenMarker = !p.input;

        const isFinalMessage =
            p.message === "opened" ||
            p.message === "text loaded" ||
            p.message === "pdf extracted" ||
            p.message.startsWith("Open done");

        const shouldLog = isOpenMarker || !p.ok || isFinalMessage;
        if (!shouldLog) return;

        if (p.input) {
            d.appendEditorLog(
                `${p.ok ? "✔" : "✖"} [OPEN] ${p.message}\n` +
                `  in:  ${p.input}\n` +
                `  out: ${p.output}`
            );
        } else {
            d.appendEditorLog(`== ${p.message} ==`);
        }
    });

    d.appendEditorLog(`[${new Date().toLocaleString()}] Listener register: OK (open-progress)`);
}

export function stopOpenFileListener(): void {
    if (!unlistenOpen) return;

    const d = requireDeps();
    unlistenOpen();
    unlistenOpen = null;
    d.appendEditorLog("Listener stopped (open-progress)");
}
