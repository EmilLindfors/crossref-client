//! An async client for the [Crossref REST API](https://api.crossref.org/swagger-ui/index.html).
//!
//! `crossref-client` is a hard fork of [`crossref-rs`](https://github.com/MattsSe/crossref-rs)
//! and has diverged substantially from it; changes are not upstreamed.
//!
//! The [`Crossref`] client provides methods matching the Crossref API routes:

//! * `works` - `/works` route
//! * `members` - `/members` route
//! * `prefixes` - `/prefixes` route
//! * `funders` - `/funders` route
//! * `journals` - `/journals` route
//! * `licenses` - `/licenses` route
//! * `types` - `/types` route
//! * `agency` - `/works/{doi}/agency` get DOI minting agency
//!
//! ## Usage

//! ### Create a `Crossref` client:

//! ```no_run
//! # use crossref_client::Crossref;
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder().build()?;
//! # Ok(())
//! # }
//! ```
//!
//! If you have an [Authorization token for Crossref's Plus service](https://github.com/CrossRef/rest-api-doc#authorization-token-for-plus-service):
//!
//! ```no_run
//! # use crossref_client::Crossref;
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder()
//! .token("token")
//! .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! Encouraged to use the **The Polite Pool**:
//!
//! [Good manners = more reliable service](https://github.com/CrossRef/rest-api-doc#good-manners--more-reliable-service)
//!
//! To get into Crossref's polite pool include a email address
//!
//! ```no_run
//! # use crossref_client::Crossref;
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder()
//!     .polite("polite@example.com")
//!     .token("your token")
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Constructing Queries
//! Not all components support queries and there are custom available parameters for each route that supports querying.
//! For each resource components that supports querying there exist a Query struct: `WorksQuery`, `MembersQuery`, `FundersQuery`. The `WorksQuery` also differs from the others by supporting [deep paging with cursors](https://github.com/CrossRef/rest-api-doc#deep-paging-with-cursors) and [field queries](https://github.com/CrossRef/rest-api-doc#works-field-queries).
//!
//! Otherwise creating queries works the same for all resource components:
//!
//! ```no_run
//! # use crossref_client::*;
//! # async fn run() -> crossref_client::Result<()> {
//! let query = WorksQuery::new("Machine Learning")
//! // field queries supported for `Works`
//! .field_query(FieldQuery::author("Some Author"))
//! // filters are specific for each resource component
//! .filter(WorksFilter::HasOrcid)
//! .order(Order::Asc)
//! .sort(Sort::Score);
//! # Ok(())
//! # }
//! ```
//!
//!
//! ### Get Records
//!
//! See [this table](https://github.com/CrossRef/rest-api-doc#resource-components) for a detailed overview of the major components.
//!
//! There are 3 available targets:
//!
//! * **standalone resource components**: `/works`, `/members`, etc. that return a list list of the corresponding items and can be specified with queries
//! * **Resource component with identifiers**: `/works/{doi}?<query>`,`/members/{member_id}?<query>`, etc. that returns a single item if found.
//! * **combined with the `works` route**: The works component can be appended to other resources: `/members/{member_id}/works?<query>` etc. that returns a list of matching `Work` items.
//!
//! This resembles in the enums of the resource components, eg. for `Members`:
//!
//! ```no_run
//! # use crossref_client::query::*;
//! pub enum Members {
//!     /// target a specific member at `/members/{id}`
//!     Identifier(String),
//!     /// target all members that match the query at `/members?query...`
//!     Query(MembersQuery),
//!     /// target a `Work` for a specific member at `/members/{id}/works?query..`
//!     Works(WorksIdentQuery),
//! }
//! ```
//!
//! All options are supported by the client:
//!
//! **Single Item by DOI (ID)**
//!
//! Analogous methods exist for all resource components
//!
//! ```no_run
//! # use crossref_client::*;
//! # async fn run() -> crossref_client::Result<()> {
//! # let client = Crossref::builder().build()?;
//! let work = client.work("10.1037/0003-066X.59.1.29").await?;
//!
//! let agency = client.work_agency("10.1037/0003-066X.59.1.29").await?;
//!
//! let funder = client.funder("funder_id").await?;
//!
//! let member = client.member("member_id").await?;
//! # Ok(())
//! # }
//! ```
//!
//! **Query**
//!
//! ```no_run
//! # use crossref_client::*;
//! # async fn run() -> crossref_client::Result<()> {
//! # let client = Crossref::builder().build()?;
//! let query = WorksQuery::new("Machine Learning");
//!
//! // one page of the matching results
//! let works = client.works(query).await?;
//! # Ok(())
//! # }
//! ```
//!
//! Alternatively insert a free form query term directly
//!
//! ```no_run
//! # use crossref_client::*;
//! # async fn run() -> crossref_client::Result<()> {
//! # let client = Crossref::builder().build()?;
//!
//! // one page of the matching results
//! let works = client.works("Machine Learning").await?;
//! # Ok(())
//! # }
//! ```
//!
//! **Combining Routes with the `Works` route**
//!
//! For each resource component other than `Works` there exist methods to append a `WorksQuery` with the ID option `/members/{member_id}/works?<query>?`
//!
//! ```no_run
//! # use crossref_client::*;
//! # async fn run() -> crossref_client::Result<()> {
//! # let client = Crossref::builder().build()?;
//! let works = client.member_works( WorksQuery::new("machine learning")
//! .sort(Sort::Score).into_ident("member_id")).await?;
//! # Ok(())
//! # }
//! ```
//!
//! This would be the same as using the [`Crossref::works`] method by supplying the combined type
//!
//! ```no_run
//! # use crossref_client::*;
//! # async fn run() -> crossref_client::Result<()> {
//! # let client = Crossref::builder().build()?;
//! let works = client.works(WorksQuery::new("machine learning")
//!     .sort(Sort::Score)
//!     .into_combined_query::<Members>("member_id")).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ** Deep paging for `Works` **
//! [Deep paging results](https://github.com/CrossRef/rest-api-doc#deep-paging-with-cursors)
//! Deep paging is supported for all queries, that return a list of `Work`, `WorkList`.
//! This function returns a new iterator over pages of `Work`, which is returned as bulk of items as a `WorkList` by crossref.
//! Usually a single page `WorkList` contains 20 items.
//!
//! # Example
//!
//! Iterate over all `Works` linked to search term `Machine Learning`
//!
//! ```no_run
//! use crossref_client::{AsyncIterator, Crossref, WorksQuery, Work};
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder().build()?;
//!
//! let mut pages = client.deep_page(WorksQuery::new("Machine Learning"));
//! let mut all_works: Vec<Work> = Vec::new();
//! while let Some(page) = pages.next().await {
//!     let page = page?;
//!     all_works.extend(page.items);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! Which can be simplified to
//!
//! ```no_run
//! use crossref_client::{AsyncIterator, Crossref, WorksQuery, Work};
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder().build()?;
//!
//! let mut works = client.deep_page("Machine Learning").into_work_iter();
//! let mut all_works: Vec<Work> = Vec::new();
//! while let Some(work) = works.next().await {
//!     let work = work?;
//!     all_works.push(work);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//!
//! # Example
//!
//! Iterate over all the pages (`WorkList`) of the funder with id `funder id` by using a combined query.
//! A single `WorkList` usually holds 20 `Work` items.
//!
//! ```no_run
//! use crossref_client::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder().build()?;
//!
//! let mut pages = client.deep_page(WorksQuery::default().into_combined_query::<Funders>("funder id"));
//! let mut all_funder_work_list: Vec<WorkList> = Vec::new();
//! while let Some(page) = pages.next().await {
//!     let page = page?;
//!     all_funder_work_list.push(page);
//! }
//!
//! # Ok(())
//! # }
//! ```
//! # Example
//!
//! Iterate over all `Work` items of a specfic funder directly.
//!
//! ```no_run
//! use crossref_client::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
//! # async fn run() -> Result<(), crossref_client::Error> {
//! let client = Crossref::builder().build()?;
//!
//! let mut works = client.deep_page(WorksQuery::default()
//!         .into_combined_query::<Funders>("funder id"))
//!         .into_work_iter();
//! let mut all_works: Vec<Work> = Vec::new();
//! while let Some(work) = works.next().await {
//!     let work = work?;
//!     all_works.push(work);
//! }
//!
//! # Ok(())
//! # }
//! ```

// TODO: 118 public items still lack doc comments; re-enable once documented.
// #![warn(missing_docs)]



mod error;
/// client side rate limiting
pub mod limit;
/// provides types to construct a specific query
pub mod query;
/// provides the response types of the crossref api
pub mod response;

/// content negotiation
pub mod cn;

#[doc(inline)]
pub use self::error::{Error, Result};

#[doc(inline)]
pub use self::limit::RateLimit;

#[doc(inline)]
pub use self::cn::CnFormat;

#[doc(inline)]
pub use self::query::works::{
    FieldQuery, WorkElement, WorkListQuery, WorkResultControl, Works, WorksFilter, WorksIdentQuery,
    WorksQuery,
};

#[doc(inline)]
pub use self::query::{Component, CrossrefQuery, CrossrefRoute, Order, ResultControl, Sort};
pub use self::query::facet::{Facet, FacetCount};
pub use self::query::{
    Funders, FundersQuery, Journals, JournalsQuery, Licenses, LicensesQuery, Members, MembersQuery,
    Prefixes, Type, Types, WorksComponent,
};
pub use self::response::{
    CrossrefType, Failure, Failures, Funder, FunderList, Journal, JournalList, LicenseCount,
    LicenseList, Member, MemberList, MessageType, StyleList, TypeList, Work, WorkAgency, WorkList,
};

/// The types that appear in the public fields of a [`Work`], re-exported so a
/// caller of [`Crossref::works`] can name them without reaching into
/// [`response::work`].
#[doc(inline)]
pub use self::response::work::{
    Affiliation, Agency, Assertion, AssertionGroup, ClinicalTrialNumber, ContentDomain,
    Contributor, Date, DateField, DateParts, Explanation, FundingBody, ISSN, InstitutionId, Issue,
    License, PartialDate, Reference, RelatedItems, Relation, Relations, ResourceLink, Review,
    Update,
};

pub(crate) use self::response::{Message, Response};

/// The async iterator trait implemented by [`WorkListIterator`].
///
/// Re-exported so callers can drive deep paging without depending on
/// `async-iterator` directly.
pub use async_iterator::Iterator as AsyncIterator;

use crate::limit::{Limiter, retry_after};
use crate::response::Prefix;
use reqwest::{Client, StatusCode};
use std::sync::Arc;

/// Unwraps the payload a route is expected to answer with, or reports what
/// crossref sent instead.
macro_rules! get_item {
    ($ident:ident, $response:expr) => {{
        let response = $response;
        match response.message {
            Message::$ident(item) => Ok(item),
            other => Err(Error::UnexpectedItem {
                expected: MessageType::$ident,
                got: other.message_type(),
            }),
        }
    }};
}

macro_rules! impl_combined_works_query {
    ($($name:ident  $component:ident,)*) => {
        $(
        /// Return one page of the components's `Work` that match the query
        ///
        pub async fn $name(&self, ident: WorksIdentQuery) -> Result<WorkList> {
            let resp = self.get_response(&$component::Works(ident)).await?;
            get_item!(WorkList, resp)
        })+
    };
}

/// The bare body crossref answers a content-negotiation request it cannot
/// serve with, which is shaped nothing like a `validation-failure`.
#[derive(serde::Deserialize)]
struct TransformFailure {
    /// what went wrong, e.g. `style-not-found`
    code: String,
    /// the DOI that was asked for
    doi: Option<String>,
    /// the explanation, e.g. `Style [vancouver] does not exist`
    message: String,
}

/// Struct for Crossref search API methods
///
/// Cloning is cheap and shares the connection pool and the rate limiter, so
/// concurrent callers should clone one client rather than build several -- see
/// [`Crossref::rate_limit`].
#[derive(Debug, Clone)]
pub struct Crossref {
    /// use another base url than `api.crossref.org`
    base_url: String,
    /// the reqwest client that handles the requests
    client: Client,
    /// paces requests against the budget crossref grants, shared between clones
    limiter: Arc<Limiter>,
    /// how many times a `429` is retried before giving up
    max_retries: u32,
}

impl Crossref {
    const BASE_URL: &'static str = "https://api.crossref.org";

    /// Constructs a new `CrossrefBuilder`.
    ///
    /// This is the same as `Crossref::builder()`.
    pub fn builder() -> CrossrefBuilder {
        CrossrefBuilder::new()
    }

    /// The url every route is appended to, `https://api.crossref.org` unless
    /// [`CrossrefBuilder::base_url`] said otherwise.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// The request budget crossref last reported for this client.
    ///
    /// Crossref sends the budget on every response and the client paces itself
    /// against it, so before the first request this is the assumed
    /// [`RateLimit::CONSERVATIVE`] (or whatever [`CrossrefBuilder::rate_limit`]
    /// was given) and afterwards it is what crossref actually granted.
    pub fn rate_limit(&self) -> RateLimit {
        self.limiter.rate()
    }

    /// The pool crossref sorted the last request into, from `x-api-pool`.
    ///
    /// `polite` if [`CrossrefBuilder::polite`] worked, `plus` for a Plus token,
    /// otherwise `public`. [`None`] before the first request.
    pub fn api_pool(&self) -> Option<String> {
        self.limiter.pool()
    }

    // generate all functions to query combined endpoints
    impl_combined_works_query!(funder_works Funders, member_works Members,
    type_works Types, journal_works Journals, prefix_works Prefixes,);

    /// Transforms the `CrossrefQuery` in the request route and  executes the request
    ///
    /// # Errors
    ///
    /// If crossref could not resolve the route, a `ResourceNotFound` error is returned.
    /// Also fails if the json response body could not be parsed into `Response`.
    /// Fails if there was an error in reqwest executing the request [::reqwest::RequestBuilder::send]
    async fn get_response<T: CrossrefQuery>(&self, query: &T) -> Result<Response> {
        let url = query.to_url(&self.base_url)?;
        let response = self.send(&url).await?;

        // crossref answers an unresolvable route with a plain-text body, which
        // would otherwise surface as an opaque deserialization failure
        if response.status() == StatusCode::NOT_FOUND {
            return Err(Error::ResourceNotFound {
                resource: Box::new(query.clone().resource_component()),
            });
        }

        // deserialized here rather than through `reqwest`'s `json()` so a
        // shape crossref changed surfaces as `Error::Serde`, which names the
        // field that failed
        let body = Self::into_success(response).await?.bytes().await?;
        serde_json::from_slice(&body).map_err(|error| Error::Serde { error })
    }

    /// Sends a `GET`, pacing it against the rate limit and retrying a `429`.
    ///
    /// Waits for a slot from the shared [`Limiter`] first, then feeds the
    /// budget crossref reports back into it. A `429` means the budget we
    /// believed in was wrong, so the whole client backs off -- for as long as
    /// `retry-after` asks, or exponentially from the reported interval -- and
    /// the request is sent again up to [`CrossrefBuilder::max_retries`] times.
    async fn send(&self, url: &str) -> Result<reqwest::Response> {
        self.send_with(url, |request| request).await
    }

    /// [`Crossref::send`], with `customize` applied to the request first.
    async fn send_with(
        &self,
        url: &str,
        customize: impl Fn(reqwest::RequestBuilder) -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let mut attempt = 0u32;
        loop {
            self.limiter.acquire().await;
            tracing::debug!(%url, attempt, "crossref request");

            let response = customize(self.client.get(url)).send().await?;
            self.limiter.observe(response.headers());

            if response.status() != StatusCode::TOO_MANY_REQUESTS {
                return Ok(response);
            }

            let limit = self.limiter.rate();
            let delay = retry_after(response.headers())
                .unwrap_or_else(|| limit.interval * 2u32.pow(attempt.min(5)));
            self.limiter.back_off(delay);
            attempt += 1;

            if attempt > self.max_retries {
                tracing::warn!(%url, attempt, "crossref rate limited, retries exhausted");
                return Err(Error::RateLimited {
                    attempts: attempt,
                    limit,
                });
            }
            tracing::debug!(%url, attempt, ?delay, "crossref rate limited, retrying");
        }
    }

    /// Turns a non-success response into the error crossref described.
    ///
    /// Crossref answers a bad filter, sort field or field query with a `400`
    /// and a `validation-failure` body naming what it did not recognise, and an
    /// unservable content-negotiation format with a `406` and a bare
    /// `{code, message}` -- both of which
    /// [`reqwest::Response::error_for_status`] would throw away.
    async fn into_success(response: reqwest::Response) -> Result<reqwest::Response> {
        if response.status().is_success() {
            return Ok(response);
        }

        // taken before the body is consumed, so the status is still reportable
        // if the body turns out not to explain itself
        let status = response.error_for_status_ref().expect_err("not a success");
        let body = response.text().await.unwrap_or_default();

        if let Ok(Response {
            message: Message::ValidationFailure(failures),
            ..
        }) = serde_json::from_str::<Response>(&body)
        {
            return Err(Error::ValidationFailure { failures });
        }
        if let Ok(failure) = serde_json::from_str::<TransformFailure>(&body) {
            return Err(Error::ValidationFailure {
                failures: vec![Failure {
                    type_: failure.code,
                    value: failure.doi.unwrap_or_default(),
                    message: failure.message,
                }]
                .into(),
            });
        }
        Err(status.into())
    }

    //fn get_response_blocking<T: CrossrefQuery>(&self, query: &T) -> Result<Response> {
    //    let resp = self
    //        .blocking_client
    //        .get(&query.to_url(&self.base_url)?)
    //        .send()?
    //        .text()?;
    //    if resp.starts_with("Resource not found") {
    //        Err(Error::ResourceNotFound {
    //            resource: Box::new(query.clone().resource_component()),
    //        }
    //        .into())
    //    } else {
    //        Ok(serde_json::from_str(&resp)?)
    //    }
    //}

    /// Return the `Work` items that match a certain query.
    ///
    /// To search only by query terms use the convenience query method [`Crossref::works`]
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::{Crossref, WorksQuery, WorksFilter, FieldQuery};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// let client = Crossref::builder().build()?;
    ///
    /// let query = WorksQuery::new("Machine Learning")
    ///     .filter(WorksFilter::HasOrcid)
    ///     .order(crossref_client::Order::Asc)
    ///     .field_query(FieldQuery::author("Some Author"))
    ///     .sort(crossref_client::Sort::Score);
    ///
    /// let works = client.works(query).await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// This method fails if the `works` element expands to a bad route `ResourceNotFound`
    /// Fails if the response body doesn't have `message` field `MissingMessage`.
    /// Fails if anything else than a `WorkList` is returned as message `UnexpectedItem`
    pub async fn works<T: Into<WorkListQuery>>(&self, query: T) -> Result<WorkList> {
        let resp = self.get_response(&query.into()).await?;

        get_item!(WorkList, resp)
    }

    /// Return the `Work` that is identified by  the `doi`.
    ///
    /// # Errors
    /// This method fails if the doi could not identified `ResourceNotFound`
    ///
    pub async fn work(&self, doi: &str) -> Result<Work> {
        let resp = self
            .get_response(&Works::Identifier(doi.to_string()))
            .await?;
        get_item!(Work, resp).map(|x| *x)
    }

    /// [Deep paging results](https://github.com/CrossRef/rest-api-doc#deep-paging-with-cursors)
    /// Deep paging is supported for all queries, that return a list of `Work`, `WorkList`.
    /// This function returns a new iterator over pages of `Work`, which is returned as bulk of items as a `WorkList` by crossref.
    /// Usually a single page `WorkList` contains 20 items.
    ///
    /// # Example
    ///
    /// Iterate over all `Works` linked to search term `Machine Learning`
    ///
    /// ```no_run
    /// use crossref_client::{AsyncIterator, Crossref, WorksQuery, Work};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// let client = Crossref::builder().build()?;
    ///
    /// let mut pages = client.deep_page(WorksQuery::new("Machine Learning"));
    /// let mut all_works: Vec<Work> = Vec::new();
    /// while let Some(page) = pages.next().await {
    ///     let page = page?;
    ///     all_works.extend(page.items);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example
    ///
    /// Iterate over all the pages (`WorkList`) of the funder with id `funder id` by using a combined query.
    /// A single `WorkList` usually holds 20 `Work` items.
    ///
    /// ```no_run
    /// use crossref_client::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// let client = Crossref::builder().build()?;
    ///
    /// let mut pages = client.deep_page(WorksQuery::default().into_combined_query::<Funders>("funder id"));
    /// let mut all_funder_work_list: Vec<WorkList> = Vec::new();
    /// while let Some(page) = pages.next().await {
    ///     let page = page?;
    ///     all_funder_work_list.push(page);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// # Example
    ///
    /// Iterate over all `Work` items of a specfic funder directly.
    ///
    /// ```no_run
    /// use crossref_client::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// let client = Crossref::builder().build()?;
    ///
    /// let mut works = client.deep_page(WorksQuery::default()
    ///         .into_combined_query::<Funders>("funder id"))
    ///         .into_work_iter();
    /// let mut all_works: Vec<Work> = Vec::new();
    /// while let Some(work) = works.next().await {
    ///     let work = work?;
    ///     all_works.push(work);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example
    ///
    /// Alternatively deep page without an iterator by handling the cursor directly
    ///
    /// ```no_run
    /// use crossref_client::{Crossref, WorksQuery, WorksFilter};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// let client = Crossref::builder().build()?;
    ///
    /// // request a next-cursor first
    /// let query = WorksQuery::new("Machine Learning")
    ///     .new_cursor();
    ///
    /// let works = client.works(query.clone()).await?;
    ///
    /// // this continues from where this first response stopped
    /// // if no more work items are available then a empty list will be returned
    /// let deep_works = client.works(
    ///     query.next_cursor(&works.next_cursor.unwrap())
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn deep_page<T: Into<WorkListQuery>>(&self, query: T) -> WorkListIterator<'_> {
        WorkListIterator::new(query.into(), self)
    }

    /// Re-serialize the work identified by `doi` into another format.
    ///
    /// Crossref renders the registered metadata itself, so this never goes
    /// through [`Work`] and the body is returned verbatim -- BibTeX, RIS, RDF
    /// or a citation formatted in a [CSL style](Crossref::styles).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::{CnFormat, Crossref};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// # let client = Crossref::builder().build()?;
    /// let bibtex = client
    ///     .transform("10.1037/0003-066X.59.1.29", &CnFormat::BibTex)
    ///     .await?;
    ///
    /// let citation = client
    ///     .transform("10.1037/0003-066X.59.1.29", &CnFormat::bibliography("apa"))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Fails with [`Error::ValidationFailure`] if crossref cannot serve the
    /// format for this DOI -- an unknown CSL style is the usual cause.
    pub async fn transform(&self, doi: &str, format: &CnFormat) -> Result<String> {
        let url = format!("{}/works/{}/transform", self.base_url, doi);
        let response = self
            .send_with(&url, |request| {
                request.header(reqwest::header::ACCEPT, format.accept().as_ref())
            })
            .await?;

        Ok(Self::into_success(response).await?.text().await?)
    }

    /// Return the [CSL styles](https://citationstyles.org) crossref can render
    /// a citation in, for [`CnFormat::Bibliography`].
    ///
    /// Roughly 2 900 of them, in one response.
    pub async fn styles(&self) -> Result<StyleList> {
        let response = self.send(&format!("{}/styles", self.base_url)).await?;
        let body = Self::into_success(response).await?.bytes().await?;
        let response: Response =
            serde_json::from_slice(&body).map_err(|error| Error::Serde { error })?;

        get_item!(StyleList, response)
    }

    /// Return the `Agency` that registers the `Work` identified by  the `doi`.
    ///
    /// # Errors
    /// This method fails if the doi could not identified `ResourceNotFound`
    ///
    pub async fn work_agency(&self, doi: &str) -> Result<WorkAgency> {
        let resp = self.get_response(&Works::Agency(doi.to_string())).await?;
        get_item!(WorkAgency, resp)
    }

    /// Return the matching `Funders` items.
    pub async fn funders(&self, funders: FundersQuery) -> Result<FunderList> {
        let resp = self.get_response(&Funders::Query(funders)).await?;
        get_item!(FunderList, resp)
    }

    /// Return the `Funder` for the `id`
    pub async fn funder(&self, id: &str) -> Result<Funder> {
        let resp = self
            .get_response(&Funders::Identifier(id.to_string()))
            .await?;
        get_item!(Funder, resp).map(|x| *x)
    }

    /// Return the matching `Members` items.
    pub async fn members(&self, members: MembersQuery) -> Result<MemberList> {
        let resp = self.get_response(&Members::Query(members)).await?;
        get_item!(MemberList, resp)
    }

    /// Return the `Member` for the `id`
    pub async fn member(&self, member_id: &str) -> Result<Member> {
        let resp = self
            .get_response(&Members::Identifier(member_id.to_string()))
            .await?;
        get_item!(Member, resp).map(|x| *x)
    }

    /// Return the `Prefix` for the `id`
    pub async fn prefix(&self, id: &str) -> Result<Prefix> {
        let resp = self
            .get_response(&Prefixes::Identifier(id.to_string()))
            .await?;
        get_item!(Prefix, resp)
    }
    /// Return a specific `Journal`
    pub async fn journal(&self, id: &str) -> Result<Journal> {
        let resp = self
            .get_response(&Journals::Identifier(id.to_string()))
            .await?;

        get_item!(Journal, resp).map(|x| *x)
    }

    /// Return the matching `Journal` items.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::{Crossref, JournalsQuery, ResultControl};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// # let client = Crossref::builder().build()?;
    /// let journals = client
    ///     .journals(JournalsQuery::new("Economic Geography").result_control(ResultControl::Rows(10)))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn journals(&self, journals: JournalsQuery) -> Result<JournalList> {
        let resp = self.get_response(&Journals::Query(journals)).await?;
        get_item!(JournalList, resp)
    }

    /// Return the licenses works in the crossref metadata are published under,
    /// each with the number of works that carry it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::{Crossref, LicensesQuery, ResultControl};
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// # let client = Crossref::builder().build()?;
    /// let licenses = client
    ///     .licenses(LicensesQuery::new("creative commons").result_control(ResultControl::Rows(10)))
    ///     .await?;
    ///
    /// for license in &licenses.items {
    ///     println!("{:>8} {}", license.work_count, license.url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn licenses(&self, licenses: LicensesQuery) -> Result<LicenseList> {
        let resp = self.get_response(&Licenses::Query(licenses)).await?;
        get_item!(LicenseList, resp)
    }

    /// Return all available `Type`
    pub async fn types(&self) -> Result<TypeList> {
        let resp = self.get_response(&Types::All).await?;
        get_item!(TypeList, resp)
    }

    /// Return the `Type` for the `id`
    pub async fn type_(&self, id: &Type) -> Result<CrossrefType> {
        let resp = self
            .get_response(&Types::Identifier(id.id().to_string()))
            .await?;
        get_item!(Type, resp)
    }

    /// Get a random set of DOIs
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::Crossref;
    /// # async fn run() -> Result<(), crossref_client::Error> {
    /// # let client = Crossref::builder().build()?;
    /// // this will return 10 random dois from the crossref api
    /// let random_dois = client.random_dois(10).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn random_dois(&self, len: usize) -> Result<Vec<String>> {
        self.works(WorksQuery::random(len))
            .await
            .map(|x| x.items.into_iter().map(|x| x.doi).collect())
    }
}

/// A `CrossrefBuilder` can be used to create `Crossref` with additional config.
///
/// # Example
///
/// ```no_run
/// use crossref_client::Crossref;
/// # async fn run() -> Result<(), crossref_client::Error> {
///
/// let client = Crossref::builder()
///     .polite("polite@example.com")
///     .token("your token")
///     .build()?;
/// # Ok(())
/// # }
/// ```
/// Every setter accepts `Option`, so options that are themselves optional --
/// a CLI flag, a config field -- can be passed straight through:
///
/// ```no_run
/// # use crossref_client::Crossref;
/// # fn run(email: Option<String>) -> Result<(), crossref_client::Error> {
/// let client = Crossref::builder().polite(email.as_deref()).build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct CrossrefBuilder {
    /// [Good manners = more reliable service.](https://github.com/CrossRef/rest-api-doc#good-manners--more-reliable-service)
    ///
    /// will add a `User-Agent` header by default with with the `email` email.
    /// crossref can contact you if your script misbehaves
    /// this will get you directed to the "polite pool"
    user_agent: Option<String>,
    /// the token for the Crossref Plus service will be included as `Authorization` header
    /// This token will ensure that said requests get directed to a pool of machines that are reserved for "Plus" SLA users.
    plus_token: Option<String>,
    /// use a different base url than `Crossref::BASE_URL` <https://api.crossref.org>
    base_url: Option<String>,
    /// the budget to assume until crossref reports one
    rate_limit: Option<RateLimit>,
    /// how many times a `429` is retried
    max_retries: Option<u32>,
}

impl CrossrefBuilder {
    /// How many times a `429` is retried before [`Error::RateLimited`] is
    /// returned, when the builder was not told otherwise.
    const DEFAULT_MAX_RETRIES: u32 = 3;

    /// Constructs a new `CrossrefBuilder`.
    ///
    /// This is the same as `Crossref::builder()`.
    pub fn new() -> CrossrefBuilder {
        CrossrefBuilder::default()
    }

    /// Be polite: identify yourself by email and get routed to crossref's
    /// [polite pool](https://api.crossref.org/swagger-ui/index.html#/Etiquette).
    ///
    /// Sends a `User-Agent` in the shape crossref asks for, e.g.
    /// `crossref-client/0.2.0 (https://github.com/…; mailto:you@example.com)`.
    /// Anonymous requests share a rate-limited pool and are the usual cause of
    /// `429` responses; check where you landed with
    /// [`Crossref::api_pool`].
    ///
    /// Use [`CrossrefBuilder::user_agent`] instead if you want to send your own
    /// application's name; include `mailto:<email>` in it to stay in the pool.
    pub fn polite<'a>(mut self, email: impl Into<Option<&'a str>>) -> Self {
        if let Some(email) = email.into() {
            self.user_agent = Some(format!(
                "{}/{} ({}; mailto:{})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("CARGO_PKG_REPOSITORY"),
                email
            ));
        }
        self
    }

    /// set the user agent directly
    pub fn user_agent<'a>(mut self, user_agent: impl Into<Option<&'a str>>) -> Self {
        self.user_agent = user_agent.into().map(str::to_owned);
        self
    }

    /// set a crossref plus service  API token
    pub fn token<'a>(mut self, token: impl Into<Option<&'a str>>) -> Self {
        self.plus_token = token.into().map(str::to_owned);
        self
    }

    /// Send requests somewhere other than `https://api.crossref.org`.
    ///
    /// Routes are appended verbatim, so the url must not end in a `/`. Mainly
    /// useful for pointing the client at a mock or a proxy in tests.
    pub fn base_url<'a>(mut self, base_url: impl Into<Option<&'a str>>) -> Self {
        self.base_url = base_url.into().map(str::to_owned);
        self
    }

    /// The request budget to assume until crossref reports its own.
    ///
    /// Defaults to [`RateLimit::CONSERVATIVE`]. Crossref sends the real budget
    /// on every response and the client follows it from then on, so this only
    /// paces the first few requests.
    pub fn rate_limit(mut self, rate_limit: impl Into<Option<RateLimit>>) -> Self {
        self.rate_limit = rate_limit.into();
        self
    }

    /// How many times a `429` is retried before giving up with
    /// [`Error::RateLimited`]. Defaults to 3; `0` disables retrying.
    pub fn max_retries(mut self, max_retries: impl Into<Option<u32>>) -> Self {
        self.max_retries = max_retries.into();
        self
    }

    /// Returns a `Crossref` that uses this `CrossrefBuilder` configuration.
    /// # Errors
    ///
    /// This will fail if TLS backend cannot be initialized see [reqwest::ClientBuilder::build]
    pub fn build(self) -> Result<Crossref> {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        if let Some(agent) = &self.user_agent {
            headers.insert(
                header::USER_AGENT,
                header::HeaderValue::from_str(agent).map_err(|_| Error::Config {
                    msg: format!("failed to create User Agent header for `{}`", agent),
                })?,
            );
        }
        if let Some(token) = &self.plus_token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(token).map_err(|_| Error::Config {
                    msg: format!("failed to create AUTHORIZATION header for `{}`", token),
                })?,
            );
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|_| Error::Config {
                msg: "failed to initialize TLS backend".to_string(),
            })?;

        Ok(Crossref {
            base_url: self
                .base_url
                .unwrap_or_else(|| Crossref::BASE_URL.to_string()),
            client,
            limiter: Arc::new(Limiter::new(self.rate_limit.unwrap_or_default())),
            max_retries: self.max_retries.unwrap_or(Self::DEFAULT_MAX_RETRIES),
        })
    }
}

/// Allows iterating of deep page work request
pub struct WorkListIterator<'a> {
    /// the query
    query: WorkListQuery,
    /// performs each request
    client: &'a Crossref,
    /// whether the iterator should finish next iteration
    finish_next_iteration: bool,
}

impl<'a> WorkListIterator<'a> {
    /// Create an iterator that deep pages `query` through `client`.
    pub fn new(query: WorkListQuery, client: &'a Crossref) -> Self {
        Self {
            query,
            client,
            finish_next_iteration: false,
        }
    }

    /// Flatten the paged results into an iterator over individual [`Work`] items.
    ///
    /// Pages are fetched lazily, one request at a time, as the buffer drains.
    pub fn into_work_iter(self) -> WorkIterator<'a> {
        WorkIterator {
            pages: self,
            buffer: Vec::new().into_iter(),
        }
    }
}

/// Yields individual [`Work`] items across the deep paged [`WorkList`] pages.
///
/// Created by [`WorkListIterator::into_work_iter`].
pub struct WorkIterator<'a> {
    pages: WorkListIterator<'a>,
    buffer: std::vec::IntoIter<Work>,
}

impl async_iterator::Iterator for WorkIterator<'_> {
    type Item = Result<Work>;

    async fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(work) = self.buffer.next() {
                return Some(Ok(work));
            }
            // `WorkListIterator` stops on the first empty page and after an
            // error, so this terminates
            match self.pages.next().await? {
                Ok(page) => self.buffer = page.items.into_iter(),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

impl async_iterator::Iterator for WorkListIterator<'_> {
    type Item = Result<WorkList>;

    async fn next(&mut self) -> Option<Self::Item> {
        if self.finish_next_iteration {
            return None;
        }

        {
            let control = &mut self.query.query_mut().result_control;

            // if no result control is set, set a new cursor
            if control.is_none() {
                *control = Some(WorkResultControl::new_cursor());
            }
        }

        let page = match self.client.get_response(&self.query).await {
            Ok(resp) => get_item!(WorkList, resp),
            Err(err) => Err(err),
        };

        let worklist = match page {
            Ok(worklist) => worklist,
            Err(err) => {
                // a transient `429` or an unparsable page used to end the
                // iteration silently, which a caller cannot tell apart from
                // having crawled everything -- surface it and stop
                self.finish_next_iteration = true;
                return Some(Err(err));
            }
        };

        if let Some(cursor) = &worklist.next_cursor {
            match &mut self.query.query_mut().result_control {
                Some(WorkResultControl::Cursor { token, .. }) => {
                    // use the received cursor token in next iteration
                    *token = Some(cursor.clone())
                }
                Some(WorkResultControl::Standard(_)) => {
                    // standard result control was set, don't deep page and return next iteration
                    self.finish_next_iteration = true;
                }
                _ => (),
            }
        } else {
            // no cursor received, end next iteration
            self.finish_next_iteration = true;
        }

        if worklist.items.is_empty() {
            None
        } else {
            Some(Ok(worklist))
        }
    }
}
