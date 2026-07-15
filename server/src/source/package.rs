use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io::{Cursor, Read},
    path::Path,
    sync::Arc,
};
use thiserror::Error;
use zip::ZipArchive;

const MAX_PACKAGE_BYTES: usize = 128 * 1024 * 1024;
const MAX_WASM_BYTES: usize = 64 * 1024 * 1024;
const MAX_RESOURCE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
    pub version: u32,
    #[serde(default)]
    pub alt_names: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub content_rating: Option<u8>,
    #[serde(default)]
    pub min_app_version: Option<String>,
    #[serde(default)]
    pub max_app_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceListing {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub kind: Option<u8>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceConfig {
    #[serde(default)]
    pub language_select_type: Option<String>,
    #[serde(default)]
    pub supports_artist_search: bool,
    #[serde(default)]
    pub supports_author_search: bool,
    #[serde(default)]
    pub supports_tag_search: bool,
    #[serde(default)]
    pub allows_base_url_select: bool,
    #[serde(default)]
    pub breaking_change_version: Option<u32>,
    #[serde(default)]
    pub hides_filters_while_searching: bool,
    #[serde(default)]
    pub maximum_parallel_requests: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceManifest {
    pub info: SourceInfo,
    #[serde(default)]
    pub listings: Vec<SourceListing>,
    #[serde(default)]
    pub config: SourceConfig,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCapabilities {
    pub provides_home: bool,
    pub provides_listings: bool,
    pub dynamic_listings: bool,
    pub dynamic_filters: bool,
    pub dynamic_settings: bool,
    pub provides_image_requests: bool,
    pub processes_pages: bool,
    pub provides_page_descriptions: bool,
    pub provides_alternate_covers: bool,
    pub provides_base_url: bool,
    pub handles_notifications: bool,
    pub handles_deep_links: bool,
    pub handles_basic_login: bool,
    pub handles_web_login: bool,
    pub handles_migration: bool,
    pub sends_partial_results: bool,
    pub uses_network: bool,
    pub uses_html: bool,
    pub uses_canvas: bool,
    pub uses_defaults: bool,
    pub uses_javascript: bool,
}

impl SourceCapabilities {
    fn from_interface(interface: &SourceInterface) -> Self {
        let has_export = |name: &str| interface.exports.contains(name);
        let has_import = |module: &str| interface.uses_module(module);

        Self {
            provides_home: has_export("get_home"),
            provides_listings: has_export("get_manga_list"),
            dynamic_listings: has_export("get_listings"),
            dynamic_filters: has_export("get_filters"),
            dynamic_settings: has_export("get_settings"),
            provides_image_requests: has_export("get_image_request"),
            processes_pages: has_export("process_page_image"),
            provides_page_descriptions: has_export("get_page_description"),
            provides_alternate_covers: has_export("get_alternate_covers"),
            provides_base_url: has_export("get_base_url"),
            handles_notifications: has_export("handle_notification"),
            handles_deep_links: has_export("handle_deep_link"),
            handles_basic_login: has_export("handle_basic_login"),
            handles_web_login: has_export("handle_web_login"),
            handles_migration: has_export("handle_key_migration"),
            sends_partial_results: interface
                .imports
                .contains(&SourceImport::new("env", "send_partial_result")),
            uses_network: has_import("net"),
            uses_html: has_import("html"),
            uses_canvas: has_import("canvas"),
            uses_defaults: has_import("defaults"),
            uses_javascript: has_import("js"),
        }
    }
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SourceImport {
    pub module: String,
    pub name: String,
}

impl SourceImport {
    fn new(module: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SourceInterface {
    pub imports: BTreeSet<SourceImport>,
    pub exports: BTreeSet<String>,
}

impl SourceInterface {
    pub fn uses_module(&self, module: &str) -> bool {
        self.imports.iter().any(|item| item.module == module)
    }
}

#[derive(Debug, Clone)]
pub struct SourcePackage {
    pub manifest: SourceManifest,
    pub filters: Vec<serde_json::Value>,
    pub settings: Vec<serde_json::Value>,
    pub wasm: Arc<[u8]>,
    pub interface: SourceInterface,
    pub capabilities: SourceCapabilities,
}

#[derive(Debug, Error)]
pub enum SourcePackageError {
    #[error("source package is too large")]
    PackageTooLarge,
    #[error("invalid source package archive: {0}")]
    Archive(#[from] zip::result::ZipError),
    #[error("missing package entry: {0}")]
    MissingEntry(&'static str),
    #[error("package entry is too large: {0}")]
    EntryTooLarge(&'static str),
    #[error("invalid UTF-8 in package entry: {0}")]
    InvalidUtf8(&'static str),
    #[error("invalid source manifest: {0}")]
    InvalidManifest(#[from] serde_json::Error),
    #[error("package entry must contain a JSON array: {0}")]
    InvalidJsonArray(&'static str),
    #[error("invalid source id: {0}")]
    InvalidSourceId(String),
    #[error("source manifest version must be positive")]
    InvalidVersion,
    #[error("WASM binary is invalid: {0}")]
    InvalidWasm(String),
    #[error("WASM binary is missing required export: {0}")]
    MissingExport(&'static str),
    #[error("I/O error while reading source package: {0}")]
    Io(#[from] std::io::Error),
}

impl SourcePackage {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, SourcePackageError> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SourcePackageError> {
        if bytes.len() > MAX_PACKAGE_BYTES {
            return Err(SourcePackageError::PackageTooLarge);
        }

        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        let source_json = read_entry(&mut archive, "Payload/source.json", MAX_RESOURCE_BYTES)?
            .ok_or(SourcePackageError::MissingEntry("Payload/source.json"))?;
        let filters_json = read_entry(&mut archive, "Payload/filters.json", MAX_RESOURCE_BYTES)?;
        let settings_json = read_entry(&mut archive, "Payload/settings.json", MAX_RESOURCE_BYTES)?;
        let wasm = read_entry(&mut archive, "Payload/main.wasm", MAX_WASM_BYTES)?
            .ok_or(SourcePackageError::MissingEntry("Payload/main.wasm"))?;

        let source_json = String::from_utf8(source_json)
            .map_err(|_| SourcePackageError::InvalidUtf8("Payload/source.json"))?;
        let manifest: SourceManifest = serde_json::from_str(&source_json)?;
        validate_manifest(&manifest)?;
        validate_wasm(&wasm)?;

        let filters = parse_json_array(filters_json.as_deref(), "Payload/filters.json")?;
        let settings = parse_json_array(settings_json.as_deref(), "Payload/settings.json")?;
        let interface = scan_wasm(&wasm)?;
        validate_interface(&interface)?;
        let capabilities = SourceCapabilities::from_interface(&interface);

        Ok(Self {
            manifest,
            filters,
            settings,
            wasm: wasm.into(),
            interface,
            capabilities,
        })
    }
}

fn read_entry(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    path: &'static str,
    max_size: usize,
) -> Result<Option<Vec<u8>>, SourcePackageError> {
    let mut file = match archive.by_name(path) {
        Ok(file) => file,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if file.size() > max_size as u64 {
        return Err(SourcePackageError::EntryTooLarge(path));
    }
    let mut bytes = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut bytes)?;
    Ok(Some(bytes))
}

fn parse_json_array(
    bytes: Option<&[u8]>,
    path: &'static str,
) -> Result<Vec<serde_json::Value>, SourcePackageError> {
    let Some(bytes) = bytes else {
        return Ok(Vec::new());
    };
    let text = std::str::from_utf8(bytes).map_err(|_| SourcePackageError::InvalidUtf8(path))?;
    let value: serde_json::Value = serde_json::from_str(text)?;
    match value {
        serde_json::Value::Array(items) => Ok(items),
        _ => Err(SourcePackageError::InvalidJsonArray(path)),
    }
}

fn validate_manifest(manifest: &SourceManifest) -> Result<(), SourcePackageError> {
    if manifest.info.version == 0 {
        return Err(SourcePackageError::InvalidVersion);
    }
    if manifest.info.id.is_empty()
        || !manifest
            .info
            .id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return Err(SourcePackageError::InvalidSourceId(
            manifest.info.id.clone(),
        ));
    }
    Ok(())
}

fn validate_wasm(wasm: &[u8]) -> Result<(), SourcePackageError> {
    if wasm.len() < 8 || &wasm[..4] != b"\0asm" || wasm[4..8] != [1, 0, 0, 0] {
        return Err(SourcePackageError::InvalidWasm("invalid header".into()));
    }
    Ok(())
}

fn validate_interface(interface: &SourceInterface) -> Result<(), SourcePackageError> {
    const REQUIRED_EXPORTS: [&str; 6] = [
        "memory",
        "start",
        "free_result",
        "get_search_manga_list",
        "get_manga_update",
        "get_page_list",
    ];
    for export in REQUIRED_EXPORTS {
        if !interface.exports.contains(export) {
            return Err(SourcePackageError::MissingExport(export));
        }
    }
    Ok(())
}

fn scan_wasm(wasm: &[u8]) -> Result<SourceInterface, SourcePackageError> {
    let mut interface = SourceInterface::default();
    let mut offset = 8;
    while offset < wasm.len() {
        let section_id = *wasm
            .get(offset)
            .ok_or_else(|| SourcePackageError::InvalidWasm("missing section id".into()))?;
        offset += 1;
        let section_len = read_leb_u32(wasm, &mut offset)? as usize;
        let end = offset
            .checked_add(section_len)
            .ok_or_else(|| SourcePackageError::InvalidWasm("section length overflow".into()))?;
        if end > wasm.len() {
            return Err(SourcePackageError::InvalidWasm(
                "section exceeds file".into(),
            ));
        }
        let section = &wasm[offset..end];
        match section_id {
            2 => scan_imports(section, &mut interface)?,
            7 => scan_exports(section, &mut interface)?,
            _ => {}
        }
        offset = end;
    }
    Ok(interface)
}

fn scan_imports(section: &[u8], interface: &mut SourceInterface) -> Result<(), SourcePackageError> {
    let mut offset = 0;
    let count = read_leb_u32(section, &mut offset)?;
    for _ in 0..count {
        let module = read_wasm_name(section, &mut offset)?;
        let name = read_wasm_name(section, &mut offset)?;
        let kind = *section
            .get(offset)
            .ok_or_else(|| SourcePackageError::InvalidWasm("missing import kind".into()))?;
        offset += 1;
        match kind {
            0 => {
                let _ = read_leb_u32(section, &mut offset)?;
            }
            1 => {
                offset = skip_table_type(section, offset)?;
            }
            2 => {
                offset = skip_limits(section, offset)?;
            }
            3 => {
                offset = offset.checked_add(2).ok_or_else(|| {
                    SourcePackageError::InvalidWasm("global type overflow".into())
                })?;
            }
            4 => {
                let _ = read_leb_u32(section, &mut offset)?;
            }
            _ => {
                return Err(SourcePackageError::InvalidWasm(
                    "unknown import kind".into(),
                ))
            }
        }
        interface.imports.insert(SourceImport::new(module, name));
    }
    Ok(())
}

fn scan_exports(section: &[u8], interface: &mut SourceInterface) -> Result<(), SourcePackageError> {
    let mut offset = 0;
    let count = read_leb_u32(section, &mut offset)?;
    for _ in 0..count {
        let name = read_wasm_name(section, &mut offset)?;
        let _kind = *section
            .get(offset)
            .ok_or_else(|| SourcePackageError::InvalidWasm("missing export kind".into()))?;
        offset += 1;
        let _ = read_leb_u32(section, &mut offset)?;
        interface.exports.insert(name);
    }
    Ok(())
}

fn read_wasm_name(bytes: &[u8], offset: &mut usize) -> Result<String, SourcePackageError> {
    let len = read_leb_u32(bytes, offset)? as usize;
    let end = offset
        .checked_add(len)
        .ok_or_else(|| SourcePackageError::InvalidWasm("name length overflow".into()))?;
    let value = bytes
        .get(*offset..end)
        .ok_or_else(|| SourcePackageError::InvalidWasm("name exceeds section".into()))?;
    *offset = end;
    String::from_utf8(value.to_vec())
        .map_err(|_| SourcePackageError::InvalidWasm("invalid UTF-8 name".into()))
}

fn read_leb_u32(bytes: &[u8], offset: &mut usize) -> Result<u32, SourcePackageError> {
    let mut value = 0u32;
    for shift in (0..35).step_by(7) {
        let byte = *bytes
            .get(*offset)
            .ok_or_else(|| SourcePackageError::InvalidWasm("truncated LEB128".into()))?;
        *offset += 1;
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Ok(value);
        }
    }
    Err(SourcePackageError::InvalidWasm("invalid LEB128".into()))
}

fn skip_limits(bytes: &[u8], mut offset: usize) -> Result<usize, SourcePackageError> {
    let flags = read_leb_u32(bytes, &mut offset)?;
    let _ = read_leb_u32(bytes, &mut offset)?;
    if flags & 1 != 0 {
        let _ = read_leb_u32(bytes, &mut offset)?;
    }
    Ok(offset)
}

fn skip_table_type(bytes: &[u8], offset: usize) -> Result<usize, SourcePackageError> {
    let offset = offset
        .checked_add(1)
        .ok_or_else(|| SourcePackageError::InvalidWasm("table type overflow".into()))?;
    skip_limits(bytes, offset)
}

#[cfg(test)]
mod tests {
    use super::{scan_wasm, SourceCapabilities, SourcePackage};
    use std::io::{Cursor, Write};
    use zip::{write::SimpleFileOptions, ZipWriter};

    #[test]
    fn scans_import_and_export_names() {
        let wasm = minimal_wasm(
            &[("net", "send"), ("canvas", "copy_image")],
            &["start", "get_page_list", "process_page_image"],
        );
        let interface = scan_wasm(&wasm).expect("scan wasm");
        let capabilities = SourceCapabilities::from_interface(&interface);
        assert!(capabilities.uses_network);
        assert!(capabilities.uses_canvas);
        assert!(capabilities.processes_pages);
    }

    #[test]
    fn loads_package_with_optional_resources() {
        let wasm = minimal_wasm(
            &[("net", "send"), ("env", "send_partial_result")],
            &[
                "memory",
                "start",
                "free_result",
                "get_search_manga_list",
                "get_manga_update",
                "get_page_list",
                "get_home",
            ],
        );
        let archive = package_archive(
            br#"{"info":{"id":"zh.example","name":"Example","version":3,"languages":["zh"]}}"#,
            Some(br#"[{"type":"select"}]"#),
            None,
            &wasm,
        );

        let package = SourcePackage::from_bytes(&archive).expect("load source package");
        assert_eq!(package.manifest.info.id, "zh.example");
        assert_eq!(package.filters.len(), 1);
        assert!(package.settings.is_empty());
        assert!(package.capabilities.provides_home);
        assert!(package.capabilities.sends_partial_results);
    }

    fn minimal_wasm(imports: &[(&str, &str)], exports: &[&str]) -> Vec<u8> {
        let mut wasm = b"\0asm\x01\0\0\0".to_vec();
        let mut import_section = vec![];
        push_leb(imports.len() as u32, &mut import_section);
        for (module, name) in imports {
            push_name(module, &mut import_section);
            push_name(name, &mut import_section);
            import_section.push(0);
            import_section.push(0);
        }
        push_section(2, import_section, &mut wasm);

        let mut export_section = vec![];
        push_leb(exports.len() as u32, &mut export_section);
        for (index, name) in exports.iter().enumerate() {
            push_name(name, &mut export_section);
            export_section.push(0);
            push_leb(index as u32, &mut export_section);
        }
        push_section(7, export_section, &mut wasm);
        wasm
    }

    fn push_section(id: u8, payload: Vec<u8>, wasm: &mut Vec<u8>) {
        wasm.push(id);
        push_leb(payload.len() as u32, wasm);
        wasm.extend(payload);
    }

    fn push_name(value: &str, output: &mut Vec<u8>) {
        push_leb(value.len() as u32, output);
        output.extend(value.as_bytes());
    }

    fn push_leb(mut value: u32, output: &mut Vec<u8>) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            output.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn package_archive(
        manifest: &[u8],
        filters: Option<&[u8]>,
        settings: Option<&[u8]>,
        wasm: &[u8],
    ) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipWriter::new(&mut output);
            let options = SimpleFileOptions::default();
            write_entry(&mut archive, "Payload/source.json", manifest, options);
            write_entry(&mut archive, "Payload/main.wasm", wasm, options);
            if let Some(filters) = filters {
                write_entry(&mut archive, "Payload/filters.json", filters, options);
            }
            if let Some(settings) = settings {
                write_entry(&mut archive, "Payload/settings.json", settings, options);
            }
            archive.finish().expect("finish archive");
        }
        output.into_inner()
    }

    fn write_entry(
        archive: &mut ZipWriter<&mut Cursor<Vec<u8>>>,
        path: &str,
        bytes: &[u8],
        options: SimpleFileOptions,
    ) {
        archive.start_file(path, options).expect("start file");
        archive.write_all(bytes).expect("write file");
    }
}
