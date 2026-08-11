//! Tests that talk to the live crossref api.
//!
//! They share one client, and therefore one rate limiter, so the suite paces
//! itself against the budget crossref reports rather than taking turns through
//! a mutex as it used to.

use crossref_client::query::ResultControl;
use crossref_client::{
    CnFormat, Crossref, Error, FieldQuery, FundersFilter, JournalsQuery, LicensesQuery,
    MembersFilter, Sort, Type, WorkElement, WorkResultControl, WorksFilter, WorksIdentQuery,
    WorksQuery,
};
use std::collections::BTreeSet;
use std::future::Future;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// The contact address to identify the suite to crossref with.
///
/// Whoever runs the tests, not whoever wrote them -- an address baked into the
/// file would attribute every contributor's traffic to one person. Unset, the
/// suite uses the anonymous pool, which is rate-limited harder but works; the
/// client paces itself against whichever budget it is granted.
fn contact_email() -> Option<String> {
    std::env::var("CROSSREF_MAILTO").ok()
}

/// Runs `test` against the shared client.
///
/// One runtime for the whole suite rather than the one-per-test that
/// `#[tokio::test]` would give: a [`Crossref`] outlives the runtime it was
/// built on badly, and sharing it is what lets a single limiter pace every
/// test.
fn api_test<F: Future<Output = ()>>(test: impl FnOnce(Crossref) -> F) {
    static SUITE: OnceLock<(Runtime, Crossref)> = OnceLock::new();

    let (runtime, client) = SUITE.get_or_init(|| {
        let runtime = Runtime::new().expect("a tokio runtime");
        let client = runtime.block_on(async {
            Crossref::builder()
                .polite(contact_email().as_deref())
                .build()
                .expect("a crossref client")
        });
        (runtime, client)
    });

    runtime.block_on(test(client.clone()));
}

/// Fetches `route`, waiting out a `429` rather than reading its empty body.
///
/// These requests do not go through the suite's client -- asking for a filter
/// that does not exist is exactly what the typed api prevents -- so they are
/// outside the budget it paces itself against and have to honour the limit
/// themselves.
async fn refused_body(route: &str) -> String {
    static PROBE: OnceLock<reqwest::Client> = OnceLock::new();

    let client = PROBE.get_or_init(|| {
        let agent = match contact_email() {
            Some(email) => format!("crossref-client (mailto:{email})"),
            None => "crossref-client".to_string(),
        };
        reqwest::Client::builder()
            .user_agent(agent)
            .build()
            .expect("a probe client")
    });

    for attempt in 0..5u32 {
        let response = client
            .get(format!("https://api.crossref.org/{route}"))
            .send()
            .await
            .expect("a response");

        if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
            return response.text().await.expect("a body");
        }

        let wait = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok())
            .unwrap_or(1 << attempt);
        tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
    }

    panic!("crossref rate limited `{route}` five times over");
}

/// The names crossref lists in the `400` it answers `route` with.
///
/// Every one of those messages ends in the vocabulary it does know, after the
/// last `: ` -- "Valid filters for this route are: a, b, c" for a filter, a
/// select or a field query, and "must be one of: a, b, c" for a sort field.
async fn vocabulary_crossref_reports(route: &str) -> Vec<String> {
    let body = refused_body(route).await;
    let body: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|err| panic!("`{route}` answered with `{body}`, which is not json: {err}"));

    let message = body["message"][0]["message"]
        .as_str()
        .unwrap_or_else(|| panic!("`{route}` was not refused with a message: {body}"));
    let (_, listed) = message
        .rsplit_once(": ")
        .unwrap_or_else(|| panic!("`{message}` lists no vocabulary"));

    listed
        .split(',')
        .map(|name| name.trim().to_string())
        .collect()
}

/// Diffs this crate's vocabulary for a route against the one crossref reports,
/// in both directions, and reports what does not line up.
///
/// The same check the unit tests make against a snapshot of these lists. This
/// is what catches crossref having changed one since the snapshot was taken:
/// a name added to a route that this crate cannot yet express, or one retired
/// that it still sends.
fn vocabulary_drift(kind: &str, ours: &[&str], theirs: &[String]) -> Vec<String> {
    let mine: BTreeSet<&str> = ours.iter().copied().collect();
    let theirs: BTreeSet<&str> = theirs.iter().map(String::as_str).collect();

    let rejected: Vec<_> = mine.difference(&theirs).collect();
    let unreachable: Vec<_> = theirs.difference(&mine).collect();

    let mut drift = Vec::new();
    if !rejected.is_empty() {
        drift.push(format!(
            "{kind} this crate sends that crossref no longer accepts: {rejected:?}"
        ));
    }
    if !unreachable.is_empty() {
        drift.push(format!(
            "{kind} crossref has added that this crate cannot express: {unreachable:?}"
        ));
    }
    drift
}

/// Checks every vocabulary this crate pins against the live api.
///
/// The unit tests compare these lists against a snapshot copied out of
/// crossref's own `400` bodies, which only catches a name this crate got wrong
/// -- not one crossref has since changed. This asks the api itself, by sending
/// each route something it has to refuse, and reads the list it answers with.
///
/// One test rather than one per vocabulary: the probes cannot go through the
/// client, so nothing paces them but running them in order.
#[test]
fn every_vocabulary_this_crate_pins_still_matches_the_api() {
    api_test(|client| async move {
        let select: Vec<&str> = WorkElement::ALL.iter().map(WorkElement::name).collect();
        let sort: Vec<&str> = Sort::ALL.iter().map(Sort::as_str).collect();

        let probes: [(&str, &str, &[&str]); 6] = [
            (
                "/works filters",
                "works?filter=not-a-filter:1",
                WorksFilter::ALL_NAMES,
            ),
            (
                "/works field queries",
                "works?query.not-a-field=x",
                FieldQuery::ALL_FIELDS,
            ),
            (
                "/works select elements",
                "works?select=not-an-element",
                &select,
            ),
            ("/works sort fields", "works?sort=not-a-sort", &sort),
            (
                "/funders filters",
                "funders?filter=not-a-filter:1",
                FundersFilter::ALL_NAMES,
            ),
            (
                "/members filters",
                "members?filter=not-a-filter:1",
                MembersFilter::ALL_NAMES,
            ),
        ];

        let mut drift = Vec::new();
        for (kind, route, ours) in probes {
            drift.extend(vocabulary_drift(
                kind,
                ours,
                &vocabulary_crossref_reports(route).await,
            ));
        }

        // the work types are a list route rather than a rejection, so they come
        // back through the client like any other response
        let types = client.types().await.expect("a type list");
        let ids: Vec<String> = types.items.iter().map(|type_| type_.id.clone()).collect();
        let ours: Vec<&str> = Type::ALL.iter().map(Type::id).collect();
        drift.extend(vocabulary_drift("work types", &ours, &ids));

        // the label is display text and crossref rewords it -- `book-part` was
        // "Book Part" until it became "Part"
        for type_ in &types.items {
            if let Ok(known) = type_.id.parse::<Type>() {
                if known.label() != type_.label {
                    drift.push(format!(
                        "crossref relabelled `{}` from `{}` to `{}`",
                        type_.id,
                        known.label(),
                        type_.label
                    ));
                }
            }
        }

        assert!(
            drift.is_empty(),
            "the api has moved on:\n{}",
            drift.join("\n")
        );
    });
}

#[test]
fn works_can_be_found_by_container_title() {
    api_test(|client| async move {
        let works = client
            .works(
                WorksQuery::empty()
                    .field_query(FieldQuery::container_title("Economic Geography"))
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(1))),
            )
            .await
            .expect("a work list");

        let work = works.items.into_iter().next().expect("at least one work");
        assert!(
            work.container_title
                .unwrap()
                .contains(&"Economic Geography".to_string())
        );
    });
}

#[test]
fn a_journal_can_be_fetched_by_issn() {
    api_test(|client| async move {
        client.journal("0013-0095").await.expect("a journal");
    });
}

#[test]
fn a_work_can_be_fetched_by_doi() {
    api_test(|client| async move {
        client.work("10.5555/12345678").await.expect("a work");
    });
}

#[test]
fn works_can_be_found_by_author() {
    api_test(|client| async move {
        let works = client
            .works(
                WorksQuery::empty()
                    .field_query(FieldQuery::author("Emil Lindfors"))
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(1))),
            )
            .await
            .expect("a work list");

        let work = works.items.into_iter().next().expect("at least one work");
        assert!(
            work.author
                .unwrap()
                .iter()
                .any(|a| a.family.as_deref() == Some("Lindfors"))
        );
    });
}

#[test]
fn a_works_query_can_be_scoped_to_a_journal() {
    api_test(|client| async move {
        let works = client
            .journal_works(WorksIdentQuery {
                id: "0013-0095".to_string(),
                query: WorksQuery::empty()
                    .filter(WorksFilter::Type(Type::JournalArticle))
                    .sort(crossref_client::Sort::Created)
                    .order(crossref_client::Order::Desc)
                    .result_control(WorkResultControl::Standard(ResultControl::RowsOffset {
                        rows: 10,
                        offset: 20,
                    })),
            })
            .await
            .expect("a work list");

        let work = works.items.into_iter().next().expect("at least one work");
        assert!(
            work.container_title
                .unwrap()
                .contains(&"Economic Geography".to_string())
        );
    });
}

#[test]
fn selected_elements_narrow_the_response() {
    api_test(|client| async move {
        let works = client
            .works(
                WorksQuery::empty()
                    .elements(vec![WorkElement::DOI, WorkElement::Title])
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(5))),
            )
            .await
            .expect("a `select` response has none of the fields `Work` used to require");

        assert_eq!(5, works.items.len());
        for work in &works.items {
            assert!(!work.doi.is_empty());
            // everything that was not selected comes back empty
            assert_eq!(None, work.created);
            assert_eq!(None, work.publisher);
        }
    });
}

#[test]
fn a_select_that_leaves_the_doi_out_still_returns_works() {
    api_test(|client| async move {
        // `Work` requires a DOI, so the query asks for one whether or not the
        // caller did -- without that this comes back as a page of works none of
        // which deserialize
        let works = client
            .works(
                WorksQuery::empty()
                    .elements(vec![WorkElement::Title])
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(5))),
            )
            .await
            .expect("a work list selected without the DOI");

        assert_eq!(5, works.items.len());
        assert!(works.items.iter().all(|work| !work.doi.is_empty()));
    });
}

#[test]
fn journals_can_be_found_by_title() {
    api_test(|client| async move {
        let journals = client
            .journals(
                JournalsQuery::new("Economic Geography").result_control(ResultControl::Rows(10)),
            )
            .await
            .expect("a journal list");

        let journal = journals
            .items
            .into_iter()
            .next()
            .expect("at least one journal");
        assert!(journal.title.contains("Economic Geography"));
    });
}

#[test]
fn licenses_can_be_listed_with_their_work_counts() {
    api_test(|client| async move {
        let licenses = client
            .licenses(LicensesQuery::new("creative commons").result_control(ResultControl::Rows(5)))
            .await
            .expect("a license list");

        assert_eq!(5, licenses.items.len());
        assert!(licenses.items.iter().all(|l| l.work_count > 0));
    });
}

#[test]
fn the_peer_reviews_of_a_work_can_be_found_through_its_relations() {
    api_test(|client| async move {
        // an open-review journal registers each review as its own work, so the
        // history is `relation.type:is-review-of` plus `relation.object:<doi>`
        let reviews = client
            .works(
                WorksQuery::empty()
                    .filter(WorksFilter::RelationType("is-review-of".to_string()))
                    .filter(WorksFilter::RelationObject(
                        "10.5194/egusphere-2026-890".to_string(),
                    ))
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(20))),
            )
            .await
            .expect("a work list");

        assert!(
            !reviews.items.is_empty(),
            "both filters used to render as `:true` and match everything"
        );
        assert!(
            reviews
                .items
                .iter()
                .all(|work| work.review.is_some() && work.relation.is_some()),
            "every hit is a review carrying its own metadata"
        );
    });
}

#[test]
fn a_work_can_be_transformed_into_bibtex_and_a_formatted_citation() {
    api_test(|client| async move {
        let doi = "10.1037/0003-066X.59.1.29";

        let bibtex = client
            .transform(doi, &CnFormat::BibTex)
            .await
            .expect("bibtex");
        // returned verbatim, and crossref pads its bibtex with a leading space
        assert!(
            bibtex.trim_start().starts_with("@article"),
            "not bibtex: {bibtex}"
        );

        let citation = client
            .transform(doi, &CnFormat::bibliography("apa"))
            .await
            .expect("an apa citation");
        assert!(citation.contains("Ray, O."), "not a citation: {citation}");
    });
}

#[test]
fn an_unknown_citation_style_reports_crossrefs_own_message() {
    api_test(|client| async move {
        // a `406` with a bare `{code, message}` body, which is shaped nothing
        // like the `validation-failure` the query routes answer with
        let error = client
            .transform(
                "10.1037/0003-066X.59.1.29",
                &CnFormat::bibliography("not-a-style"),
            )
            .await
            .expect_err("crossref has no such style");

        let Error::ValidationFailure { failures } = error else {
            panic!("expected a validation failure, got {error:?}");
        };
        assert!(
            failures.to_string().contains("not-a-style"),
            "crossref's own message should survive: {failures}"
        );
    });
}

#[test]
fn the_styles_a_citation_can_be_rendered_in_are_listed() {
    api_test(|client| async move {
        let styles = client.styles().await.expect("a style list");

        assert!(styles.items.len() > 1_000);
        assert!(styles.items.iter().any(|style| style == "apa"));
    });
}

#[test]
fn the_filters_added_for_ror_awards_and_events_are_accepted() {
    api_test(|client| async move {
        // the coverage tests pin the names against what crossref reports; this
        // checks the values render into something the route actually takes
        client
            .works(
                WorksQuery::empty()
                    .filter(WorksFilter::HasRorId)
                    .filter(WorksFilter::HasAlias)
                    .filter(WorksFilter::UpdateType("correction".to_string()))
                    .filter(WorksFilter::GteAwardAmount(1_000))
                    .filter(WorksFilter::AlternativeId("x".to_string()))
                    .filter(WorksFilter::RelationType("is-review-of".to_string()))
                    .filter(WorksFilter::FromIssuedDate(
                        "2020-01-01".parse().expect("a date"),
                    ))
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(1))),
            )
            .await
            .expect("crossref accepts every one of these filters");
    });
}

#[test]
fn a_rejected_request_reports_what_crossref_objected_to() {
    api_test(|client| async move {
        // crossref caps `rows` at 1000 and says so in a `validation-failure`
        // body, which used to be flattened into an opaque `400`
        let error = client
            .works(
                WorksQuery::empty()
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(10_000))),
            )
            .await
            .expect_err("crossref rejects a 10 000 row page");

        let Error::ValidationFailure { failures } = error else {
            panic!("expected a validation failure, got {error:?}");
        };
        assert!(
            failures.to_string().contains("1000"),
            "crossref's own message should survive: {failures}"
        );
    });
}

#[test]
fn the_reported_budget_and_pool_reach_the_client() {
    api_test(|client| async move {
        client
            .works(WorksQuery::empty())
            .await
            .expect("a work list");

        let pool = client
            .api_pool()
            .expect("a pool, once a response has come back");
        // whatever crossref grants, it is never nothing
        assert!(client.rate_limit().limit > 0);

        if contact_email().is_some() {
            assert!(pool.starts_with("polite"), "landed in the `{pool}` pool");
        }
    });
}
