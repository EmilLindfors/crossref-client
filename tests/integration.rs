//! Tests that talk to the live crossref api.
//!
//! They share one client, and therefore one rate limiter, so the suite paces
//! itself against the budget crossref reports rather than taking turns through
//! a mutex as it used to.

use crossref_client::query::ResultControl;
use crossref_client::{
    Crossref, Error, FieldQuery, JournalsQuery, Type, WorkElement, WorkResultControl, WorksFilter,
    WorksIdentQuery, WorksQuery,
};
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
fn journals_can_be_found_by_title() {
    api_test(|client| async move {
        let journals = client
            .journals(JournalsQuery::new("Economic Geography").result_control(ResultControl::Rows(10)))
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
            .works(WorksQuery::empty().result_control(WorkResultControl::Standard(
                ResultControl::Rows(10_000),
            )))
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
        client.works(WorksQuery::empty()).await.expect("a work list");

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
