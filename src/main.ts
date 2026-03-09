// main.ts

import {invoke} from "@tauri-apps/api/core";

import {basicSetup} from "codemirror";
import {EditorState} from "@codemirror/state";
import {EditorView} from "@codemirror/view";

import {createCompareFeature} from "./features/compare";
import {mustGetEl} from "./dom/refs";
import {
    clearEditor,
    focusInput,
    getEditorText,
    getSelectedText,
    hasSelection,
    setEditorText,
    updateCharCount,
} from "./editors/codemirror";
import {
    getBatchHadError,
    setupListeners,
    startBatchListener,
    startOpenFileListener,
    stopBatchListener,
    stopOpenFileListener,
} from "./tauri/listeners";
import {
    getAppSettings,
    getCustomHeadingRegex,
    initAppSettings,
    isEditorLogEnabled,
} from "./app/settings";
import {getCurrentConfigFromUi, TextCode, ZhoConfig} from "./app/config";
import {setupUnifiedDrop} from "./tauri/dragdrop";

import {initUiLanguage} from "./i18n/initUiLanguage";
import {getRuntimeLabel, formatCharCount} from "./i18n/runtimeLabels";
import {openUrl} from "@tauri-apps/plugin-opener";

window.addEventListener("error", (e) => {
    console.error("JS error:", e.error || e.message);
});

document.addEventListener("DOMContentLoaded", async () => {
    initAppSettings();
    const app = createApp();
    app.init();
    initUiLanguage();
    await initAboutVersion();
});

document.getElementById("about-github")?.addEventListener("click", async (e) => {
    e.preventDefault();

    try {
        await openUrl("https://github.com/laisuk/opencc-fmmseg-gui");
    } catch (err) {
        console.error("Failed to open GitHub URL:", err);
    }
});

async function initAboutVersion() {
    try {
        const version = await invoke<string>("get_app_version");
        const el = document.getElementById("about-version")!;
        el.textContent = `v${version}`;
    } catch (err) {
        console.error("Failed to load app version:", err);
    }
}

function createApp() {
    // =========================================================
    // DOM REFS
    // =========================================================

    const editorLeftHost = mustGetEl<HTMLElement>("editor-left");
    const editorRightHost = mustGetEl<HTMLElement>("editor-right");

    const cbCompare = mustGetEl<HTMLInputElement>("compare-toggle");

    // Config
    const rbS2t = mustGetEl<HTMLInputElement>("rb_st");
    const rbT2s = mustGetEl<HTMLInputElement>("rb_ts");
    const rbManual = mustGetEl<HTMLInputElement>("rb_manual");

    const cbPunctuation = mustGetEl<HTMLInputElement>("cb_punctuation");
    const cbZhTwP = mustGetEl<HTMLInputElement>("cb_zhtwp");

    const rbZhTw = mustGetEl<HTMLInputElement>("rb_zhtw");
    const rbStd = mustGetEl<HTMLInputElement>("rb_std");
    const rbZhHk = mustGetEl<HTMLInputElement>("rb_zhhk");

    const selectConfig = mustGetEl<HTMLSelectElement>("select_config");

    // Labels
    const lblInput = mustGetEl<HTMLElement>("lbl_input");
    const lblOutput = mustGetEl<HTMLElement>("lbl_output");
    const lblStatusBar = mustGetEl<HTMLElement>("lbl_status_bar");
    const lblCharCount = mustGetEl<HTMLElement>("lbl_char_count");

    // Main buttons
    const btnPaste = mustGetEl<HTMLButtonElement>("paste");
    const btnCopy = mustGetEl<HTMLButtonElement>("copy");
    const btnOpenFile = mustGetEl<HTMLButtonElement>("open-file");
    const btnSaveFile = mustGetEl<HTMLButtonElement>("save-file");
    const btnConvert = mustGetEl<HTMLButtonElement>("convert");
    const btnReflow = mustGetEl<HTMLButtonElement>("reflow");
    const btnClearSource = mustGetEl<HTMLButtonElement>("clear-source");
    const btnClearDestination = mustGetEl<HTMLButtonElement>("clear-destination");

    // Batch buttons
    const btnAdd = document.getElementById("batch-add") as HTMLButtonElement | null;
    const btnRemove = document.getElementById("batch-remove") as HTMLButtonElement | null;
    const btnBatchClear = document.getElementById("batch-clear") as HTMLButtonElement | null;
    const btnBatchBrowse = document.getElementById("batch-browse") as HTMLButtonElement | null;
    const btnBatchRun = document.getElementById("batch-run") as HTMLButtonElement | null;
    const btnOpenOut = document.getElementById("batch-open-outdir") as HTMLButtonElement | null;
    const btnBatchLogClear = document.getElementById("batch-log-clear") as HTMLButtonElement | null;

    // Batch refs
    const batchList = document.getElementById("batch-files") as HTMLSelectElement | null;
    const batchCount = document.getElementById("batch-count") as HTMLElement | null;
    const batchOutDir = document.getElementById("batch-outdir") as HTMLInputElement | null;
    const batchLog = document.getElementById("batch-log") as HTMLTextAreaElement | null;

    // Tabs
    const tabs = document.querySelectorAll(".tab");
    const panels = document.querySelectorAll(".tab-panel");

    // =========================================================
    // EDITORS
    // =========================================================

    let editorLeft!: EditorView;
    let editorRight!: EditorView;

    const compare = createCompareFeature({
        getSourceText: () =>
            hasSelection(editorLeft) ? getSelectedText(editorLeft) : getEditorText(editorLeft),
        getDestText: () => getEditorText(editorRight),
        isEnabled: () => cbCompare.checked,
        dispatchToDest: (spec) => editorRight.dispatch(spec),
    });

    function createEditors() {
        editorLeft = new EditorView({
            state: EditorState.create({
                doc: "",
                extensions: [
                    basicSetup,
                    EditorView.lineWrapping,
                    EditorView.updateListener.of((update) => {
                        if (!update.docChanged) return;
                        setCharCountFast(update.state.doc.length);
                    }),
                ],
            }),
            parent: editorLeftHost,
        });

        editorRight = new EditorView({
            state: EditorState.create({
                doc: "",
                extensions: [
                    basicSetup,
                    EditorView.lineWrapping,
                    EditorState.readOnly.of(true),
                    EditorView.editable.of(false),
                    EditorView.contentAttributes.of({tabindex: "-1"}),
                    compare.compareField,
                ],
            }),
            parent: editorRightHost,
        });
    }

    // =========================================================
    // SMALL HELPERS
    // =========================================================

    function setStatus(text: string) {
        lblStatusBar.textContent = text;
    }

    function setCharCountFast(charCount: number) {
        lblCharCount.textContent = formatCharCount(charCount);
    }

    function appendLog(line: string) {
        if (!batchLog) return;

        if (batchLog.value.length > 0) {
            batchLog.value += "\n";
        }

        batchLog.value += line;
        batchLog.scrollTop = batchLog.scrollHeight;
    }

    function appendEditorLog(line: string) {
        if (!isEditorLogEnabled()) return;
        appendLog(line);
    }

    function getCurrentConfig(): ZhoConfig {
        return getCurrentConfigFromUi({
            rbManual,
            selectConfig,
            rbT2s,
            rbStd,
            rbZhHk,
            rbZhTw,
            cbZhTwP,
        });
    }

    function getInputText(): string {
        return hasSelection(editorLeft) ? getSelectedText(editorLeft) : getEditorText(editorLeft);
    }

    function updateInputInfo(textCode: TextCode) {
        if (textCode === 1) {
            rbT2s.checked = true;
            lblInput.innerText = getRuntimeLabel("zhHant");
            lblInput.dataset.kind = "zhHant";
        } else if (textCode === 2) {
            rbS2t.checked = true;
            lblInput.innerText = getRuntimeLabel("zhHans");
            lblInput.dataset.kind = "zhHans";
        } else {
            lblInput.innerText = getRuntimeLabel("others");
            lblInput.dataset.kind = "others";
        }

        setCharCountFast(editorLeft.state.doc.length);
    }

    function updateOutputInfo(config: ZhoConfig) {
        if (lblInput.dataset.kind === "others") {
            lblOutput.innerText = getRuntimeLabel("others");
            lblOutput.dataset.kind = "others";
        } else if (config.includes("jp")) {
            lblOutput.innerText = getRuntimeLabel("japanese");
            lblOutput.dataset.kind = "japanese";
        } else if (config.startsWith("s") || !config.includes("s")) {
            lblOutput.innerText = getRuntimeLabel("zhHant");
            lblOutput.dataset.kind = "zhHant";
        } else {
            lblOutput.innerText = getRuntimeLabel("zhHans");
            lblOutput.dataset.kind = "zhHans";
        }
    }

    async function detectInputText(text: string) {
        const preview = text.trim().slice(0, 200);
        const textCode = await invoke<TextCode>("zho_check", {text: preview});
        updateInputInfo(textCode);
    }

    function updateBatchCount() {
        if (batchList && batchCount) {
            batchCount.textContent = String(batchList.options.length);
        }
    }

    function getSortedUniquePaths(existingPaths: string[], incomingPaths: string[]): string[] {
        const set = new Set(existingPaths);

        for (const path of incomingPaths) {
            if (!path) continue;
            set.add(path);
        }

        return Array.from(set).sort((a, b) =>
            a.localeCompare(b, undefined, {numeric: true, sensitivity: "base"}),
        );
    }

    function rebuildBatchList(paths: string[]) {
        if (!batchList) return;

        batchList.innerHTML = "";

        for (const path of paths) {
            const option = document.createElement("option");
            option.value = path;
            option.textContent = path;
            batchList.appendChild(option);
        }

        updateBatchCount();
    }

    // =========================================================
    // MAIN ACTIONS
    // =========================================================

    async function handlePaste() {
        const pasted = await invoke<string | null>("paste_text2");
        setEditorText(editorLeft, pasted ?? "");

        const text = getEditorText(editorLeft);
        if (!text) {
            setStatus("Clipboard empty");
            updateCharCount(editorLeft, lblCharCount);
            return;
        }

        await detectInputText(text);
        setStatus("Clipboard contents pasted");
    }

    async function handleCopy() {
        const text = getEditorText(editorRight);

        if (!text) {
            console.warn("No text to copy");
            setStatus("No text to copy");
            return;
        }

        try {
            await invoke<void>("copy_text2", {text});
            setStatus("Output contents copied");
        } catch (error) {
            console.error("Error copying text:", error);
            setStatus("Error copying text: " + String(error));
        }
    }

    async function handleOpenFile() {
        await startOpenFileListener();

        const appSettings = getAppSettings();
        const customHeadingRegex = getCustomHeadingRegex();

        try {
            const [filePath, fileContents] = await invoke<[string, string]>("open_file", {
                isReflow: appSettings.autoReflow,
                pageHeader: appSettings.addPageHeader,
                compact: appSettings.compactPdf,
                config: getCurrentConfig(),
                punctuation: cbPunctuation.checked,
                customHeadingRegex,
            });

            if (!filePath) {
                setStatus("No file selected");
                return;
            }

            const text = fileContents ?? "";
            setEditorText(editorLeft, text);
            await detectInputText(text);

            setStatus("File Path: " + filePath);
        } catch (error) {
            appendLog(`✖ [OPEN] invoke open_file failed: ${String(error)}`);
            setStatus("Open failed");
        } finally {
            stopOpenFileListener();
        }
    }

    async function handleSaveFile() {
        try {
            const content = getEditorText(editorRight);
            const result = await invoke<string>("save_file", {content});
            setStatus("Saved: " + result);
        } catch (error) {
            console.error("Error saving file:", error);
            setStatus("File not saved: " + String(error));
        }
    }

    async function handleConvert() {
        try {
            btnConvert.disabled = true;
            setStatus("Converting...");

            const config = getCurrentConfig();
            const punctuation = cbPunctuation.checked;
            const text = getInputText();

            const result = await invoke<string>("convert_text", {
                text,
                config,
                punctuation,
            });

            compare.clear();
            setEditorText(editorRight, result);
            if (cbCompare.checked) {
                compare.apply();
            }
            updateOutputInfo(config);

            setStatus(
                hasSelection(editorLeft)
                    ? `Selection converted (${config})`
                    : `Conversion complete (${config})`,
            );
        } catch (error) {
            const msg =
                typeof error === "string"
                    ? error
                    : (error as { message?: string })?.message ?? String(error);

            setStatus(`Convert failed: ${msg}`);
        } finally {
            btnConvert.disabled = false;
        }
    }

    async function handleReflow() {
        try {
            setStatus("Reflowing...");

            const text = getInputText();
            const appSettings = getAppSettings();
            const pageHeader = appSettings.addPageHeader;
            const compact = appSettings.compactPdf;
            const customHeadingRegex = getCustomHeadingRegex();

            const result = await invoke<string>("reflow_text", {
                text,
                pageHeader,
                compact,
                customHeadingRegex,
            });

            compare.clear();
            setEditorText(editorRight, result);
            lblOutput.innerText = lblInput.innerText;

            setStatus(
                hasSelection(editorLeft)
                    ? `Selection reflow complete`
                    : `Reflow complete`,
            );
        } catch (error) {
            const msg =
                typeof error === "string"
                    ? error
                    : (error as any)?.message ?? String(error);

            setStatus(`Reflow failed: ${msg}`);
        }
    }

    function handleClearSource() {
        if (!clearEditor(editorLeft)) return;

        updateCharCount(editorLeft, lblCharCount);
        focusInput(editorLeft);
        setStatus("Editor source cleared");
    }

    function handleClearDestination() {
        compare.clear();
        if (!clearEditor(editorRight)) return;
        setStatus("Editor destination cleared");
    }

    // =========================================================
    // BATCH ACTIONS
    // =========================================================

    async function handleBatchAdd() {
        if (!batchList) return;

        const paths = await invoke<string[]>("pick_paths_batch");
        const existing = Array.from(batchList.options).map((o) => o.value);
        const sorted = getSortedUniquePaths(existing, paths);

        rebuildBatchList(sorted);
    }

    function handleBatchRemove() {
        if (!batchList) return;

        Array.from(batchList.selectedOptions).forEach((option) => option.remove());
        updateBatchCount();
    }

    function handleBatchClear() {
        if (!batchList) return;

        batchList.innerHTML = "";
        updateBatchCount();
    }

    async function handleBatchBrowse() {
        const dir = await invoke<string>("pick_output_dir");
        if (dir && batchOutDir) {
            batchOutDir.value = dir;
        }
    }

    async function handleBatchRun() {
        if (!batchList) return;

        const paths = Array.from(batchList.options)
            .map((o) => o.value)
            .filter(Boolean);

        if (paths.length === 0) {
            setStatus("No files selected");
            return;
        }

        const outputDir = (batchOutDir?.value ?? "").trim();
        if (!outputDir) {
            setStatus("Output directory not set");
            return;
        }

        const config = getCurrentConfig();
        const punctuation = cbPunctuation.checked;
        const appSettings = getAppSettings();
        const convertFilename = appSettings.convertFilename;
        const overwriteOutput = appSettings.overwriteOutput;
        const customHeadingRegex = getCustomHeadingRegex();

        if (batchLog) {
            batchLog.value = "";
        }

        appendLog("== Batch Global Info ==");
        appendLog(`Files: ${paths.length}`);
        appendLog(`Output: ${outputDir}`);
        appendLog(`Config: ${config}`);
        appendLog(`Punctuation: ${punctuation}`);
        appendLog(`Convert Filename: ${convertFilename}`);
        appendLog("-----------------------------");

        try {
            await startBatchListener();
            setStatus("Batch running...");

            await invoke<void>("run_batch_convert", {
                paths,
                outputDir,
                config,
                punctuation,
                convertFilename,
                overwriteOutput,
                customHeadingRegex,
            });

            const hadError = getBatchHadError();
            appendLog(hadError ? "⚠ Conversion completed with errors" : "✔ Conversion completed successfully");
            setStatus(hadError ? `Batch done with errors (${config})` : `Batch complete (${config})`);
        } catch (error: unknown) {
            const msg =
                typeof error === "string"
                    ? error
                    : (error as { message?: string })?.message ?? JSON.stringify(error);

            appendLog("✖ Batch failed:");
            appendLog(msg);
            setStatus("Batch failed");
        } finally {
            stopBatchListener();
        }
    }

    async function handleOpenOutputDir() {
        const outputDir = (batchOutDir?.value ?? "").trim();

        if (!outputDir) {
            setStatus("No output folder selected");
            return;
        }

        try {
            await invoke<void>("open_output_dir", {outputDir});
        } catch (error) {
            console.error("open_output_dir failed:", error);
            setStatus("Error: " + String(error));
        }
    }

    function handleBatchLogClear() {
        if (!batchLog) return;

        batchLog.value = "";
        setStatus("Logs Cleared");
    }

    function addPathsToBatchList(paths: string[]) {
        if (!batchList) return;

        const existing = Array.from(batchList.options).map((o) => o.value);
        const before = existing.length;
        const sorted = getSortedUniquePaths(existing, paths);

        rebuildBatchList(sorted);

        const added = sorted.length - before;
        if (added > 0) {
            setStatus(`Added ${added} file(s)`);
        }
    }

    // =========================================================
    // DRAG / DROP
    // =========================================================

    async function openFirstPathToEditor(path: string) {
        try {
            await startOpenFileListener();
            setStatus("Opening...");

            const appSettings = getAppSettings();

            const [pathStr, text] = await invoke<[string, string]>("open_path_to_editor", {
                path,
                isReflow: appSettings.autoReflow,
                pageHeader: appSettings.addPageHeader,
                compact: appSettings.compactPdf,
                config: getCurrentConfig(),
                punctuation: cbPunctuation.checked,
                customHeadingRegex: appSettings.customHeadingRegex,
            });

            setEditorText(editorLeft, text);
            await detectInputText(text);

            setStatus("File dropped: " + pathStr);
        } catch (error: unknown) {
            const msg =
                typeof error === "string"
                    ? error
                    : (error as { message?: string })?.message ?? JSON.stringify(error);

            setStatus("Drop failed");
            appendEditorLog("✖ Drop failed:");
            appendEditorLog(msg);
        } finally {
            stopOpenFileListener();
        }
    }

    async function setupDragDrop() {
        await setupUnifiedDrop({
            editorHost: editorLeftHost,
            batchList,
            setStatus,
            onOpenEditor: openFirstPathToEditor,
            onAddBatch: addPathsToBatchList,
        });
    }

    // =========================================================
    // UI WIRING
    // =========================================================

    function setupTabBehavior() {
        tabs.forEach((tab) => {
            tab.addEventListener("click", () => {
                tabs.forEach((t) => {
                    t.classList.remove("active");
                    t.setAttribute("aria-selected", "false");
                });

                panels.forEach((panel) => panel.classList.remove("active"));

                tab.classList.add("active");
                tab.setAttribute("aria-selected", "true");

                const panelId = tab.getAttribute("aria-controls");
                if (panelId) {
                    document.getElementById(panelId)?.classList.add("active");
                }
            });
        });
    }

    function setupConfigBehavior() {
        cbZhTwP.addEventListener("click", () => {
            if (cbZhTwP.checked) {
                rbZhTw.checked = true;
            }
        });

        rbStd.addEventListener("click", () => {
            if (cbZhTwP.checked) {
                cbZhTwP.checked = false;
            }
        });

        rbZhHk.addEventListener("click", () => {
            if (cbZhTwP.checked) {
                cbZhTwP.checked = false;
            }
        });

        selectConfig.addEventListener("click", () => {
            rbManual.checked = true;
        });
    }

    function setupMainButtons() {
        btnPaste.addEventListener("click", handlePaste);
        btnCopy.addEventListener("click", handleCopy);
        btnOpenFile.addEventListener("click", handleOpenFile);
        btnSaveFile.addEventListener("click", handleSaveFile);
        btnConvert.addEventListener("click", handleConvert);
        btnReflow.addEventListener("click", handleReflow);   // ⭐ NEW
        btnClearSource.addEventListener("click", handleClearSource);
        btnClearDestination.addEventListener("click", handleClearDestination);
        cbCompare.addEventListener("change", () => {
            compare.clear();

            if (cbCompare.checked) {
                compare.apply();
            }
        });
    }

    function setupBatchButtons() {
        btnAdd?.addEventListener("click", handleBatchAdd);
        btnRemove?.addEventListener("click", handleBatchRemove);
        btnBatchClear?.addEventListener("click", handleBatchClear);
        btnBatchBrowse?.addEventListener("click", handleBatchBrowse);
        btnBatchRun?.addEventListener("click", handleBatchRun);
        btnOpenOut?.addEventListener("click", handleOpenOutputDir);
        btnBatchLogClear?.addEventListener("click", handleBatchLogClear);
    }

    function setupTauriListeners() {
        setupListeners({
            setStatusBar: setStatus,
            appendLog,
            appendEditorLog,
        });
    }

    // =========================================================
    // PUBLIC APP API
    // =========================================================

    function init() {
        createEditors();
        setupTauriListeners();
        setupTabBehavior();
        setupConfigBehavior();
        setupMainButtons();
        setupBatchButtons();

        setCharCountFast(editorLeft.state.doc.length);

        setupDragDrop().catch((error) => {
            console.error("setupUnifiedDrop failed:", error);
        });
    }

    return {init};
}