//! Offline end-to-end tests of the request the client actually sends.
//!
//! The route tests inside the crate stop at the string a query renders to.
//! These point [`CrossrefBuilder::base_url`] at a socket and assert on the
//! request line that arrives, which is the only place the encoding, the
//! cursor threading and the rate-limit handling can be checked together.

use crossref_client::{
    AsyncIterator, Crossref, Error, LicensesQuery, ResultControl, WorkElement, WorkResultControl,
    WorksFilter, WorksQuery,
};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::mpsc::{Receiver, channel};
use std::thread;

/// A canned reply for the mock to hand back.
struct Reply {
    status: &'static str,
    headers: Vec<(&'static str, String)>,
    body: String,
}

impl Reply {
    /// `200 OK` with a json body.
    fn ok(body: impl Into<String>) -> Self {
        Reply {
            status: "200 OK",
            headers: Vec::new(),
            body: body.into(),
        }
    }

    /// `429 Too Many Requests`, asking to be retried after `seconds`.
    fn rate_limited(seconds: u32) -> Self {
        Reply {
            status: "429 Too Many Requests",
            headers: vec![("retry-after", seconds.to_string())],
            body: String::new(),
        }
    }

    /// Adds the budget headers crossref sends on every response.
    fn with_budget(mut self, limit: u32, interval: &str) -> Self {
        self.headers.push(("x-rate-limit-limit", limit.to_string()));
        self.headers
            .push(("x-rate-limit-interval", interval.to_string()));
        self.headers.push(("x-api-pool", "polite".to_string()));
        self
    }
}

/// Serves `replies` in order on a loopback port, one per request, and reports
/// the request line of each.
///
/// Returns the base url to point a client at and the channel the request lines
/// arrive on. The thread ends once every reply has been handed out, so a test
/// has to queue exactly as many as it triggers.
fn mock_api(replies: Vec<Reply>) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let base_url = format!("http://{}", listener.local_addr().unwrap());
    let (requests, received) = channel();

    thread::spawn(move || {
        for reply in replies {
            let (mut stream, _) = listener.accept().expect("a connection");
            let mut lines = BufReader::new(stream.try_clone().unwrap()).lines();

            let request_line = lines.next().expect("a request line").expect("valid utf8");
            // a `GET` has no body, so the headers run to the blank line
            for line in lines.by_ref() {
                if line.expect("valid utf8").is_empty() {
                    break;
                }
            }
            requests.send(request_line).expect("the test is listening");

            let mut response = format!("HTTP/1.1 {}\r\n", reply.status);
            for (name, value) in &reply.headers {
                response.push_str(&format!("{name}: {value}\r\n"));
            }
            response.push_str(&format!(
                "content-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                reply.body.len(),
                reply.body
            ));
            stream
                .write_all(response.as_bytes())
                .expect("a live socket");
        }
    });

    (base_url, received)
}

/// A `work-list` reply carrying one work and the given `next-cursor`.
fn work_list(next_cursor: Option<&str>) -> String {
    work_list_of(1, next_cursor)
}

/// A `work-list` reply carrying `items` works, each of which is a bare DOI --
/// every other field of a [`Work`](crossref_client::Work) is optional.
///
/// An empty page is what ends a deep-paging crawl, so `items` is what decides
/// how many requests an iterator makes.
fn work_list_of(items: usize, next_cursor: Option<&str>) -> String {
    let cursor = match next_cursor {
        Some(cursor) => format!("\"next-cursor\":\"{cursor}\","),
        None => String::new(),
    };
    let items: Vec<String> = (0..items)
        .map(|i| format!(r#"{{"DOI":"10.5555/{i}"}}"#))
        .collect();
    format!(
        r#"{{"status":"ok","message-type":"work-list","message-version":"1.0.0",
           "message":{{{cursor}"total-results":{},"items":[{}]}}}}"#,
        items.len(),
        items.join(",")
    )
}

/// The path and query of a request line, e.g. `/works?query=R%26D`.
fn path_of(request_line: &str) -> String {
    request_line
        .split_whitespace()
        .nth(1)
        .expect("a request target")
        .to_string()
}

fn client(base_url: &str) -> Crossref {
    Crossref::builder()
        .base_url(base_url)
        .build()
        .expect("a crossref client")
}

#[tokio::test]
async fn a_query_reaches_the_wire_percent_encoded() {
    let (base_url, requests) = mock_api(vec![Reply::ok(work_list(None))]);

    let _ = client(&base_url)
        .works(
            WorksQuery::new("R&D")
                .filter(WorksFilter::HasOrcid)
                .elements(vec![WorkElement::DOI, WorkElement::Title])
                .result_control(WorkResultControl::Standard(ResultControl::RowsOffset {
                    rows: 10,
                    offset: 20,
                })),
        )
        .await;

    assert_eq!(
        "/works?query=R%26D&filter=has-orcid:true&select=DOI,title&rows=10&offset=20",
        path_of(&requests.recv().unwrap())
    );
}

#[tokio::test]
async fn a_combined_route_carries_the_component_id() {
    let (base_url, requests) = mock_api(vec![Reply::ok(work_list(None))]);

    let _ = client(&base_url)
        .journal_works(WorksQuery::new("kelp").into_ident("0013-0095"))
        .await;

    assert_eq!(
        "/journals/0013-0095/works?query=kelp",
        path_of(&requests.recv().unwrap())
    );
}

#[tokio::test]
async fn deep_paging_threads_the_cursor_from_one_page_into_the_next() {
    // the empty second page ends the crawl, so there is no third request
    let (base_url, requests) = mock_api(vec![
        Reply::ok(work_list_of(2, Some("cursor-two"))),
        Reply::ok(work_list_of(0, Some("cursor-three"))),
    ]);
    let client = client(&base_url);

    let mut works = client.deep_page(WorksQuery::new("kelp")).into_work_iter();
    let mut crawled = 0;
    while let Some(work) = works.next().await {
        work.expect("a work");
        crawled += 1;
    }

    assert_eq!(2, crawled);
    let paths: Vec<String> = requests.into_iter().map(|line| path_of(&line)).collect();
    assert_eq!(
        vec![
            "/works?query=kelp&cursor=*".to_string(),
            "/works?query=kelp&cursor=cursor-two".to_string(),
        ],
        paths
    );
}

#[tokio::test]
async fn the_budget_crossref_reports_replaces_the_assumed_one() {
    let (base_url, _requests) = mock_api(vec![Reply::ok(work_list(None)).with_budget(50, "1s")]);
    let client = client(&base_url);

    assert_eq!(5, client.rate_limit().limit, "the assumed budget");
    assert_eq!(None, client.api_pool());

    let _ = client.works(WorksQuery::empty()).await;

    assert_eq!(50, client.rate_limit().limit);
    assert_eq!(Some("polite".to_string()), client.api_pool());
}

#[tokio::test(start_paused = true)]
async fn a_rate_limited_request_is_retried() {
    let (base_url, requests) = mock_api(vec![
        Reply::rate_limited(1),
        Reply::ok(work_list(None)).with_budget(50, "1s"),
    ]);

    let works = client(&base_url)
        .licenses(LicensesQuery::empty())
        .await
        .expect_err("a work-list is not a license-list");

    // the retry happened: the second reply is what produced that mismatch
    assert!(
        matches!(works, Error::UnexpectedItem { .. }),
        "expected the second reply to be parsed, got {works:?}"
    );
    assert_eq!(2, requests.into_iter().count());
}

#[tokio::test(start_paused = true)]
async fn a_request_rate_limited_past_the_retry_budget_gives_up() {
    let replies = (0..3).map(|_| Reply::rate_limited(1)).collect();
    let (base_url, requests) = mock_api(replies);

    let error = Crossref::builder()
        .base_url(base_url.as_str())
        .max_retries(2)
        .build()
        .expect("a crossref client")
        .works(WorksQuery::empty())
        .await
        .expect_err("crossref never let the request through");

    let Error::RateLimited { attempts, .. } = error else {
        panic!("expected a rate limit error, got {error:?}");
    };
    assert_eq!(3, attempts, "the first try plus two retries");
    assert_eq!(3, requests.into_iter().count());
}
