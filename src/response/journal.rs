use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// How many DOIs crossref holds for a journal.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Counts {
    /// every DOI, current and backfile
    pub total_dois: i32,
    /// DOIs for material published within the last two years
    pub current_dois: i32,
    /// DOIs for material published more than two years ago
    pub backfile_dois: i32,
}

/// The per-year DOI counts crossref reports for a journal.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Breakdowns {
    /// `(year, number of DOIs issued that year)` pairs, in the order crossref returned them
    #[serde(default)]
    pub dois_by_issued_year: Vec<(i64, i64)>,
}

/// response item for the `/journals/{issn}` route
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
    /// the publisher that deposits the journal
    pub publisher: String,
    /// what share of the journal's deposits carry each optional field
    ///
    /// Left untyped: crossref keeps adding fields to it, and a struct listing
    /// them would reject every journal deposited after the next one lands.
    pub coverage: Option<serde_json::Value>,
    /// the journal's title
    pub title: String,
    /// subject categories the journal is classified under
    #[serde(default)]
    pub subjects: Vec<String>,
    /// [`Journal::coverage`], split into current, backfile and all
    pub coverage_type: Option<serde_json::Value>,
    /// the `deposits-*` flags crossref records for the journal, keyed by flag name
    pub flags: Option<HashMap<String, bool>>,
    /// the journal's ISSNs, print and electronic
    #[serde(rename = "ISSN", default)]
    pub issn: Vec<String>,
    /// the same ISSNs, each labelled with which kind it is
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

/// an ISSN together with the kind of ISSN it is
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct IssnType {
    /// the identifier, in `xxxx-xxxx` form
    pub value: String,
    /// `print`, `electronic` or `link`; crossref leaves this `null` for some journals
    #[serde(rename = "type")]
    pub type_: Option<String>,
}
