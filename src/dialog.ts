import "./dialog.css";

const DIALOG_ID = "quoteValidationDialog";

export interface DialogQuoteIssue {
    line_number: number;
    text: string;
}

export interface DialogQuoteValidationResult {
    is_valid: boolean;
    summary: string;
    suspicious_lines: DialogQuoteIssue[];
}

export type GoToLineCallback = (line: number) => void;

export function initDialogs(): void {
    if (document.getElementById(DIALOG_ID)) {
        return;
    }

    document.body.insertAdjacentHTML("beforeend", createQuoteValidationDialogHtml());

    const dialog = getElement<HTMLDivElement>(DIALOG_ID);
    const closeBtn = getElement<HTMLButtonElement>("closeQuoteValidationDialogBtn");

    closeBtn.addEventListener("click", hideQuoteValidationDialog);

    dialog.addEventListener("click", (event: MouseEvent) => {
        if (event.target === dialog) {
            hideQuoteValidationDialog();
        }
    });
}

export function showQuoteValidationDialog(
    result: DialogQuoteValidationResult,
    goToLineCallback?: GoToLineCallback
): void {
    initDialogs();

    const dialog = getElement<HTMLDivElement>(DIALOG_ID);
    const title = getElement<HTMLHeadingElement>("quoteValidationTitle");
    const summary = getElement<HTMLPreElement>("quoteValidationSummary");
    const issues = getElement<HTMLDivElement>("quoteValidationIssues");
    const goToBtn = getElement<HTMLButtonElement>("goToFirstSuspiciousLineBtn");

    title.textContent = result.is_valid
        ? "Dialog Quote Validation Passed"
        : "Validation Warning";

    summary.textContent = result.summary;
    renderIssues(issues, result.suspicious_lines);

    goToBtn.classList.toggle("hidden", result.suspicious_lines.length === 0);
    goToBtn.onclick = () => {
        const first = result.suspicious_lines[0];

        hideQuoteValidationDialog();

        if (first && goToLineCallback) {
            goToLineCallback(first.line_number);
        }
    };

    dialog.classList.remove("hidden");

}

export function hideQuoteValidationDialog(): void {
    const dialog = document.getElementById(DIALOG_ID);

    if (dialog) {
        dialog.classList.add("hidden");
    }
}

function createQuoteValidationDialogHtml(): string {
    return `
        <div id="${DIALOG_ID}" class="modal hidden">
            <div class="modal-card">
                <h2 id="quoteValidationTitle">Dialog Quote Validation</h2>

                <pre id="quoteValidationSummary" class="modal-summary"></pre>

                <div id="quoteValidationIssues" class="issue-list"></div>

                <div class="modal-actions">
                    <button id="goToFirstSuspiciousLineBtn" class="hidden">
                        Go To First Suspicious Line
                    </button>
                    <button id="closeQuoteValidationDialogBtn">
                        Close
                    </button>
                </div>
            </div>
        </div>
    `;
}

function getElement<T extends HTMLElement>(id: string): T {
    const element = document.getElementById(id);

    if (!element) {
        throw new Error(`Element #${id} not found.`);
    }

    return element as T;
}

function renderIssues(
    container: HTMLDivElement,
    suspiciousLines: DialogQuoteIssue[]
): void {
    container.replaceChildren();

    for (const item of suspiciousLines.slice(0, 5)) {
        const div = document.createElement("div");
        div.className = "issue-item";

        div.innerHTML = `
            <span class="issue-line">${item.line_number}:</span>
            <span class="issue-text">${escapeHtml(item.text)}</span>
        `;

        container.appendChild(div);
    }

    if (suspiciousLines.length > 5) {
        const more = document.createElement("div");
        more.className = "issue-more";
        more.textContent = `...and ${suspiciousLines.length - 5} more.`;

        container.appendChild(more);
    }
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
}