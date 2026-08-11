use crate::error::{Error, Result};
use crate::query::works::{WorksCombiner, WorksIdentQuery};
use crate::query::{Component, CrossrefQuery, CrossrefRoute, ResourceComponent};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Defines [`Type`] together with everything that has to stay in step with it:
/// the identifier crossref knows each variant by, the label it displays, the
/// [`FromStr`] that reads one back, and [`Type::ALL`].
///
/// The vocabulary is crossref's own and changes -- `grant`, `database` and
/// `report-component` were added after this crate first modelled it, and
/// `standard-series` was dropped -- which is what
/// `the_type_vocabulary_matches_the_api` pins this list against.
macro_rules! define_types {
    ($($variant:ident => $id:literal / $label:literal,)*) => {
        /// One of the work types crossref registers, as listed by `/types`.
        ///
        /// ```
        /// # use crossref_client::Type;
        /// # use std::str::FromStr;
        /// assert_eq!(Type::JournalArticle, Type::from_str("journal-article").unwrap());
        /// assert_eq!("Journal Article", Type::JournalArticle.label());
        /// ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
        #[serde(tag = "id")]
        pub enum Type {
            $(
                #[doc = concat!("`", $id, "`, which crossref labels ", $label)]
                #[serde(rename = $id)]
                $variant,
            )*
        }

        impl Type {
            /// Every type crossref registers, in the order `/types` lists them.
            pub const ALL: &'static [Type] = &[$(Type::$variant,)*];

            /// the display-friendly label crossref gives this type
            pub fn label(&self) -> &'static str {
                match self { $(Type::$variant => $label,)* }
            }

            /// the identifier crossref knows this type by
            pub fn id(&self) -> &'static str {
                match self { $(Type::$variant => $id,)* }
            }
        }

        impl FromStr for Type {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self> {
                match s {
                    $($id => Ok(Type::$variant),)*
                    name => Err(Error::InvalidTypeName { name: name.to_string() }),
                }
            }
        }
    };
}

define_types! {
    BookSection => "book-section" / "Book Section",
    Monograph => "monograph" / "Monograph",
    ReportComponent => "report-component" / "Report Component",
    Report => "report" / "Report",
    PeerReview => "peer-review" / "Peer Review",
    BookTrack => "book-track" / "Book Track",
    JournalArticle => "journal-article" / "Journal Article",
    BookPart => "book-part" / "Part",
    Other => "other" / "Other",
    Book => "book" / "Book",
    JournalVolume => "journal-volume" / "Journal Volume",
    BookSet => "book-set" / "Book Set",
    ReferenceEntry => "reference-entry" / "Reference Entry",
    ProceedingsArticle => "proceedings-article" / "Proceedings Article",
    Journal => "journal" / "Journal",
    Component => "component" / "Component",
    BookChapter => "book-chapter" / "Book Chapter",
    ProceedingsSeries => "proceedings-series" / "Proceedings Series",
    ReportSeries => "report-series" / "Report Series",
    Proceedings => "proceedings" / "Proceedings",
    Database => "database" / "Database",
    Standard => "standard" / "Standard",
    ReferenceBook => "reference-book" / "Reference Book",
    PostedContent => "posted-content" / "Posted Content",
    JournalIssue => "journal-issue" / "Journal Issue",
    Dissertation => "dissertation" / "Dissertation",
    Grant => "grant" / "Grant",
    Dataset => "dataset" / "Dataset",
    BookSeries => "book-series" / "Book Series",
    EditedBook => "edited-book" / "Edited Book",
}

/// constructs the request payload for the `/types` route
///
/// The route takes `query`, `rows` and `offset`, but answers all three with the
/// same complete list -- there are only [30 types](Type::ALL), so crossref
/// serves them in one page and ignores the paging. This crate therefore models
/// the list, one type, and the works of one type, and nothing else.
#[derive(Debug, Clone)]
pub enum Types {
    /// every available type
    All,
    /// target a specific type at `/types/{id}`
    Identifier(String),
    /// target a `Work` for a specific type at `/types/{id}/works?query..`
    Works(WorksIdentQuery),
}

impl CrossrefRoute for Types {
    fn route(&self) -> Result<String> {
        match self {
            Types::All => Component::Types.route(),
            Types::Identifier(s) => Ok(format!("{}/{}", Component::Types.route()?, s)),
            Types::Works(combined) => Self::combined_route(combined),
        }
    }
}

impl CrossrefQuery for Types {
    fn resource_component(self) -> ResourceComponent {
        ResourceComponent::Types(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_type_deserializes_from_the_record_the_api_sends() {
        let section = r#"{
    "id": "book-section",
    "label": "Book Section"
  }"#;
        let ref_type: Type = serde_json::from_str(section).unwrap();

        assert_eq!(Type::BookSection, ref_type);
    }

    #[test]
    fn every_type_round_trips_through_from_str() {
        for type_ in Type::ALL {
            assert_eq!(*type_, Type::from_str(type_.id()).expect("a known type"));
        }
    }

    #[test]
    fn a_type_crossref_no_longer_registers_is_not_a_type() {
        // `/types/standard-series` is a `404` -- crossref dropped it
        assert!(Type::from_str("standard-series").is_err());
    }
}
