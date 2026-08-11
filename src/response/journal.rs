use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::{JournalList, QueryResponse};


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Counts {
    pub total_dois: i32,
    pub current_dois: i32,
    pub backfile_dois: i32,
}

//#[derive(Debug, Deserialize, Serialize, Clone)]
//#[serde(rename_all = "kebab-case")]
//pub struct Coverage {
//    pub affiliations_current: f32,
//    pub similarity_checking_current: f32,
//    pub descriptions_current: f32,
//    pub ror_ids_current: f32,
//    pub references_backfie: f32,
//    pub funders_backfile: f32,
//    pub licenses_backfile: f32,
//    pub funders_current: f32,
//    pub affiliations_backfile: f32,
//    pub resource_links_backfile: f32,
//    pub orcids_backfile: f32,
//    pub update_policies_current: f32,
//    pub ror_ids_backfile: f32,
//    pub orcids_current: f32,
//    pub similarity_checking_backfile: f32,
//    pub descriptions_backfile: f32,
//    pub award_numbers_backfile: f32,
//    pub update_policies_backfile: f32,
//    pub licenses_current: f32,
//    pub award_numbers_current: f32,
//    pub abstracts_backfile: f32,
//    pub resource_links_current: f32,
//    pub abstracts_current: f32,
//    pub references_current: f32,
//}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Flags {
    pub deposits_abstracts_current: bool,
    pub deposits_orcids_current: bool,
    pub deposits: bool,
    pub deposits_affiliations_backfile: bool,
    pub deposits_update_policies_backfile: bool,
    pub deposits_award_numbers_current: bool,
    pub deposits_resource_links_current: bool,
    pub deposits_ror_ids_current: bool,
    pub deposits_articles: bool,
    pub deposits_affiliations_current: bool,
    pub deposits_funders_current: bool,
    pub deposits_references_backfile: bool,
    pub deposits_ror_ids_backfile: bool,
    pub deposits_abstracts_backfile: bool,
    pub deposits_licenses_backfile: bool,
    pub deposits_award_numbers_backfile: bool,
    pub deposits_descriptions_current: bool,
    pub deposits_references_current: bool,
    pub deposits_resource_links_backfile: bool,
    pub deposits_descriptions_backfile: bool,
    pub deposits_orcids_backfile: bool,
    pub deposits_funders_backfile: bool,
    pub deposits_update_policies_current: bool,
    pub deposits_licenses_current: bool,
}

//#[derive(Debug, Deserialize, Serialize, Clone)]
//#[serde(rename_all = "kebab-case")]
//pub struct CoverageType {
//    pub all: Coverage,
//    pub current: Coverage,
//    pub backfile: Coverage,
//}

/// The per-year DOI counts crossref reports for a journal.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Breakdowns {
    /// `(year, number of DOIs issued that year)` pairs, in the order crossref returned them
    #[serde(default)]
    pub dois_by_issued_year: Vec<(i64, i64)>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Journal {
    /// when crossref last checked the journal's status, if ever
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub last_status_check_time: Option<DateTime<Utc>>,
    /// total/current/backfile DOI counts; crossref returns `null` for journals it has not tallied
    pub counts: Option<Counts>,
    /// DOI counts per issued year; crossref returns `null` for journals it has not tallied
    pub breakdowns: Option<Breakdowns>,
    pub publisher: String,
    pub coverage: Option<serde_json::Value>,
    pub title: String,
    #[serde(default)]
    pub subjects: Vec<String>,
    pub coverage_type: Option<serde_json::Value>,
    /// the `deposits-*` flags crossref records for the journal, keyed by flag name
    pub flags: Option<HashMap<String, bool>>,
    #[serde(rename = "ISSN", default)]
    pub issn: Vec<String>,
    #[serde(default)]
    pub issn_type: Vec<IssnType>,
}

impl Journal {
    /// `(year, DOI count)` pairs sorted ascending by year.
    ///
    /// Empty if crossref reported no breakdown for this journal.
    pub fn dois_by_issued_year(&self) -> Vec<(i64, i64)> {
        let mut years = self
            .breakdowns
            .as_ref()
            .map(|b| b.dois_by_issued_year.clone())
            .unwrap_or_default();
        years.sort_by_key(|(year, _)| *year);
        years
    }
}

impl TryFrom<serde_json::Value> for Journal {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map_err(|error| Error::Serde { error })
    }
}

impl TryFrom<serde_json::Value> for JournalList {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct Raw {
            total_results: usize,
            items_per_page: Option<usize>,
            query: Option<QueryResponse>,
            items: Vec<Journal>,
        }

        let raw: Raw = serde_json::from_value(value).map_err(|error| Error::Serde { error })?;

        Ok(JournalList {
            total_results: raw.total_results,
            items_per_page: raw.items_per_page,
            query: raw.query,
            facets: HashMap::new(),
            items: raw.items,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IssnType {
    pub value: String,
    /// `print`, `electronic` or `link`; crossref leaves this `null` for some journals
    #[serde(rename = "type")]
    pub type_: Option<String>,
}
