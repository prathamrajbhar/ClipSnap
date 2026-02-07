/// The type of content stored in a clipboard history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentType {
    Image,
    Text,
}

impl ContentType {
    /// Parse from database string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "image" => Some(ContentType::Image),
            "text" => Some(ContentType::Text),
            _ => None,
        }
    }

    /// Convert to database string.
    pub fn to_str(&self) -> &'static str {
        match self {
            ContentType::Image => "image",
            ContentType::Text => "text",
        }
    }
}

/// A single entry in the clipboard history.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub content_type: ContentType,
    pub image_data: Option<Vec<u8>>,
    pub thumbnail: Option<Vec<u8>>,
    pub text_content: Option<String>,
    pub created_at: i64,
    pub file_size: i64,
}

/// A rectangular screen region.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Information about a monitor.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Monitor {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}
