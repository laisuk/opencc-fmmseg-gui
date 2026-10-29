# Vendored ZIP 3.0.0 for the Tauri backend

Synced on 2026-10-30 from the clean local source tree
`D:\codes\Rust\opencc-fmmseg\tools\opencc-rs\src\office_converter\zip`,
repository commit `87afddd02ecd70c2ebdb4699de4d1ebb87b1182c`.
This is the same trimmed and hardened implementation ported to the WASM and
pdfium-helper converters, not a standalone Cargo crate.

The original MIT license is preserved verbatim in [zip/LICENSE](zip/LICENSE).
Historical source-port notes and the supplied advisory are retained in
[UPSTREAM_VENDOR_NOTES.md](UPSTREAM_VENDOR_NOTES.md) and
[SECURITY_ADVISORY.md](SECURITY_ADVISORY.md). Their verification claims and
relative paths describe the original repositories, not this GUI port.

## Integration and dependency changes

- Added `pub(crate) mod zip` inside `office_converter`. Original internal paths
  already match `crate::office_converter::zip`; no public ZIP API was added.
- Routed converter production/tests, format detection (`utils`), and DOCX,
  ODT, and EPUB extractors through this module, including their ZIP error types.
- Removed external zip 3.0.0. Added crc32fast 1.5, memchr 2.7, flate2 =1.1.1
  with only rust_backend, and indexmap 2.7 as direct support dependencies.
  Indexmap deliberately uses a compatible range rather than the source CLI's
  exact =2.7.1 pin, retaining versions required by Tauri's support crates.
- Used the shared portable path predicate while preserving unsafe-entry omission
  in the converter. Drive-relative names, alternate data streams, and NULs are
  now omitted alongside rooted and traversal paths.
- Preserved text conversion, font handling, entry selection, EPUB mimetype
  ordering/compression, raw asset copying, and output replacement semantics.
- Retained eight ZIP regressions and adapted the source converter security tests
  with isolated temporary directories. Added detection/extraction regressions
  proving malformed headers fail at the ZIP boundary in all three extractors.
- Marked upstream private ZIP reference examples `rust,ignore`: they describe an
  external ZIP crate/API and cannot be doctested as this private app module.

The pure-Rust DEFLATE path is flate2 → miniz_oxide → adler2. Support dependencies
may be shared with Tauri and other app dependencies; C`argo.lock` records the
resolved graph. Compression byte sequences may change without changing content.

## Validation limits and maintenance

Both `ZipArchive::new` and `with_config` validate raw names, UnicodePath identity,
selected flags, compression method, CRC/sizes, checked offsets, central-directory
boundaries, duplicate effective names, and overlapping indexed-entry spans before
entries can be inspected or raw-copied. Rebuilt conversion output is reopened
through that validator before return or destination replacement.

Output validation retains its metadata-only reopen; raw-copy does not decompress
payloads to verify CRCs. No archive resource quotas, data-descriptor trailer
reconciliation, timestamp/permission parity, protection against concurrent file
mutation, or general parser-equivalence guarantee is added. Some ambiguous archives
formerly accepted now fail. No output lifecycle redesign is part of this port;
existing native destination removal/rename is not a general atomicity guarantee.
See the historical source notes for further limitations.

Vendoring removes automated external zip dependency alerts, not maintenance duties.
Track upstream ZIP/RustSec advisories, assess fixes against retained code paths,
preserve the license and notes, and run converter/ZIP plus full app tests when
updating. Removal from `Cargo.lock` alone is not proof an advisory is resolved.


## Port verification (2026-10-30, Windows x86_64 MSVC)

- `cargo test --locked --offline` from `src-tauri`: PASS, 50 library
  tests and the binary test target. Eleven private-module reference doctests are
  explicitly ignored (ten vendored examples plus the existing converter example).
- `node --test tests/*.test.mjs` from the app root: PASS, 4 frontend tests.
- `rustfmt --edition 2021 --check` on all edited Rust modules, including the
  recursive Office/ZIP module and security regressions: PASS.
- `git diff --check`: PASS.
- External zip, Zopfli, and zlib-rs packages are absent from Cargo.lock. Other
  Tauri consumers still require shared dependencies such as bumpalo, log, and
  simd-adler32. Unrelated package versions and dependency edges were preserved.
- This app retains indexmap 2.14.0 and memchr 2.8.0 in its resolved graph;
  the ZIP support backend resolves to flate2 1.1.1 / miniz_oxide 0.8.9 / adler2 2.0.1.
- The original ZIP MIT license SHA-256 matches the supplied source exactly.

No interactive GUI, installer bundling, cross-platform runtime, or large
real-world Office/EPUB corpus testing was run during this port.
