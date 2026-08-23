use crate::dictionary_json::DictionaryMaxlengthSerde;
use opencc_fmmseg::{CustomDictFileSpec, CustomDictMode, DictSlot, DictionaryMaxlength, OpenCC};
use rfd::AsyncFileDialog;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tauri::State;

const REQUIRED_DICTIONARY_FILES: &[&str] = &[
    "STCharacters.txt",
    "STPhrases.txt",
    "TSCharacters.txt",
    "TSPhrases.txt",
    "TWPhrases.txt",
    "TWPhrasesRev.txt",
    "TWVariantsPhrases.txt",
    "TWVariants.txt",
    "TWVariantsRev.txt",
    "TWVariantsRevPhrases.txt",
    "HKVariantsPhrases.txt",
    "HKVariants.txt",
    "HKVariantsRev.txt",
    "HKVariantsRevPhrases.txt",
    "JPShinjitaiCharacters.txt",
    "JPShinjitaiCharactersRev.txt",
    "JPShinjitaiPhrases.txt",
    "STPunctuations.txt",
    "TSPunctuations.txt",
];

struct CustomConverter {
    converter: Arc<OpenCC>,
    spec_count: usize,
}

/// Owns one immutable permanent converter and one transactional, hot-swappable converter.
pub struct OpenccManager {
    base: Arc<OpenCC>,
    custom: RwLock<Option<CustomConverter>>,
}

impl Default for OpenccManager {
    fn default() -> Self {
        Self {
            base: Arc::new(OpenCC::new()),
            custom: RwLock::new(None),
        }
    }
}

impl OpenccManager {
    pub fn active(&self) -> Arc<OpenCC> {
        self.custom
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|custom| Arc::clone(&custom.converter))
            .unwrap_or_else(|| Arc::clone(&self.base))
    }

    fn install(&self, candidate: Option<(OpenCC, usize)>) {
        let custom = candidate.map(|(converter, spec_count)| CustomConverter {
            converter: Arc::new(converter),
            spec_count,
        });
        *self
            .custom
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = custom;
    }

    pub fn status(&self) -> DictionaryRuntimeStatus {
        let custom = self
            .custom
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        DictionaryRuntimeStatus {
            is_custom: custom.is_some(),
            custom_count: custom.as_ref().map_or(0, |value| value.spec_count),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CustomDictRowDto {
    pub slot: String,
    pub mode: String,
    pub path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryOptions {
    slots: Vec<&'static str>,
    modes: [&'static str; 2],
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DictionaryRuntimeStatus {
    pub is_custom: bool,
    pub custom_count: usize,
}

fn normalize_rows(rows: Vec<CustomDictRowDto>) -> Result<Vec<CustomDictFileSpec<PathBuf>>, String> {
    rows.into_iter()
        .enumerate()
        .filter(|(_, row)| !row.path.trim().is_empty())
        .map(|(index, row)| {
            let row_number = index + 1;
            let slot = DictSlot::from_name_ignore_ascii_case(&row.slot).ok_or_else(|| {
                format!(
                    "Custom dictionary row {row_number}: invalid slot '{}'.",
                    row.slot
                )
            })?;
            let mode = match row.mode.trim().to_ascii_lowercase().as_str() {
                "append" => CustomDictMode::Append,
                "override" => CustomDictMode::Override,
                _ => {
                    return Err(format!(
                        "Custom dictionary row {row_number}: invalid mode '{}'.",
                        row.mode
                    ))
                }
            };
            let path = PathBuf::from(row.path.trim());
            if !path.is_file() {
                return Err(format!(
                    "Custom dictionary row {row_number}: file not found: {}",
                    path.display()
                ));
            }
            Ok(CustomDictFileSpec {
                slot,
                files: vec![path],
                mode,
            })
        })
        .collect()
}

fn build_custom_converter(
    specs: &[CustomDictFileSpec<PathBuf>],
) -> Result<Option<(OpenCC, usize)>, String> {
    if specs.is_empty() {
        return Ok(None);
    }
    let dictionary = DictionaryMaxlength::new()
        .map_err(|error| format!("Could not load the built-in dictionary: {error}"))?
        .with_custom_dict_files(specs)
        .map_err(|error| format!("Could not load custom dictionaries: {error}"))?;
    Ok(Some((OpenCC::from_dictionary(dictionary), specs.len())))
}

pub fn validate_base_directory(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!(
            "Dictionary source directory not found: {}",
            path.display()
        ));
    }
    let missing: Vec<_> = REQUIRED_DICTIONARY_FILES
        .iter()
        .filter(|name| !path.join(name).is_file())
        .copied()
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Dictionary source is missing required files: {}",
            missing.join(", ")
        ))
    }
}

#[tauri::command]
pub fn get_dictionary_options() -> DictionaryOptions {
    DictionaryOptions {
        slots: DictSlot::ALL
            .iter()
            .map(|slot| slot.canonical_name())
            .collect(),
        modes: ["Append", "Override"],
    }
}

#[tauri::command]
pub fn validate_dictionary_source(base_directory: String) -> Result<(), String> {
    validate_base_directory(Path::new(base_directory.trim()))
}

#[tauri::command]
pub async fn pick_dictionary_directory() -> String {
    AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|handle| handle.path().display().to_string())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn pick_custom_dictionary_file() -> String {
    AsyncFileDialog::new()
        .add_filter("Dictionary text files", &["txt"])
        .add_filter("All files", &["*"])
        .pick_file()
        .await
        .map(|handle| handle.path().display().to_string())
        .unwrap_or_default()
}

#[tauri::command]
pub async fn apply_custom_dictionaries(
    state: State<'_, crate::AppState>,
    rows: Vec<CustomDictRowDto>,
) -> Result<DictionaryRuntimeStatus, String> {
    let specs = normalize_rows(rows)?;
    let candidate = tauri::async_runtime::spawn_blocking(move || build_custom_converter(&specs))
        .await
        .map_err(|error| error.to_string())??;
    state.opencc.install(candidate);
    Ok(state.opencc.status())
}

#[tauri::command]
pub fn get_dictionary_runtime_status(state: State<'_, crate::AppState>) -> DictionaryRuntimeStatus {
    state.opencc.status()
}

#[tauri::command]
pub async fn generate_dictionary(
    base_directory: String,
    output_directory: String,
    format: String,
    rows: Vec<CustomDictRowDto>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let specs = normalize_rows(rows)?;
        let base = PathBuf::from(base_directory.trim());
        let output_dir = PathBuf::from(output_directory.trim());
        validate_base_directory(&base)?;
        if !output_dir.is_dir() {
            return Err(format!(
                "Output directory not found: {}",
                output_dir.display()
            ));
        }
        let dictionary = DictionaryMaxlength::from_dicts_at(&base)
            .map_err(|error| format!("Could not load source dictionaries: {error}"))?
            .with_custom_dict_files(&specs)
            .map_err(|error| format!("Could not load custom dictionaries: {error}"))?;
        let output = match format.trim().to_ascii_lowercase().as_str() {
            "zstd" => {
                let path = output_dir.join("dictionary_maxlength.zstd");
                DictionaryMaxlength::save_cbor_compressed(
                    &dictionary,
                    path.to_string_lossy().as_ref(),
                )
                .map_err(|error| format!("Could not generate ZSTD: {error}"))?;
                path
            }
            "cbor" => {
                let path = output_dir.join("dictionary_maxlength.cbor");
                dictionary
                    .serialize_to_cbor(&path)
                    .map_err(|error| format!("Could not generate CBOR: {error}"))?;
                path
            }
            "json" => {
                let path = output_dir.join("dictionary_maxlength.json");
                let writer = BufWriter::new(
                    File::create(&path)
                        .map_err(|error| format!("Could not create JSON output: {error}"))?,
                );
                let dto = DictionaryMaxlengthSerde::from(&dictionary);
                serde_json::to_writer_pretty(writer, &dto)
                    .map_err(|error| format!("Could not generate JSON: {error}"))?;
                path
            }
            _ => return Err(format!("Unsupported dictionary format: {format}")),
        };
        Ok(output.display().to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dictionary(contents: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("opencc-gui-{unique}.txt"));
        fs::write(&path, contents).unwrap();
        path
    }

    fn spec(slot: DictSlot, mode: CustomDictMode, path: PathBuf) -> CustomDictFileSpec<PathBuf> {
        CustomDictFileSpec {
            slot,
            files: vec![path],
            mode,
        }
    }

    #[test]
    fn base_converter_is_active_initially() {
        let manager = OpenccManager::default();
        assert_eq!(manager.active().convert("汉字", "s2t", false), "漢字");
        assert_eq!(manager.status().custom_count, 0);
    }

    #[test]
    fn valid_append_changes_conversion_and_reset_restores_base() {
        let manager = OpenccManager::default();
        let path = temp_dictionary("测试\t自訂結果\n");
        let candidate = build_custom_converter(&[spec(
            DictSlot::STPhrases,
            CustomDictMode::Append,
            path.clone(),
        )])
        .unwrap();
        manager.install(candidate);
        assert_eq!(manager.active().convert("测试", "s2t", false), "自訂結果");
        manager.install(None);
        assert_eq!(manager.active().convert("测试", "s2t", false), "測試");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn multiple_specs_and_override_work() {
        let manager = OpenccManager::default();
        let first = temp_dictionary("测试\t附加結果\n");
        let second = temp_dictionary("测试\t覆寫結果\n");
        manager.install(
            build_custom_converter(&[
                spec(DictSlot::STPhrases, CustomDictMode::Append, first.clone()),
                spec(
                    DictSlot::STPhrases,
                    CustomDictMode::Override,
                    second.clone(),
                ),
            ])
            .unwrap(),
        );
        assert_eq!(manager.status().custom_count, 2);
        assert_eq!(manager.active().convert("测试", "s2t", false), "覆寫結果");
        let _ = fs::remove_file(first);
        let _ = fs::remove_file(second);
    }

    #[test]
    fn failed_replacement_keeps_previous_converter() {
        let manager = OpenccManager::default();
        let valid = temp_dictionary("测试\t保留結果\n");
        manager.install(
            build_custom_converter(&[spec(
                DictSlot::STPhrases,
                CustomDictMode::Append,
                valid.clone(),
            )])
            .unwrap(),
        );
        let missing = valid.with_file_name("missing-opencc-gui-dictionary.txt");
        let failed =
            build_custom_converter(&[spec(DictSlot::STPhrases, CustomDictMode::Append, missing)]);
        assert!(failed.is_err());
        assert_eq!(manager.active().convert("测试", "s2t", false), "保留結果");
        let _ = fs::remove_file(valid);
    }

    #[test]
    fn gui_boundary_uses_canonical_slot_parsing_and_skips_empty_paths() {
        let rows = vec![CustomDictRowDto {
            slot: "  jpscharactersrev  ".into(),
            mode: "append".into(),
            path: "".into(),
        }];
        assert!(normalize_rows(rows).unwrap().is_empty());
        assert_eq!(
            DictSlot::from_name_ignore_ascii_case("jpscharactersrev"),
            Some(DictSlot::JPSCharactersRev)
        );
    }
}
