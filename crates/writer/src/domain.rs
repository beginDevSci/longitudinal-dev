use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A complete tutorial document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub title: String,
    pub author: Author,
    pub metadata: Metadata,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
}

/// Tutorial metadata matching parser frontmatter requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    // Optional basic fields
    pub date_iso: Option<String>,
    pub tags: Vec<String>,

    // Method classification fields (all optional per parser)
    pub family: Option<String>,       // e.g., "LGCM", "GLMM"
    pub family_label: Option<String>, // e.g., "Latent Growth Curve Models (LGCM)"
    pub engine: Option<String>,       // e.g., "lavaan", "lme4"
    pub covariates: Option<String>,   // e.g., "None", "TIC", "TVC", "Both"
    pub outcome_type: Option<String>, // e.g., "Continuous", "Count", "Binary"

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            date_iso: Some(now.format("%Y-%m-%d").to_string()),
            tags: Vec::new(),
            family: None,
            family_label: None,
            engine: None,
            covariates: None,
            outcome_type: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A tutorial section (must be one of the 6 required H1 sections)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Section {
    pub id: String,
    pub section_type: SectionType,
    pub blocks: Vec<Block>,
}

impl Section {
    pub fn new(section_type: SectionType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            section_type,
            blocks: vec![Block::Paragraph {
                content: String::new(),
            }],
        }
    }

    pub fn has_content(&self) -> bool {
        self.blocks.iter().any(|block| block.has_content())
    }
}

/// The 6 required H1 sections in exact order
/// CRITICAL: Parser requires exactly these 6 sections in this order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SectionType {
    Overview,
    DataAccess, // CORRECTED: Was "Setup" in source
    DataPreparation,
    StatisticalAnalysis,
    Discussion,
    AdditionalResources,
}

impl SectionType {
    /// Returns the exact H1 heading text required by parser
    pub fn heading(&self) -> &'static str {
        match self {
            SectionType::Overview => "Overview",
            SectionType::DataAccess => "Data Access", // CORRECTED
            SectionType::DataPreparation => "Data Preparation",
            SectionType::StatisticalAnalysis => "Statistical Analysis",
            SectionType::Discussion => "Discussion",
            SectionType::AdditionalResources => "Additional Resources",
        }
    }

    /// All 6 required sections in the exact order parser expects
    pub fn all_required() -> &'static [SectionType] {
        &[
            SectionType::Overview,
            SectionType::DataAccess,
            SectionType::DataPreparation,
            SectionType::StatisticalAnalysis,
            SectionType::Discussion,
            SectionType::AdditionalResources,
        ]
    }
}

impl std::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.heading())
    }
}

/// Content blocks within sections
/// NOTE: This is simplified v1 structure. v2 blocks (Code/Output/Note with metadata)
/// can be added in future iterations per the roadmap.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Block {
    Paragraph {
        content: String,
    },
    Code {
        language: String,
        code: String,
        description: Option<String>,
        filename: Option<String>,
    },
    List {
        ordered: bool,
        items: Vec<String>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Image {
        src: String,
        alt: String,
    },
    Note {
        content: String,
    },
}

impl Block {
    pub fn has_content(&self) -> bool {
        match self {
            Block::Paragraph { content } => !content.trim().is_empty(),
            Block::Code { code, .. } => !code.trim().is_empty(),
            Block::List { items, .. } => {
                !items.is_empty() && items.iter().any(|item| !item.trim().is_empty())
            }
            Block::Table { rows, .. } => {
                !rows.is_empty()
                    && rows
                        .iter()
                        .any(|row| row.iter().any(|cell| !cell.trim().is_empty()))
            }
            Block::Image { src, .. } => !src.trim().is_empty(),
            Block::Note { content } => !content.trim().is_empty(),
        }
    }
}

impl Tutorial {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: String::new(),
            author: Author {
                name: String::new(),
            },
            metadata: Metadata::default(),
            sections: SectionType::all_required()
                .iter()
                .map(|st| Section::new(*st))
                .collect(),
        }
    }

    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = Utc::now();
    }
}

impl Default for Tutorial {
    fn default() -> Self {
        Self::new()
    }
}
