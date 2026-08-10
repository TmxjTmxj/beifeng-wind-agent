//! Wind Knowledge Hub document detection, parsing, and rule-based metadata.

use std::fmt;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Pdf,
    Docx,
    Xlsx,
    Csv,
    Txt,
    Markdown,
    Json,
    Jpg,
    Jpeg,
    Png,
    Mp4,
}

impl FileType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Csv => "csv",
            Self::Txt => "txt",
            Self::Markdown => "md",
            Self::Json => "json",
            Self::Jpg => "jpg",
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::Mp4 => "mp4",
        }
    }

    #[must_use]
    pub const fn is_reserved_media(self) -> bool {
        matches!(self, Self::Jpg | Self::Jpeg | Self::Png | Self::Mp4)
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_type: String,
    pub domain: String,
    pub equipment: String,
    pub source_type: String,
    pub original_path: String,
    pub parser_status: String,
    pub reserved_media: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentRecord {
    pub text: Option<String>,
    pub metadata: DocumentMetadata,
}

#[must_use]
pub fn detect_file_type(path: &Path) -> Option<FileType> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "pdf" => Some(FileType::Pdf),
        "docx" => Some(FileType::Docx),
        "xlsx" => Some(FileType::Xlsx),
        "csv" => Some(FileType::Csv),
        "txt" => Some(FileType::Txt),
        "md" | "markdown" => Some(FileType::Markdown),
        "json" => Some(FileType::Json),
        "jpg" => Some(FileType::Jpg),
        "jpeg" => Some(FileType::Jpeg),
        "png" => Some(FileType::Png),
        "mp4" => Some(FileType::Mp4),
        _ => None,
    }
}

pub fn parse_document(path: &Path, key_path: &str) -> Result<DocumentRecord, String> {
    let file_type = detect_file_type(path)
        .ok_or_else(|| format!("unsupported file type: {}", path.display()))?;
    let text = if file_type.is_reserved_media() {
        None
    } else {
        Some(parse_text_content(path, file_type)?)
    };
    let metadata = classify_document(path, key_path, file_type, text.as_deref());
    Ok(DocumentRecord { text, metadata })
}

fn parse_text_content(path: &Path, file_type: FileType) -> Result<String, String> {
    match file_type {
        FileType::Txt | FileType::Markdown | FileType::Json | FileType::Csv => {
            std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))
        }
        FileType::Docx => extract_docx_text(path),
        FileType::Xlsx => extract_xlsx_text(path),
        FileType::Pdf => extract_pdf_text(path),
        FileType::Jpg | FileType::Jpeg | FileType::Png | FileType::Mp4 => Ok(String::new()),
    }
}

fn classify_document(
    path: &Path,
    key_path: &str,
    file_type: FileType,
    text: Option<&str>,
) -> DocumentMetadata {
    let haystack = classification_haystack(path, text);
    let domain = classify_domain(&haystack, file_type);
    let equipment = classify_equipment(&haystack, file_type);
    let source_type = classify_source_type(key_path, file_type);
    DocumentMetadata {
        file_type: file_type.as_str().to_string(),
        domain,
        equipment,
        source_type,
        original_path: path.to_string_lossy().replace('\\', "/"),
        parser_status: if file_type.is_reserved_media() {
            "reserved_media_only".to_string()
        } else {
            "parsed_text".to_string()
        },
        reserved_media: file_type.is_reserved_media(),
    }
}

fn classification_haystack(path: &Path, text: Option<&str>) -> String {
    let mut value = path.to_string_lossy().to_lowercase();
    if let Some(text) = text {
        value.push('\n');
        value.push_str(&text.to_lowercase());
    }
    value
}

fn classify_domain(haystack: &str, file_type: FileType) -> String {
    if file_type.is_reserved_media()
        && contains_any(
            haystack,
            &[
                "uav",
                "无人机",
                "巡检照片",
                "inspection image",
                "blade",
                "叶片",
            ],
        )
    {
        return "Blade".to_string();
    }
    if contains_any(
        haystack,
        &[
            "blade",
            "叶片",
            "裂纹",
            "leading edge",
            "前缘",
            "叶尖",
            "雷击",
        ],
    ) {
        return "Blade".to_string();
    }
    if contains_any(haystack, &["gearbox", "齿轮箱", "油温", "轴承", "bearing"]) {
        return "Gearbox".to_string();
    }
    if contains_any(haystack, &["generator", "发电机", "绝缘", "定子", "转子"]) {
        return "Generator".to_string();
    }
    if contains_any(haystack, &["converter", "变流器", "逆变", "整流", "igbt"]) {
        return "Converter".to_string();
    }
    if contains_any(
        haystack,
        &[
            "scada",
            "scada_rules",
            "风速",
            "功率曲线",
            "报警",
            "运行数据",
            "power curve",
        ],
    ) {
        return "SCADA".to_string();
    }
    if contains_any(haystack, &["yaw", "偏航"]) {
        return "Yaw".to_string();
    }
    if contains_any(haystack, &["pitch", "变桨"]) {
        return "Pitch".to_string();
    }
    if contains_any(haystack, &["hydraulic", "液压"]) {
        return "Hydraulic".to_string();
    }
    if contains_any(haystack, &["tower", "塔筒"]) {
        return "Tower".to_string();
    }
    if contains_any(haystack, &["foundation", "基础", "锚栓"]) {
        return "Foundation".to_string();
    }
    if contains_any(haystack, &["vibration", "振动", "频谱"]) {
        return "Gearbox".to_string();
    }
    if contains_any(
        haystack,
        &["safety", "高压", "吊装", "并网", "工作票", "安全"],
    ) {
        return "Safety".to_string();
    }
    if contains_any(haystack, &["maintenance", "维护", "检修", "工单", "备件"]) {
        return "Maintenance".to_string();
    }
    "Maintenance".to_string()
}

fn classify_equipment(haystack: &str, file_type: FileType) -> String {
    if contains_any(haystack, &["thermal", "红外", "热像"]) {
        return "thermal".to_string();
    }
    if contains_any(haystack, &["blade", "叶片", "裂纹", "leading edge"]) {
        return "blade".to_string();
    }
    if file_type == FileType::Mp4 || contains_any(haystack, &["uav", "无人机"]) {
        return "uav".to_string();
    }
    if contains_any(haystack, &["gearbox", "齿轮箱"]) {
        return "gearbox".to_string();
    }
    if contains_any(haystack, &["generator", "发电机"]) {
        return "generator".to_string();
    }
    if contains_any(haystack, &["vibration", "振动", "频谱"]) {
        return "vibration".to_string();
    }
    if contains_any(haystack, &["converter", "变流器"]) {
        return "converter".to_string();
    }
    if contains_any(haystack, &["yaw", "偏航"]) {
        return "yaw_system".to_string();
    }
    if contains_any(haystack, &["pitch", "变桨"]) {
        return "pitch_system".to_string();
    }
    if contains_any(haystack, &["hydraulic", "液压"]) {
        return "hydraulic_system".to_string();
    }
    if contains_any(haystack, &["tower", "塔筒"]) {
        return "tower".to_string();
    }
    if contains_any(haystack, &["foundation", "基础"]) {
        return "foundation".to_string();
    }
    if contains_any(haystack, &["scada", "报警", "功率曲线"]) {
        return "scada".to_string();
    }
    "turbine".to_string()
}

fn classify_source_type(key_path: &str, file_type: FileType) -> String {
    let key = key_path.to_lowercase();
    if file_type.is_reserved_media() {
        return "reserved_media".to_string();
    }
    if key.contains("manuals/") {
        "manual".to_string()
    } else if key.contains("fault_cases/") {
        "fault_case".to_string()
    } else if key.contains("inspection_reports/") || key.contains("uav_inspection/") {
        "inspection_report".to_string()
    } else if key.contains("maintenance_records/") {
        "maintenance_record".to_string()
    } else if key.contains("regulations/") {
        "regulation".to_string()
    } else if key.contains("vibration_analysis/") {
        "vibration_analysis".to_string()
    } else if key.contains("scada_rules/") {
        "scada_rule".to_string()
    } else if key.contains("safety_rules/") {
        "safety_rule".to_string()
    } else {
        "document".to_string()
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn extract_docx_text(path: &Path) -> Result<String, String> {
    let mut zip = open_zip(path)?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|e| format!("docx missing word/document.xml in {}: {e}", path.display()))?
        .read_to_string(&mut xml)
        .map_err(|e| format!("read docx xml {}: {e}", path.display()))?;
    Ok(normalize_text(&strip_xml_tags(&xml)))
}

fn extract_xlsx_text(path: &Path) -> Result<String, String> {
    let mut zip = open_zip(path)?;
    let mut out = String::new();

    if let Ok(mut shared) = zip.by_name("xl/sharedStrings.xml") {
        let mut xml = String::new();
        shared
            .read_to_string(&mut xml)
            .map_err(|e| format!("read xlsx shared strings {}: {e}", path.display()))?;
        out.push_str(&extract_xml_text_nodes(&xml));
    }

    let names: Vec<String> = zip.file_names().map(ToOwned::to_owned).collect();
    for name in names {
        if !name.starts_with("xl/worksheets/") || !name.ends_with(".xml") {
            continue;
        }
        let mut xml = String::new();
        zip.by_name(&name)
            .map_err(|e| format!("read xlsx worksheet {name}: {e}"))?
            .read_to_string(&mut xml)
            .map_err(|e| format!("read xlsx worksheet {name}: {e}"))?;
        out.push('\n');
        out.push_str(&extract_xml_text_nodes(&xml));
    }

    Ok(normalize_text(&out))
}

fn open_zip(path: &Path) -> Result<ZipArchive<Cursor<Vec<u8>>>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("open zip {}: {e}", path.display()))
}

fn extract_pdf_text(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let text = extract_pdf_literal_strings(&bytes);
    Ok(normalize_text(&text))
}

fn extract_pdf_literal_strings(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'(' {
            i += 1;
            continue;
        }
        i += 1;
        let mut depth = 1;
        let mut value = Vec::new();
        while i < bytes.len() && depth > 0 {
            let b = bytes[i];
            if b == b'\\' && i + 1 < bytes.len() {
                value.push(bytes[i + 1]);
                i += 2;
                continue;
            }
            if b == b'(' {
                depth += 1;
                value.push(b);
            } else if b == b')' {
                depth -= 1;
                if depth > 0 {
                    value.push(b);
                }
            } else {
                value.push(b);
            }
            i += 1;
        }
        let s = String::from_utf8_lossy(&value);
        if s.chars().any(|c| c.is_alphanumeric()) {
            out.push_str(&s);
            out.push('\n');
        }
    }
    out
}

fn extract_xml_text_nodes(xml: &str) -> String {
    let mut out = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find('>') {
        rest = &rest[start + 1..];
        let Some(end) = rest.find('<') else {
            break;
        };
        let text = decode_xml_entities(rest[..end].trim());
        if !text.is_empty() {
            out.push_str(&text);
            out.push('\n');
        }
        rest = &rest[end..];
    }
    out
}

fn strip_xml_tags(xml: &str) -> String {
    extract_xml_text_nodes(xml)
}

fn decode_xml_entities(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn normalize_text(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[must_use]
pub fn default_knowledge_base_path() -> PathBuf {
    let relative = PathBuf::from("beifeng")
        .join("knowledge")
        .join("knowledge_base");
    if relative.is_dir() {
        return relative;
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors().take(6) {
            let candidate = ancestor.join(&relative);
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    let legacy = PathBuf::from("knowledge_base");
    if legacy.is_dir() {
        return legacy;
    }
    relative
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_supported_file_types() {
        assert_eq!(detect_file_type(Path::new("a.pdf")), Some(FileType::Pdf));
        assert_eq!(detect_file_type(Path::new("a.docx")), Some(FileType::Docx));
        assert_eq!(detect_file_type(Path::new("a.xlsx")), Some(FileType::Xlsx));
        assert_eq!(detect_file_type(Path::new("a.csv")), Some(FileType::Csv));
        assert_eq!(detect_file_type(Path::new("a.txt")), Some(FileType::Txt));
        assert_eq!(
            detect_file_type(Path::new("a.md")),
            Some(FileType::Markdown)
        );
        assert_eq!(detect_file_type(Path::new("a.json")), Some(FileType::Json));
        assert_eq!(detect_file_type(Path::new("a.jpg")), Some(FileType::Jpg));
        assert_eq!(detect_file_type(Path::new("a.jpeg")), Some(FileType::Jpeg));
        assert_eq!(detect_file_type(Path::new("a.png")), Some(FileType::Png));
        assert_eq!(detect_file_type(Path::new("a.mp4")), Some(FileType::Mp4));
        assert_eq!(detect_file_type(Path::new("a.bin")), None);
    }

    #[test]
    fn classifies_blade_and_scada_rules() {
        let blade = classify_document(
            Path::new("knowledge_base/uav_inspection/blade_crack.md"),
            "repo:knowledge_base/uav_inspection/blade_crack.md",
            FileType::Markdown,
            Some("leading edge crack"),
        );
        assert_eq!(blade.domain, "Blade");
        assert_eq!(blade.equipment, "blade");

        let scada = classify_document(
            Path::new("knowledge_base/scada_rules/power_curve.csv"),
            "repo:knowledge_base/scada_rules/power_curve.csv",
            FileType::Csv,
            Some("风速,功率曲线,报警"),
        );
        assert_eq!(scada.domain, "SCADA");
        assert_eq!(scada.equipment, "scada");
    }

    #[test]
    fn reserved_media_has_no_text() {
        let temp = tempfile::tempdir().expect("temp dir");
        let image = temp.path().join("uav_blade.jpg");
        std::fs::write(&image, [0_u8, 1, 2, 3]).expect("write image");

        let record = parse_document(&image, "repo:knowledge_base/uav_inspection/uav_blade.jpg")
            .expect("media should classify");

        assert!(record.text.is_none());
        assert!(record.metadata.reserved_media);
        assert_eq!(record.metadata.parser_status, "reserved_media_only");
        assert_eq!(record.metadata.domain, "Blade");
    }

    #[test]
    fn simple_pdf_literal_strings_are_extracted() {
        let pdf = b"%PDF-1.4\n1 0 obj\n<<>>\nstream\nBT (Gearbox bearing oil temperature) Tj ET\nendstream\nendobj\n%%EOF";
        let text = extract_pdf_literal_strings(pdf);
        assert!(text.contains("Gearbox bearing oil temperature"));
    }

    #[test]
    fn parses_minimal_docx_text() {
        let temp = tempfile::tempdir().expect("temp dir");
        let docx = temp.path().join("blade_manual.docx");
        write_zip_file(
            &docx,
            &[(
                "word/document.xml",
                r#"<w:document><w:body><w:p><w:r><w:t>Blade leading edge inspection</w:t></w:r></w:p></w:body></w:document>"#,
            )],
        );

        let record = parse_document(&docx, "repo:knowledge_base/manuals/blade_manual.docx")
            .expect("docx should parse");

        let text = record.text.expect("docx text");
        assert!(text.contains("Blade leading edge inspection"));
        assert_eq!(record.metadata.file_type, "docx");
        assert_eq!(record.metadata.domain, "Blade");
    }

    #[test]
    fn parses_minimal_xlsx_text() {
        let temp = tempfile::tempdir().expect("temp dir");
        let xlsx = temp.path().join("scada_export.xlsx");
        write_zip_file(
            &xlsx,
            &[
                (
                    "xl/sharedStrings.xml",
                    r#"<sst><si><t>风速</t></si><si><t>功率曲线</t></si></sst>"#,
                ),
                (
                    "xl/worksheets/sheet1.xml",
                    r#"<worksheet><sheetData><row><c><v>12.4</v></c><c><v>1800</v></c></row></sheetData></worksheet>"#,
                ),
            ],
        );

        let record = parse_document(&xlsx, "repo:knowledge_base/scada_rules/scada_export.xlsx")
            .expect("xlsx should parse");

        let text = record.text.expect("xlsx text");
        assert!(text.contains("风速"));
        assert!(text.contains("1800"));
        assert_eq!(record.metadata.file_type, "xlsx");
        assert_eq!(record.metadata.domain, "SCADA");
    }

    fn write_zip_file(path: &Path, entries: &[(&str, &str)]) {
        let file = std::fs::File::create(path).expect("create zip");
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        for (name, body) in entries {
            zip.start_file(name, options).expect("start zip entry");
            zip.write_all(body.as_bytes()).expect("write zip entry");
        }
        zip.finish().expect("finish zip");
    }
}
