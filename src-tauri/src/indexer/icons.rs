use std::path::Path;
use windows_icons::get_icon_base64_by_path;

// Minimalist SVG icons (Base64 encoded) for generic file types
pub const ICON_FOLDER: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjYmJiYmJiIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTIyIDE5YTIgMiAwIDAgMS0yIDJINDIgMiAwIDAgMS0yLTJWNWEyIDIgMCAwIDEgMi0yaDVsMiAzaDlhMiAyIDAgMCAxIDIgMnoiLz48L3N2Zz4=";
pub const ICON_FILE: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjYmJiYmJiIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTEzIDJINDIuOGExIDIgMCAwIDAgMiAydjE2YTIgMiAwIDAgMCAyIDJIMThhMiAyIDAgMCAwIDItMmg2LjgiLz48cGF0aCBkPSJNMTMgMnY0YTIgMiAwIDAgMCAyIDJoNCIvPjwvc3ZnPg==";
pub const ICON_DOC: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjM2I4MmY2IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTE0IDJINmEyIDIgMCAwIDAtMiAydjE2YTIgMiAwIDAgMCAyIDJoMTJhMiAyIDAgMCAwIDItMlY4eiIvPjxwYXRoIGQ9Ik0xNCAydjZIOHY2aDgiLz48cGF0aCBkPSJNOSAxNWg2Ii8+PHBhdGggZD0iTTkgMTloNiIvPjwvc3ZnPg==";
pub const ICON_IMG: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjMTBkYjgxIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHJlY3QgeD0iMyIgeT0iMyIgd2lkdGg9IjE4IiBoZWlnaHQ9IjE4IiByeD0iMiIgcnk9IjIiLz48Y2lyY2xlIGN4PSI4LjUiIGN5PSI4LjUiIHI9IjEuNSIvPjxwYXRoIGQ9Ik0yMSAxNWwtNS01TDUgMjEiLz48L3N2Zz4=";
pub const ICON_VID: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjZjk3MzE2IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHJlY3QgeD0iMiIgeT0iMiIgd2lkdGg9IjIwIiBoZWlnaHQ9IjIwIiByeD0iMiIgcnk9IjIiLz48cGF0aCBkPSJNMTAgOEw4IDEybDUgNHoiLz48L3N2Zz4=";
pub const ICON_XLS: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjMTY2NTM0IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHJlY3QgeD0iMyIgeT0iMyIgd2lkdGg9IjE4IiBoZWlnaHQ9IjE4IiByeD0iMiIgcnk9IjIiLz48cGF0aCBkPSJNOSAzdjE4TTE1IDN2MThNMyA5aDE4TTMgMTVoMTgiLz48L3N2Zz4=";
pub const ICON_PPT: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjYmI0NDIyIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTIyIDNIMXYxOGgyMVYzem0tMiA0SDR2MTBoMTZWN3oiLz48L3N2Zz4=";
pub const ICON_CODE: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjOGI1Y2Y2IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBvbHlsaW5lIHBvaW50cz0iMTYgMTggMjIgMTIgMTYgNiIvPjxwb2x5bGluZSBwb2ludHM9IjggNiAyIDEyIDggMTgiLz48L3N2Zz4=";
pub const ICON_ZIP: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjZWFiMzA4IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTIxIDhWNWEyIDIgMCAwIDAtMi0ySDVhMiAyIDAgMCAwLTIgMnYzbTE4IDB2MTFhMiAyIDAgMCAxLTIgMkg1YTIgMiAwIDAgMS0yLTJWOG0xOCAwSDNtNy01djRtNC00djRtLTQgNHYybTQtMnYybS00IDJ2Mm00LTJ2MiIvPjwvc3ZnPg==";
pub const ICON_AUDIO: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjZWM0ODk5IiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PHBhdGggZD0iTTkgMThWNWwxMi0ydjEzIi8+PGNpcmNsZSBjeD0iNiIgY3k9IjE4IiByPSIzIi8+PGNpcmNsZSBjeD0iMTgiIGN5PSIxNiIgcj0iMyIvPjwvc3ZnPg==";

pub fn get_file_category_and_icon(path: &Path) -> (String, Option<String>) {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" | "doc" | "docx" | "txt" | "rtf" | "odt" | "md" | "markdown" => ("DOC".to_string(), Some(ICON_DOC.to_string())),
        "xlsx" | "xls" | "csv" | "tsv" => ("XLS".to_string(), Some(ICON_XLS.to_string())),
        "pptx" | "ppt" => ("PPT".to_string(), Some(ICON_PPT.to_string())),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico" => ("IMG".to_string(), Some(ICON_IMG.to_string())),
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => ("VID".to_string(), Some(ICON_VID.to_string())),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" => ("AUDIO".to_string(), Some(ICON_AUDIO.to_string())),
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" => ("ARCHIVE".to_string(), Some(ICON_ZIP.to_string())),
        "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "c" | "cpp" | "h" | "go" | "java" | "sql" | "sh" | "ps1" | "bat" => ("CODE".to_string(), Some(ICON_CODE.to_string())),
        _ => ("FILE".to_string(), Some(ICON_FILE.to_string())),
    }
}

pub fn get_app_icon(file_path: &Path) -> Option<String> {
    if let Ok(base64) = get_icon_base64_by_path(file_path.to_str()?) {
        return Some(base64);
    }
    None
}
