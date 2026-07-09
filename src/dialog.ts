// import "./dialog.css";

import {getLocale} from "./i18n";

const DIALOG_ID = "quoteValidationDialog";
const DIALOG_STYLE_ID = "quoteValidationDialogStyle";

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
    if (!document.getElementById(DIALOG_STYLE_ID)) {
        document.head.insertAdjacentHTML("beforeend", createQuoteValidationDialogCss());
    }

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
    const summary = getElement<HTMLPreElement>("quoteValidationSummary");
    const issues = getElement<HTMLDivElement>("quoteValidationIssues");
    const goToBtn = getElement<HTMLButtonElement>("goToFirstSuspiciousLineBtn");

    dialog.dataset.quoteValidationPassed = String(result.is_valid);
    applyQuoteValidationDialogLocale();

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
    const s = getLocale().dialogQuoteValidation;

    return `
        <div id="${DIALOG_ID}" class="modal hidden">
            <div class="modal-card">
                <h2 id="quoteValidationTitle">${s.warningTitle}</h2>

                <pre id="quoteValidationSummary" class="modal-summary"></pre>

                <div id="quoteValidationIssues" class="issue-list"></div>

                <div class="modal-actions">
                    <button id="goToFirstSuspiciousLineBtn" class="hidden">
                        ${s.goToFirstSuspiciousLine}
                    </button>
                    <button id="closeQuoteValidationDialogBtn">
                        ${s.close}
                    </button>
                </div>
            </div>
        </div>
    `;
}

export function applyQuoteValidationDialogLocale(): void {
    const dialog = document.getElementById(DIALOG_ID);

    if (!dialog) {
        return;
    }

    const s = getLocale().dialogQuoteValidation;
    const title = getElement<HTMLHeadingElement>("quoteValidationTitle");
    const goToBtn = getElement<HTMLButtonElement>("goToFirstSuspiciousLineBtn");
    const closeBtn = getElement<HTMLButtonElement>("closeQuoteValidationDialogBtn");
    const passed = dialog.dataset.quoteValidationPassed === "true";

    title.textContent = passed ? s.passedTitle : s.warningTitle;
    goToBtn.textContent = s.goToFirstSuspiciousLine;
    closeBtn.textContent = s.close;
}

function createQuoteValidationDialogCss(): string {
    return `
        <style id="${DIALOG_STYLE_ID}">
            /* --------------------------------------------------------------------------
               Modal
               -------------------------------------------------------------------------- */

            .modal {
                position: fixed;
                inset: 0;
                z-index: 10000;

                display: flex;
                justify-content: center;
                align-items: center;

                padding: 18px;
                background: rgba(0, 0, 0, 0.46);
                backdrop-filter: blur(8px);
                -webkit-backdrop-filter: blur(8px);
            }

            .modal.hidden,
            .hidden {
                display: none !important;
            }

            /* --------------------------------------------------------------------------
               Card
               -------------------------------------------------------------------------- */

            .modal-card {
                width: min(760px, calc(100vw - 32px));
                max-height: calc(100vh - 48px);

                display: flex;
                flex-direction: column;
                overflow: hidden;

                color: var(--text);
                background: linear-gradient(180deg, var(--micaB), var(--micaA));
                border: 1px solid var(--stroke);
                border-radius: var(--r-xl, 22px);
                box-shadow: var(--shadow);
            }

            .modal-card h2 {
                margin: 0;
                padding: 18px 24px;

                font-size: 20px;
                font-weight: 700;

                border-bottom: 1px solid var(--stroke);
            }

            /* --------------------------------------------------------------------------
               Body
               -------------------------------------------------------------------------- */

            .modal-summary {
                margin: 0;
                padding: 18px 24px;

                flex: 1 1 auto;
                min-height: 0;
                overflow-y: auto;

                white-space: pre-wrap;
                word-break: break-word;

                font-family: var(--font-ui), sans-serif;
                font-size: 15px;
                line-height: 1.45;
                color: var(--text);

                border-bottom: 1px solid var(--stroke);
            }

            .issue-list {
                padding: 12px 24px 8px;
            }

            .issue-item {
                display: flex;
                align-items: flex-start;
                gap: 12px;

                margin-bottom: 10px;
                padding: 10px 12px;

                color: var(--text);
                background: var(--surface);

                border-left: 4px solid #f59e0b;
                border-radius: 8px;

                font-family: var(--font-mono), monospace;
                font-size: 13px;
                line-height: 1.5;
            }

            .issue-line {
                flex: 0 0 56px;

                text-align: right;
                font-weight: 600;
                color: var(--text);
            }

            .issue-text {
                flex: 1;

                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }

            .issue-more {
                margin-top: 4px;
                margin-left: 72px;

                color: var(--muted);
                font-size: 14px;
            }

            /* --------------------------------------------------------------------------
               Buttons
               -------------------------------------------------------------------------- */

            .modal-actions {
                display: flex;
                justify-content: flex-end;
                gap: 12px;
                flex: 0 0 auto;

                padding: 16px 24px;

                border-top: 1px solid var(--stroke);
            }

            .modal-actions button {
                min-width: 150px;
                height: 42px;

                display: inline-flex;
                align-items: center;
                justify-content: center;

                padding: 0 18px;

                font: inherit;
                font-weight: 650;
                color: var(--ctl-text);

                background: var(--ctl-bg);
                border: 1px solid var(--ctl-border);
                border-radius: 12px;

                cursor: pointer;
                box-shadow: var(--shadow2);

                transition: background 140ms ease,
                border-color 140ms ease,
                transform 60ms ease,
                box-shadow 140ms ease;
            }

            .modal-actions button:hover {
                background: var(--ctl-bg-hover);
                border-color: var(--ctl-border-hover);
            }

            .modal-actions button:active {
                transform: translateY(1px);
            }

            .modal-actions button:focus-visible {
                border-color: var(--ctl-border-focus);
                box-shadow: var(--ctl-shadow-focus);
            }

            /* --------------------------------------------------------------------------
               Responsive
               -------------------------------------------------------------------------- */

            @media (max-width: 640px) {
                .modal-card {
                    width: calc(100vw - 20px);
                    max-height: calc(100vh - 20px);
                    border-radius: 16px;
                }

                .modal-actions {
                    flex-direction: column-reverse;
                }

                .modal-actions button {
                    width: 100%;
                }
            }
        </style>
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
        more.textContent = getLocale().dialogQuoteValidation.more(suspiciousLines.length - 5);

        container.appendChild(more);
    }
}

function escapeHtml(text: string): string {
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
}
