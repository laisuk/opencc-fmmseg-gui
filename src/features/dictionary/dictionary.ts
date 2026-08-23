import {invoke} from "@tauri-apps/api/core";
import {getLocale} from "../../i18n";
import dictionaryHtml from "./dictionary.html?raw";
import folderIcon from "../../assets/icons/folder_open.svg?raw";
import bookIcon from "../../assets/icons/book.svg?raw";
import bookAddIcon from "../../assets/icons/book_add.svg?raw";
import addIcon from "../../assets/icons/add_circle.svg?raw";
import removeIcon from "../../assets/icons/subtract_circle.svg?raw";
import "./dictionary.css";

type CustomDictionaryRow = { slot: string; mode: string; path: string };
type DictionaryOptions = { slots: string[]; modes: string[] };
type RuntimeStatus = { isCustom: boolean; customCount: number };

const STORAGE = {
    rows: "dictionary.customRows",
    applyStartup: "dictionary.applyDuringStartup",
    baseDirectory: "dictionary.baseDirectory",
    outputDirectory: "dictionary.outputDirectory",
    generateWithCustom: "dictionary.generateWithCustom",
} as const;

const icons: Record<string, string> = {
    folder: folderIcon,
    book: bookIcon,
    bookAdd: bookAddIcon,
    add: addIcon,
    remove: removeIcon,
};

let rows: CustomDictionaryRow[] = [];
let options: DictionaryOptions = {slots: [], modes: ["Append", "Override"]};
let initialized = false;
let runtimeStatus: RuntimeStatus = {isCustom: false, customCount: 0};
let validationSequence = 0;

function el<T extends HTMLElement>(id: string): T {
    const value = document.getElementById(id);
    if (!value) throw new Error(`Missing Dictionary element: #${id}`);
    return value as T;
}

function readRows(): CustomDictionaryRow[] {
    try {
        const value = JSON.parse(localStorage.getItem(STORAGE.rows) ?? "[]");
        if (!Array.isArray(value)) return [];
        return value.map((row) => ({
            slot: typeof row?.slot === "string" ? row.slot : "",
            mode: row?.mode === "Override" ? "Override" : "Append",
            path: typeof row?.path === "string" ? row.path : "",
        }));
    } catch {
        return [];
    }
}

function saveRows(): void {
    localStorage.setItem(STORAGE.rows, JSON.stringify(rows));
}

function setStatus(message: string, error = false): void {
    const status = el<HTMLDivElement>("dictionary-status");
    status.textContent = message;
    status.classList.toggle("error", error);
}

function format(template: string, values: Record<string, string | number>): string {
    return template.replace(/\{(\w+)}/g, (_, key: string) => String(values[key] ?? ""));
}

function errorText(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

function setBusy(busy: boolean): void {
    document.querySelectorAll<HTMLButtonElement>("#panel-dictionary button")
        .forEach((button) => button.disabled = busy);
    el<HTMLInputElement>("dictionary-generate-with-custom").disabled = busy;
}

function updateRuntimeStatus(): void {
    const strings = getLocale().dictionary;
    el("dictionary-runtime-status").textContent = runtimeStatus.isCustom
        ? format(strings.customRuntime, {count: runtimeStatus.customCount})
        : strings.defaultRuntime;
}

function makeOption(value: string): HTMLOptionElement {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = value;
    return option;
}

function renderRows(): void {
    const strings = getLocale().dictionary;
    const host = el<HTMLDivElement>("dictionary-custom-rows");
    host.replaceChildren();
    rows.forEach((row, index) => {
        const wrapper = document.createElement("div");
        wrapper.className = "dictionary-row";

        const slot = document.createElement("select");
        slot.className = "fluent-select";
        options.slots.forEach((value) => slot.append(makeOption(value)));
        if (options.slots.includes(row.slot)) slot.value = row.slot;
        else if (options.slots.length) {
            row.slot = options.slots[0];
            slot.value = row.slot;
        }
        slot.addEventListener("change", () => {
            rows[index].slot = slot.value;
            saveRows();
        });

        const mode = document.createElement("select");
        mode.className = "fluent-select";
        options.modes.forEach((value) => mode.append(makeOption(value)));
        mode.value = options.modes.includes(row.mode) ? row.mode : options.modes[0];
        mode.addEventListener("change", () => {
            rows[index].mode = mode.value;
            saveRows();
        });

        const path = document.createElement("input");
        path.className = "dictionary-file-input fluent-input";
        path.type = "text";
        path.placeholder = strings.filePlaceholder;
        path.value = row.path;
        path.addEventListener("input", () => {
            rows[index].path = path.value;
            saveRows();
        });

        const browse = document.createElement("button");
        browse.className = "dictionary-browse-file";
        browse.type = "button";
        browse.innerHTML = `<span class="dictionary-icon">${bookAddIcon}</span><span>${strings.browse}</span>`;
        browse.addEventListener("click", async () => {
            const selected = await invoke<string>("pick_custom_dictionary_file");
            if (selected) {
                rows[index].path = selected;
                path.value = selected;
                saveRows();
            }
        });

        const remove = document.createElement("button");
        remove.className = "dictionary-remove";
        remove.type = "button";
        remove.title = strings.remove;
        remove.innerHTML = `<span class="dictionary-icon">${removeIcon}</span><span>${strings.remove}</span>`;
        remove.addEventListener("click", () => {
            rows.splice(index, 1);
            saveRows();
            renderRows();
        });

        wrapper.append(slot, mode, path, browse, remove);
        host.append(wrapper);
    });
    el("dictionary-empty-rows").hidden = rows.length > 0;
}

async function validateSource(): Promise<void> {
    const sequence = ++validationSequence;
    const baseDirectory = el<HTMLInputElement>("dictionary-base-directory").value.trim();
    setBusy(false);
    if (!baseDirectory) return;
    try {
        await invoke("validate_dictionary_source", {baseDirectory});
        if (sequence === validationSequence) setStatus("");
    } catch (error) {
        if (sequence === validationSequence) {
            const strings = getLocale().dictionary;
            setStatus(format(strings.error, {error: errorText(error)}), true);
        }
    }
    if (sequence === validationSequence) setBusy(false);
}

async function chooseDirectory(target: HTMLInputElement): Promise<void> {
    const selected = await invoke<string>("pick_dictionary_directory");
    if (!selected) return;
    target.value = selected;
    target.dispatchEvent(new Event("input"));
}

async function applyRows(startup = false): Promise<void> {
    const strings = getLocale().dictionary;
    setBusy(true);
    setStatus(strings.applying);
    try {
        runtimeStatus = await invoke<RuntimeStatus>("apply_custom_dictionaries", {rows});
        updateRuntimeStatus();
        setStatus(runtimeStatus.isCustom ? strings.applied : strings.reset);
    } catch (error) {
        const message = startup
            ? format(strings.startupFailed, {error: errorText(error)})
            : format(strings.error, {error: errorText(error)});
        setStatus(message, true);
    } finally {
        setBusy(false);
    }
}

async function generate(formatName: string): Promise<void> {
    const strings = getLocale().dictionary;
    setBusy(true);
    setStatus(format(strings.generating, {format: formatName.toUpperCase()}));
    try {
        const path = await invoke<string>("generate_dictionary", {
            baseDirectory: el<HTMLInputElement>("dictionary-base-directory").value,
            outputDirectory: el<HTMLInputElement>("dictionary-output-directory").value,
            format: formatName,
            rows: el<HTMLInputElement>("dictionary-generate-with-custom").checked ? rows : [],
        });
        setStatus(format(strings.generated, {path}));
    } catch (error) {
        setStatus(format(strings.error, {error: errorText(error)}), true);
    } finally {
        setBusy(false);
    }
}

export function applyDictionaryLocale(): void {
    if (!initialized) return;
    const s = getLocale().dictionary;
    el("dictionary-title").textContent = s.title;
    el("dictionary-generation-heading").textContent = s.generationHeading;
    el("dictionary-base-label").textContent = s.baseDirectory;
    el("dictionary-output-label").textContent = s.outputDirectory;
    el("dictionary-custom-heading").textContent = s.customSlots;
    el("dictionary-slot-column").textContent = s.slot;
    el("dictionary-mode-column").textContent = s.mode;
    el("dictionary-file-column").textContent = s.dictionaryFile;
    el("dictionary-empty-rows").textContent = s.empty;
    el("dictionary-apply-startup-text").textContent = s.applyStartup;
    el("dictionary-generate-with-custom-text").textContent = s.generateWithCustom;
    el<HTMLButtonElement>("dictionary-browse-base").lastElementChild!.textContent = s.browse;
    el<HTMLButtonElement>("dictionary-browse-output").lastElementChild!.textContent = s.browse;
    el<HTMLButtonElement>("dictionary-add-row").lastElementChild!.textContent = s.add;
    el<HTMLButtonElement>("dictionary-apply").lastElementChild!.textContent = s.apply;
    const generateLabels = [s.generateZstd, s.generateCbor, s.generateJson];
    document.querySelectorAll<HTMLButtonElement>("[data-generate]").forEach((button, index) => {
        button.lastElementChild!.textContent = generateLabels[index];
    });
    updateRuntimeStatus();
    renderRows();
}

export async function initDictionary(): Promise<void> {
    const host = el<HTMLDivElement>("dictionary-host");
    host.innerHTML = dictionaryHtml;
    host.querySelectorAll<HTMLElement>("[data-icon]").forEach((node) => {
        node.innerHTML = icons[node.dataset.icon ?? ""] ?? "";
    });
    initialized = true;
    rows = readRows();
    options = await invoke<DictionaryOptions>("get_dictionary_options");
    const base = el<HTMLInputElement>("dictionary-base-directory");
    const output = el<HTMLInputElement>("dictionary-output-directory");
    const startup = el<HTMLInputElement>("dictionary-apply-startup");
    const generateWithCustom = el<HTMLInputElement>("dictionary-generate-with-custom");
    base.value = localStorage.getItem(STORAGE.baseDirectory) ?? "";
    output.value = localStorage.getItem(STORAGE.outputDirectory) ?? "";
    startup.checked = localStorage.getItem(STORAGE.applyStartup) === "true";
    generateWithCustom.checked = localStorage.getItem(STORAGE.generateWithCustom) !== "false";

    base.addEventListener("input", () => {
        localStorage.setItem(STORAGE.baseDirectory, base.value);
        window.setTimeout(() => void validateSource(), 180);
    });
    output.addEventListener("input", () => localStorage.setItem(STORAGE.outputDirectory, output.value));
    startup.addEventListener("change", () => localStorage.setItem(STORAGE.applyStartup, String(startup.checked)));
    generateWithCustom.addEventListener("change", () =>
        localStorage.setItem(STORAGE.generateWithCustom, String(generateWithCustom.checked)));
    el("dictionary-browse-base").addEventListener("click", () => void chooseDirectory(base));
    el("dictionary-browse-output").addEventListener("click", () => void chooseDirectory(output));
    el("dictionary-add-row").addEventListener("click", () => {
        rows.push({slot: options.slots[0] ?? "", mode: options.modes[0] ?? "Append", path: ""});
        saveRows();
        renderRows();
    });
    el("dictionary-apply").addEventListener("click", () => void applyRows());
    document.querySelectorAll<HTMLButtonElement>("[data-generate]").forEach((button) => {
        button.addEventListener("click", () => void generate(button.dataset.generate ?? ""));
    });

    runtimeStatus = await invoke<RuntimeStatus>("get_dictionary_runtime_status");
    applyDictionaryLocale();
    await validateSource();
    if (startup.checked) await applyRows(true);
}
