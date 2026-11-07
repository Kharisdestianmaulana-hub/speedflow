use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Documents,
    Compressed,
    Video,
    Audio,
    Programs,
    Images,
    General,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Documents => "Documents",
            Self::Compressed => "Compressed",
            Self::Video => "Video",
            Self::Audio => "Audio",
            Self::Programs => "Programs",
            Self::Images => "Images",
            Self::General => "General",
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "tgz" | "zst" => Self::Compressed,
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "ts" => Self::Video,
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" | "wma" => Self::Audio,
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => Self::Images,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "epub" | "csv" => {
                Self::Documents
            }
            "exe" | "msi" | "apk" | "dmg" | "iso" | "deb" | "rpm" | "bin" | "appimage" => {
                Self::Programs
            }
            _ => Self::General,
        }
    }

    pub fn subfolder_name(&self) -> Option<&'static str> {
        match self {
            Self::Compressed => Some("Compressed"),
            Self::Video => Some("Video"),
            Self::Audio => Some("Music"),
            Self::Programs => Some("Programs"),
            Self::Documents => Some("Documents"),
            Self::Images => Some("Images"),
            Self::General => None,
        }
    }

    /// Strips any trailing category subfolder from a path to get the base Downloads directory
    pub fn strip_category_subfolder(path: &Path) -> std::path::PathBuf {
        const KNOWN_SUBFOLDERS: [&str; 6] = [
            "Compressed",
            "Video",
            "Music",
            "Programs",
            "Documents",
            "Images",
        ];
        if let Some(last) = path.file_name().and_then(|s| s.to_str()) {
            if KNOWN_SUBFOLDERS.iter().any(|k| k.eq_ignore_ascii_case(last)) {
                if let Some(parent) = path.parent() {
                    return parent.to_path_buf();
                }
            }
        }
        path.to_path_buf()
    }

    /// Resolves the target download folder with category subfolder if enabled
    pub fn resolve_category_folder(
        base_or_current_dir: &Path,
        filename: &str,
        auto_categorize: bool,
    ) -> std::path::PathBuf {
        let base_dir = Self::strip_category_subfolder(base_or_current_dir);
        if !auto_categorize {
            return base_dir;
        }
        let cat = Self::from_filename(filename);
        match cat.subfolder_name() {
            Some(sub) => base_dir.join(sub),
            None => base_dir,
        }
    }
}
