#[cfg(test)]
mod tests {
    use crossref_client::query::ResultControl;
    use crossref_client::{
        Crossref, CrossrefBuilder, FieldQuery, JournalsQuery, Type, WorkElement, WorkResultControl,
        WorksFilter, WorksIdentQuery, WorksQuery,
    };

    /// Contact address sent to crossref so these tests run in the polite pool
    /// rather than the shared anonymous one, which rate-limits at `429`.
    const CONTACT_EMAIL: &str = "emil@lindfors.no";

    /// Even the polite pool caps requests per second, so these tests take turns
    /// instead of firing concurrently and tripping `429`.
    static API_TURN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    async fn client() -> (Crossref, tokio::sync::MutexGuard<'static, ()>) {
        let turn = API_TURN.lock().await;
        let client = CrossrefBuilder::default()
            .polite(CONTACT_EMAIL)
            .build()
            .unwrap();
        (client, turn)
    }

    #[tokio::test]
    async fn test_journal_by_name() {
        let (client, _turn) = client().await;
        let response = client
            .works(
                WorksQuery::empty()
                    .field_query(FieldQuery::container_title("Economic Geography"))
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(1))),
            )
            .await;
        println!("{:?}", response);
        assert!(response.is_ok());
        let work = response.unwrap().items.into_iter().next().unwrap();

        assert!(work
            .container_title
            .unwrap()
            .contains(&"Economic Geography".to_string()));
    }

    #[tokio::test]
    async fn test_journal_by_issn() {
        let (client, _turn) = client().await;
        let jorunal = client.journal("0013-0095").await;
        println!("{:?}", jorunal);
        assert!(jorunal.is_ok());
    }

    #[tokio::test]
    async fn test_work_by_doi() {
        let (client, _turn) = client().await;
        let work = client.work("10.5555/12345678").await;
        println!("{:?}", work);
        assert!(work.is_ok());
    }

    #[tokio::test]
    async fn test_works_by_author() {
        let (client, _turn) = client().await;
        let response = client
            .works(
                WorksQuery::empty()
                    .field_query(FieldQuery::author("Emil Lindfors"))
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(1))),
            )
            .await;
        println!("{:?}", response);
        assert!(response.is_ok());
        let work = response.unwrap().items.into_iter().next().unwrap();

        assert!(work.author.unwrap().iter().any(|a| a
            .family
            .as_ref()
            .unwrap()
            .contains(&"Lindfors".to_string())));
    }

    #[tokio::test]
    async fn combined_query() {
        tracing_subscriber::fmt::init();
        let (client, _turn) = client().await;
        let span = tracing::info_span!("combined_query");
        let _guard = span.enter();
        let response = client
            .journal_works(WorksIdentQuery {
                id: "0013-0095".to_string(),
                query: WorksQuery::empty()
                    //.field_query(FieldQuery::container_title("Economic Geography"))
                    .filter(WorksFilter::Type(Type::JournalArticle))
                    .sort(crossref_client::Sort::Created)
                    .order(crossref_client::Order::Desc)
                    .result_control(WorkResultControl::Standard(ResultControl::RowsOffset { rows: 10, offset: 20 })),
                        
            })
            .await;
        println!("{:?}", response);
        assert!(response.is_ok());
        let work = response.unwrap().items.into_iter().next().unwrap();

        assert!(work
            .container_title
            .unwrap()
            .contains(&"Economic Geography".to_string()));
    }

    #[tokio::test]
    async fn selected_elements_narrow_the_response() {
        let (client, _turn) = client().await;
        let response = client
            .works(
                WorksQuery::empty()
                    .elements(vec![WorkElement::DOI, WorkElement::Title])
                    .result_control(WorkResultControl::Standard(ResultControl::Rows(5))),
            )
            .await;
        println!("{:?}", response);
        let works = response.expect("a `select` response has none of the fields `Work` used to require");
        assert_eq!(5, works.items.len());
        for work in &works.items {
            assert!(!work.doi.is_empty());
            // everything that was not selected comes back empty
            assert_eq!(None, work.created);
            assert_eq!(None, work.publisher);
        }
    }

    #[tokio::test]
    async fn journal_query() {
        let (client, _turn) = client().await;
        let response = client
            .journals(JournalsQuery::new("Economic Geography").result_control(ResultControl::Rows(10)))
            .await;
        println!("{:?}", response);
        assert!(response.is_ok());
        let journal = response.unwrap().items.into_iter().next().unwrap();

        assert!(journal
            .title
            .contains(&"Economic Geography".to_string()));
    }
}
