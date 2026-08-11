//! Content negotiation: a work's metadata in a format other than crossref json.
//!
//! Crossref re-serializes a registered work on request, so a DOI can be pulled
//! straight out as BibTeX, RIS, RDF or a formatted citation without going
//! through [`Work`](crate::Work) at all. Pass a [`CnFormat`] to
//! [`Crossref::transform`](crate::Crossref::transform); the format becomes the
//! `Accept` header and the body comes back verbatim.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A format crossref will re-serialize a work into.
///
/// Every variant here was checked against the live api. `application/vnd.datacite.datacite+xml`
/// is deliberately absent: crossref answers it with a `406`, since it only
/// serves formats for the DOIs it registers itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CnFormat {
    /// RDF/XML
    RdfXml,
    /// RDF in the Turtle syntax
    Turtle,
    /// [CSL](https://citationstyles.org) JSON, the standard citeproc shape
    CiteProcJson,
    /// crossref's older, non-standard citeproc flavour
    CiteProcJsonIsh,
    /// [RIS](https://en.wikipedia.org/wiki/RIS_(file_format)), for reference managers
    Ris,
    /// BibTeX
    BibTex,
    /// crossref's own deposit schema
    CrossrefXml,
    /// crossref's [text and data mining](https://www.crossref.org/documentation/retrieve-metadata/rest-api/text-and-data-mining/)
    /// schema, which carries the full-text links
    CrossrefTdm,
    /// a citation formatted for reading, rendered by crossref itself
    Bibliography {
        /// the [CSL style](https://api.crossref.org/styles) to render in, e.g.
        /// `apa` or `ieee`; crossref answers an unknown one with a `406`.
        ///
        /// List what it accepts with [`Crossref::styles`](crate::Crossref::styles).
        style: String,
        /// the locale to render in, e.g. `de-DE`. [`None`] leaves it to
        /// crossref, which uses `en-US`.
        locale: Option<String>,
    },
}

impl CnFormat {
    /// A citation in the named [CSL style](https://api.crossref.org/styles),
    /// rendered in crossref's default locale.
    pub fn bibliography(style: impl Into<String>) -> Self {
        CnFormat::Bibliography {
            style: style.into(),
            locale: None,
        }
    }

    /// The `Accept` header that asks crossref for this format.
    pub fn accept(&self) -> Cow<'_, str> {
        let simple = match self {
            CnFormat::RdfXml => "application/rdf+xml",
            CnFormat::Turtle => "text/turtle",
            CnFormat::CiteProcJson => "application/vnd.citationstyles.csl+json",
            CnFormat::CiteProcJsonIsh => "application/citeproc+json",
            CnFormat::Ris => "application/x-research-info-systems",
            CnFormat::BibTex => "application/x-bibtex",
            CnFormat::CrossrefXml => "application/vnd.crossref.unixref+xml",
            CnFormat::CrossrefTdm => "application/vnd.crossref.unixsd+xml",
            CnFormat::Bibliography { style, locale } => {
                let mut accept = format!("text/x-bibliography; style={style}");
                if let Some(locale) = locale {
                    accept.push_str("; locale=");
                    accept.push_str(locale);
                }
                return Cow::Owned(accept);
            }
        };
        Cow::Borrowed(simple)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of every variant, for the tests below.
    fn one_of_each() -> Vec<CnFormat> {
        vec![
            CnFormat::RdfXml,
            CnFormat::Turtle,
            CnFormat::CiteProcJson,
            CnFormat::CiteProcJsonIsh,
            CnFormat::Ris,
            CnFormat::BibTex,
            CnFormat::CrossrefXml,
            CnFormat::CrossrefTdm,
            CnFormat::bibliography("apa"),
        ]
    }

    #[test]
    fn no_two_formats_ask_for_the_same_thing() {
        let mut accepts: Vec<_> = one_of_each()
            .iter()
            .map(|f| f.accept().to_string())
            .collect();
        let total = accepts.len();
        accepts.sort();
        accepts.dedup();

        assert_eq!(
            total,
            accepts.len(),
            "duplicate `Accept` values: {accepts:?}"
        );
    }

    #[test]
    fn a_style_and_locale_become_accept_parameters() {
        assert_eq!(
            "text/x-bibliography; style=apa",
            CnFormat::bibliography("apa").accept()
        );

        assert_eq!(
            "text/x-bibliography; style=apa; locale=de-DE",
            CnFormat::Bibliography {
                style: "apa".to_string(),
                locale: Some("de-DE".to_string()),
            }
            .accept()
        );
    }
}
