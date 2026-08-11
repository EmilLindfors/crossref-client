//! Tests that talk to the live crossref api.
//!
//! They share one client, and therefore one rate limiter, so the suite paces
//! itself against the budget crossref reports rather than taking turns through
//! a mutex as it used to.

use crossref_client::query::ResultControl;
use crossref_client::{
    Crossref, FieldQuery, JournalsQuery, Type, WorkElement, WorkResultControl, WorksFilter,
    WorksIdentQuery, WorksQuery,
};
use std::future::Future;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// Contact address sent to crossref so these tests run in the polite pool
/// rather than the shared anonymous one, which rate-limits harder.
const CONTACT_EMAIL: &str = "emil@lindfors.no";

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
                .polite(CONTACT_EMAIL)
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
fn the_client_lands_in_the_polite_pool() {
    api_test(|client| async move {
        client.works(WorksQuery::empty()).await.expect("a work list");

        let pool = client.api_pool().expect("a pool, once a response has come back");
        assert!(pool.starts_with("polite"), "landed in the `{pool}` pool");
        // whatever crossref grants, it is never nothing
        assert!(client.rate_limit().limit > 0);
    });
}
