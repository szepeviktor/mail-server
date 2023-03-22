pub mod filter;
pub mod sort;

use roaring::RoaringBitmap;

use crate::{
    fts::{lang::LanguageDetector, Language},
    Serialize,
};

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    LowerThan,
    LowerEqualThan,
    GreaterThan,
    GreaterEqualThan,
    Equal,
}

#[derive(Debug)]
pub enum Filter {
    HasKeyword {
        field: u8,
        value: String,
    },
    HasKeywords {
        field: u8,
        value: String,
    },
    MatchValue {
        field: u8,
        op: Operator,
        value: Vec<u8>,
    },
    HasText {
        field: u8,
        text: String,
        language: Language,
        match_phrase: bool,
    },
    InBitmap {
        family: u8,
        field: u8,
        key: Vec<u8>,
    },
    DocumentSet(RoaringBitmap),
    And,
    Or,
    Not,
    End,
}

#[derive(Debug)]
pub enum Comparator {
    Field { field: u8, ascending: bool },
    DocumentSet { set: RoaringBitmap, ascending: bool },
}

pub struct ResultSet {
    results: RoaringBitmap,
    document_ids: RoaringBitmap,
}

pub struct SortedResultRet {
    pub position: i32,
    pub ids: Vec<u32>,
    pub found_anchor: bool,
}

impl Filter {
    pub fn new_condition(field: impl Into<u8>, op: Operator, value: impl Serialize) -> Self {
        Filter::MatchValue {
            field: field.into(),
            op,
            value: value.serialize(),
        }
    }

    pub fn eq(field: impl Into<u8>, value: impl Serialize) -> Self {
        Filter::MatchValue {
            field: field.into(),
            op: Operator::Equal,
            value: value.serialize(),
        }
    }

    pub fn lt(field: impl Into<u8>, value: impl Serialize) -> Self {
        Filter::MatchValue {
            field: field.into(),
            op: Operator::LowerThan,
            value: value.serialize(),
        }
    }

    pub fn le(field: impl Into<u8>, value: impl Serialize) -> Self {
        Filter::MatchValue {
            field: field.into(),
            op: Operator::LowerEqualThan,
            value: value.serialize(),
        }
    }

    pub fn gt(field: impl Into<u8>, value: impl Serialize) -> Self {
        Filter::MatchValue {
            field: field.into(),
            op: Operator::GreaterThan,
            value: value.serialize(),
        }
    }

    pub fn ge(field: impl Into<u8>, value: impl Serialize) -> Self {
        Filter::MatchValue {
            field: field.into(),
            op: Operator::GreaterEqualThan,
            value: value.serialize(),
        }
    }

    pub fn match_text(field: impl Into<u8>, mut text: String, mut language: Language) -> Self {
        let match_phrase = (text.starts_with('"') && text.ends_with('"'))
            || (text.starts_with('\'') && text.ends_with('\''));

        if !match_phrase && language == Language::Unknown {
            language = if let Some((l, t)) = text
                .split_once(':')
                .and_then(|(l, t)| (Language::from_iso_639(l)?, t.to_string()).into())
            {
                text = t;
                l
            } else {
                LanguageDetector::detect_single(&text)
                    .and_then(|(l, c)| if c > 0.3 { Some(l) } else { None })
                    .unwrap_or(Language::Unknown)
            };
        }

        Filter::HasText {
            field: field.into(),
            text,
            language,
            match_phrase,
        }
    }
}

impl Comparator {
    pub fn ascending(field: impl Into<u8>) -> Self {
        Self::Field {
            field: field.into(),
            ascending: true,
        }
    }

    pub fn descending(field: impl Into<u8>) -> Self {
        Self::Field {
            field: field.into(),
            ascending: false,
        }
    }
}
