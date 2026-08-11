//! Percent-encoding for the crossref query string.
//!
//! Routes used to be assembled by string concatenation, so a term containing
//! `&`, `=`, `#` or `%` terminated the parameter it was meant to be part of --
//! `WorksQuery::new("R&D")` produced `/works?query=R&D`, which crossref reads as
//! `query=R` plus a stray `D` parameter and answers with the results for `R`.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use std::borrow::Cow;

/// The characters that stay literal in a crossref query string.
///
/// Everything outside the RFC 3986 unreserved set is encoded, apart from the
/// punctuation crossref's own syntax is built from: `:` and `,` separate the
/// fragments of a `filter`/`facet` value, `/` appears in every DOI and `*` is
/// the initial deep-paging cursor. Crossref percent-decodes before it splits on
/// those, so keeping them literal is a matter of legible urls rather than of
/// correctness -- and by the same token a `,` *inside* a filter value cannot be
/// escaped in either form, because the api splits it after decoding.
const CROSSREF_QUERY: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b':')
    .remove(b',')
    .remove(b'/')
    .remove(b'*');

/// Percent-encodes a single key or value of the query string.
pub(crate) fn encode(value: &str) -> Cow<'_, str> {
    utf8_percent_encode(value, CROSSREF_QUERY).into()
}

/// Renders `params` as the query string of a route, percent-encoding each pair.
///
/// Returns an empty string for an empty slice, so callers can concatenate it
/// onto a route unconditionally; otherwise the result starts with `?`.
pub(crate) fn query_string(params: &[(Cow<'_, str>, Cow<'_, str>)]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let mut route = String::from("?");
    for (i, (key, value)) in params.iter().enumerate() {
        if i > 0 {
            route.push('&');
        }
        route.push_str(&encode(key));
        route.push('=');
        route.push_str(&encode(value));
    }
    route
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ampersand_can_no_longer_terminate_a_parameter() {
        assert_eq!("R%26D", &encode("R&D"));
    }

    #[test]
    fn the_reserved_characters_are_all_encoded() {
        assert_eq!("%26%3D%23%3F%25%2B%20", &encode("&=#?%+ "));
    }

    #[test]
    fn crossrefs_own_syntax_stays_literal() {
        assert_eq!(
            "type:journal-article,has-orcid:true",
            &encode("type:journal-article,has-orcid:true")
        );
        assert_eq!("10.1037/0003-066X.59.1.29", &encode("10.1037/0003-066X.59.1.29"));
        assert_eq!("*", &encode("*"));
    }

    #[test]
    fn non_ascii_terms_are_encoded_as_utf8() {
        assert_eq!("Ol%C3%A9", &encode("Olé"));
    }

    #[test]
    fn an_empty_parameter_list_has_no_question_mark() {
        assert_eq!("", &query_string(&[]));
    }

    #[test]
    fn pairs_are_joined_with_ampersands() {
        let params = [
            (Cow::Borrowed("query"), Cow::Borrowed("R&D")),
            (Cow::Borrowed("rows"), Cow::Borrowed("20")),
        ];

        assert_eq!("?query=R%26D&rows=20", &query_string(&params));
    }
}
