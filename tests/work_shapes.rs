//! Offline regression tests for the shapes crossref actually returns.
//!
//! Crossref validates member deposits loosely, so a field the api documents is
//! not necessarily a field every record carries. Each entry in the fixture is a
//! real work sampled from the live api that the response types used to reject
//! (or, before the hand-written parsers were replaced by serde derives, panic
//! on) -- one page of a deep-paging crawl was lost per record.

use std::collections::BTreeMap;

use crossref_client::Work;

const EDGE_CASES: &str = include_str!("fixtures/work_edge_cases.json");

fn edge_cases() -> BTreeMap<String, serde_json::Value> {
    serde_json::from_str(EDGE_CASES).expect("fixture is valid json")
}

fn parse(case: &str) -> Work {
    let cases = edge_cases();
    let value = cases
        .get(case)
        .unwrap_or_else(|| panic!("fixture has no `{case}` case"));

    Work::try_from(value.clone()).unwrap_or_else(|err| panic!("`{case}` failed to parse: {err}"))
}

#[test]
fn every_edge_case_parses() {
    for case in edge_cases().keys() {
        parse(case);
    }
}

#[test]
fn a_work_without_a_title_parses_with_an_empty_title_list() {
    assert!(parse("no_title").title.is_empty());
}

#[test]
fn the_member_deposited_fields_are_all_optional() {
    assert_eq!(None, parse("no_publisher").publisher);
    assert_eq!(None, parse("no_type").type_);
    assert_eq!(None, parse("no_member").member);
    assert!(
        parse("funder_without_name")
            .funder
            .expect("the work has funders")
            .iter()
            .any(|funder| funder.name.is_none())
    );
    assert!(
        parse("empty_affiliation")
            .author
            .expect("the work has authors")
            .iter()
            .flat_map(|author| author.affiliation.iter())
            .any(|affiliation| affiliation.name.is_none())
    );
}

#[test]
fn a_license_can_omit_its_start_date() {
    assert!(
        parse("license_without_start")
            .license
            .expect("the work has licenses")
            .iter()
            .any(|license| license.start.is_none())
    );
}

#[test]
fn a_content_domain_without_a_restriction_flag_reads_as_unrestricted() {
    let content_domain = parse("content_domain_without_restriction")
        .content_domain
        .expect("the work has a content domain");

    assert!(!content_domain.crossmark_restriction);
}

#[test]
fn an_assertion_explanation_can_be_a_url_object() {
    use crossref_client::response::work::Explanation;

    let explanations: Vec<_> = parse("assertion_explanation_object")
        .assertion
        .expect("the work has assertions")
        .into_iter()
        .filter_map(|assertion| assertion.explanation)
        .collect();

    assert!(
        explanations
            .iter()
            .any(|explanation| matches!(explanation, Explanation::Url { .. })),
        "expected a `{{\"URL\": …}}` explanation, got {explanations:?}"
    );
}
