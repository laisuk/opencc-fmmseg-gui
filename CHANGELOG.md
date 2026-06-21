# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and uses
the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

---

## [0.11.0] - Unreleased

### Added

- Added config selection for `s2hkp()` + `hk2sp()` via UI controls.

### Changed

- Update `opencc-fmmseg` to v0.11.0

---

## [0.10.1] - 2026-05-31

### Changed

- Update `opencc-fmmseg` to v0.10.1

---

## [0.10.0] - 2026-05-24

### Changed

- Update `opencc-fmmseg` to v0.10.0

---

## [0.9.2] - 2026-05-10

### Changed

- Update `opencc-fmmseg` to v0.9.2

### Fixed

- Fixed: Reset previous selection after update new context to Editor Left.

---

## [0.9.1] – 2026-03-21

### Changed

- Update `opencc-fmmseg` to v0.9.1

---

## [0.9.0] – 2026-03-07

### Changed

- Refactored `compare.ts` to use **CodeMirror StateEffect-based decoration updates** (Plan-3 architecture)
- Replaced external decoration refresh logic with transaction-driven updates
- Added update gate using `value.map(tr.changes)` to prevent range errors such as  
  `Position x is out of range for changeset of length y`
- Improved stability when replacing destination text during convert, reflow, or clear operations
- Enhanced robustness during development hot-reload (Vite / WebView reload scenarios)
- Optimized Unicode-safe comparison logic for very large texts (tested with multi-million character documents)
- Compare decorations now remain fully **copy/paste friendly** as an overlay instead of modifying editor content
- Internal architecture simplified for clearer state flow between editor document updates and compare overlay

### Renamed

- Project renamed from **`tauri-zho-vite`** → **`opencc-fmmseg-gui`**

---

## [0.0.0] – 2026-03-00

### Added

- Initial public release of **opencc-fmmseg-gui**
- Cross-platform **Tauri 2** GUI
- Supports **Simplified ↔ Traditional Chinese conversion**
- Built using Rust-based **Opencc-Fmmseg** library

---