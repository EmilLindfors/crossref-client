use crate::error::Result;
pub use crate::query::funders::{Funders, FundersQuery};
pub use crate::query::journals::{Journals, JournalsQuery};
pub use crate::query::licenses::{Licenses, LicensesQuery};
pub use crate::query::members::{Members, MembersQuery};
pub use crate::query::prefixes::Prefixes;
pub use crate::query::types::{Type, Types};
use crate::query::works::Works;
pub use crate::query::works::{WorksIdentQuery, WorksQuery};
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;
use std::str::FromStr;

/// How a filter's payload renders inside a `filter` fragment.
///
/// Implemented for the handful of types a filter can carry, so
/// [`define_filter!`] can render every variant without a per-type match arm --
/// the catch-all arm that used to do that job silently turned
/// `WorksFilter::AlternativeId(id)` into `alternative-id:true`.
pub(crate) trait FilterValue {
    /// the value as it appears after the `:`
    fn render(&self) -> Cow<'_, str>;
}

impl FilterValue for String {
    fn render(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
}

impl FilterValue for i32 {
    fn render(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string())
    }
}

impl FilterValue for u64 {
    fn render(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string())
    }
}

impl FilterValue for chrono::NaiveDate {
    fn render(&self) -> Cow<'_, str> {
        Cow::Owned(self.format("%Y-%m-%d").to_string())
    }
}

impl FilterValue for types::Type {
    fn render(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.id())
    }
}

/// Defines a route's filter enum together with everything that has to stay in
/// step with it: the key each variant renders under, how its payload renders,
/// and -- for the coverage tests -- one value of every variant.
///
/// Marker filters carry no payload and render as `key:true`; value filters
/// carry one, whose type implements [`FilterValue`].
macro_rules! define_filter {
    (
        $(#[$meta:meta])*
        $name:ident;
        markers { $($(#[$m_doc:meta])* $m_variant:ident => $m_key:literal,)* }
        values  { $($(#[$v_doc:meta])* $v_variant:ident($v_ty:ty) => $v_key:literal,)* }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $($(#[$m_doc])* $m_variant,)*
            $($(#[$v_doc])* $v_variant($v_ty),)*
        }

        impl $name {
            /// Every filter name this route accepts, in declaration order.
            ///
            /// Pinned against the list crossref reports in its `400` body by
            /// the coverage tests, so it is the vocabulary of the live api
            /// rather than of this crate's enum.
            pub const ALL_NAMES: &'static [&'static str] = &[$($m_key,)* $($v_key,)*];

            /// the key this filter renders under in the query string
            pub fn name(&self) -> &'static str {
                match self {
                    $($name::$m_variant => $m_key,)*
                    $($name::$v_variant(_) => $v_key,)*
                }
            }
        }

        impl $crate::query::ParamFragment for $name {
            fn key(&self) -> ::std::borrow::Cow<'_, str> {
                ::std::borrow::Cow::Borrowed(self.name())
            }

            fn value(&self) -> Option<::std::borrow::Cow<'_, str>> {
                match self {
                    $($name::$m_variant => Some(::std::borrow::Cow::Borrowed("true")),)*
                    $($name::$v_variant(value) => {
                        Some($crate::query::FilterValue::render(value))
                    })*
                }
            }
        }

        impl $crate::query::Filter for $name {}
    };
}

/// Defines a query for a list route that takes free form terms, paging, and
/// optionally a filter -- and nothing else.
///
/// `/works` is the only route with `sort`, `order`, `facet`, `select` and
/// `sample`; `/funders`, `/members`, `/journals` and `/licenses` all answer
/// those with a `400`, and only the first two take a `filter` on top of terms
/// and paging.
macro_rules! impl_list_query {
    ($(#[$meta:meta])* $name:ident $(, filter = $filter:ident)?) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $name {
            /// search by non specific query
            pub queries: Vec<String>,
            $(
                /// filter to apply while querying
                pub filter: Vec<$filter>,
            )?
            /// limit the returned items and set an offset
            ///
            /// Only `rows` and `offset` are supported by this route;
            /// [`ResultControl::Sample`] is rejected by crossref with a `400`.
            pub result_control: Option<ResultControl>,
        }

        impl $name {
            $(
                /// add a new filter to the query
                pub fn filter(mut self, filter: $filter) -> Self {
                    self.filter.push(filter);
                    self
                }
            )?
            /// alias for creating an empty default element
            pub fn empty() -> Self {
                Self::default()
            }

            /// Convenience method to create a new query with a term directly
            pub fn new<T: ToString>(query: T) -> Self {
                Self::empty().query(query)
            }

            /// add a new free form query
            pub fn query<T: ToString>(mut self, query: T) -> Self {
                self.queries.push(query.to_string());
                self
            }

            /// add a bunch of free form query terms
            pub fn queries<T: ToString>(mut self, queries: &[T]) -> Self {
                self.queries.extend(queries.iter().map(T::to_string));
                self
            }

            /// Limit the results, or take crossref's default page if given [`None`].
            pub fn result_control(
                mut self,
                result_control: impl Into<Option<ResultControl>>,
            ) -> Self {
                self.result_control = result_control.into();
                self
            }
        }

        impl CrossrefRoute for $name {
            fn route(&self) -> Result<String> {
                let mut params: Vec<(Cow<'_, str>, Cow<'_, str>)> = Vec::new();
                if !self.queries.is_empty() {
                    params.push((
                        Cow::Borrowed("query"),
                        Cow::Owned(format_queries(&self.queries)),
                    ));
                }
                $(
                    let filters: &[$filter] = &self.filter;
                    if !filters.is_empty() {
                        $crate::query::reject_unsendable_filters(filters)?;
                        params.extend(self.filter.params());
                    }
                )?
                if let Some(rc) = &self.result_control {
                    params.extend(rc.params());
                }
                Ok(encode::query_string(&params))
            }
        }
    };
}

/// Defines an enum of crossref identifiers together with the list of them, so
/// a coverage test can check the list against what the api reports.
macro_rules! define_keyed_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $($(#[$doc:meta])* $variant:ident => $key:literal,)* }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                #[doc = concat!("the `", $key, "` field")]
                $(#[$doc])*
                $variant,
            )*
        }

        impl $name {
            /// the identifier crossref knows this by
            pub fn name(&self) -> &'static str {
                match self { $($name::$variant => $key,)* }
            }

            /// Every variant, in declaration order.
            pub const ALL: &'static [$name] = &[$($name::$variant,)*];
        }
    };
}

/// Defines [`FieldQuery`](works::FieldQuery), the queries that match against
/// one field of a work's metadata rather than all of it.
///
/// Each entry names the variant, the field crossref knows it by, and the
/// lower-case constructor that saves callers an `into`.
macro_rules! define_field_queries {
    ($($(#[$doc:meta])* $variant:ident => $key:literal / $ctor:ident,)*) => {
        /// Matches against one field of a work's metadata rather than all of
        /// it. Available on the `/works` route only.
        ///
        /// ```
        /// # use crossref_client::FieldQuery;
        /// assert_eq!("query.author", FieldQuery::author("feynman").name());
        /// ```
        ///
        /// Covers all 21 field queries `/works` accepts, which
        /// `every_field_query_is_accepted_by_the_api` pins against what the api
        /// reports.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum FieldQuery {
            $(
                #[doc = concat!("matches against `", $key, "`: ")]
                $(#[$doc])*
                $variant(String),
            )*
        }

        impl FieldQuery {
            /// Every field `/works` can be queried against, in declaration
            /// order -- the `author` of `query.author`.
            pub const ALL_FIELDS: &'static [&'static str] = &[$($key,)*];

            /// the key this renders under in the query string, e.g. `query.author`
            pub fn name(&self) -> &'static str {
                match self { $(FieldQuery::$variant(_) => concat!("query.", $key),)* }
            }

            /// the field crossref knows this by, e.g. `author`
            pub fn field(&self) -> &'static str {
                match self { $(FieldQuery::$variant(_) => $key,)* }
            }

            /// the term being matched
            pub fn value(&self) -> &str {
                match self { $(FieldQuery::$variant(value) => value,)* }
            }

            $(
                #[doc = concat!("a query against `", $key, "`")]
                pub fn $ctor(value: impl Into<String>) -> Self {
                    FieldQuery::$variant(value.into())
                }
            )*
        }
    };
}

/// percent-encoding of the crossref query string
pub(crate) mod encode;
/// provides types to filter facets
pub mod facet;
/// provides support to query the `/funders` route
pub mod funders;
/// provides support to query the `/funders` route
pub mod journals;
/// provides support to query the `/licenses` route
pub mod licenses;
/// provides support to query the `/members` route
pub mod members;
/// provides support to query the `/members` route
pub mod prefixes;
/// provides support to query the `/prefixes` route
pub mod types;
/// provides support to query the `/types` route
pub mod works;

/// represents the visibility of an crossref item
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    /// readable by anyone
    Open,
    /// readable by Metadata Plus subscribers
    Limited,
    /// not readable
    Closed,
}

impl Visibility {
    /// str identifier
    pub fn as_str(&self) -> &str {
        match self {
            Visibility::Open => "open",
            Visibility::Limited => "limited",
            Visibility::Closed => "closed",
        }
    }
}

/// Determines how results should be sorted
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Order {
    /// list results in ascending order
    Asc,
    /// list results in descending order
    Desc,
}

impl Order {
    /// the key name for the order parameter
    pub fn as_str(&self) -> &str {
        match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        }
    }
}

impl FromStr for Order {
    type Err = String;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Order::Asc),
            "desc" => Ok(Order::Desc),
            other => Err(format!("Unable to convert {} to Order", other)),
        }
    }
}

impl CrossrefQueryParam for Order {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        vec![(Cow::Borrowed("order"), Cow::Borrowed(self.as_str()))]
    }
}

/// Results from a list response can be sorted by applying the sort and order parameters.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Sort {
    /// Sort by relevance score
    Score,
    /// Sort by date of most recent change to metadata. Currently the same as `Deposited`
    Updated,
    /// Sort by time of most recent deposit
    Deposited,
    /// Sort by time of most recent index
    Indexed,
    /// Sort by publication date
    Published,
    /// Sort by print publication date
    PublishedPrint,
    /// Sort by online publication date
    PublishedOnline,
    /// Sort by issued date (earliest known publication date)
    Issued,
    /// Sort by number of times this DOI is referenced by other Crossref DOIs
    IsReferencedByCount,
    /// Sort by number of references included in the references section of the document identified by this DOI
    ReferenceCount,
    /// Sort by date the record was created
    Created,
    /// Sort by relevance to the query terms
    Relevance,
}

impl Sort {
    /// Every field `/works` can be sorted by, in declaration order.
    pub const ALL: &'static [Sort] = &[
        Sort::Score,
        Sort::Updated,
        Sort::Deposited,
        Sort::Indexed,
        Sort::Published,
        Sort::PublishedPrint,
        Sort::PublishedOnline,
        Sort::Issued,
        Sort::IsReferencedByCount,
        Sort::ReferenceCount,
        Sort::Created,
        Sort::Relevance,
    ];

    /// the key name for the filter element
    pub fn as_str(&self) -> &str {
        match self {
            Sort::Score => "score",
            Sort::Updated => "updated",
            Sort::Deposited => "deposited",
            Sort::Indexed => "indexed",
            Sort::Published => "published",
            Sort::PublishedPrint => "published-print",
            Sort::PublishedOnline => "published-online",
            Sort::Issued => "issued",
            Sort::IsReferencedByCount => "is-referenced-by-count",
            Sort::ReferenceCount => "references-count",
            Sort::Created => "created",
            Sort::Relevance => "relevance",
        }
    }
}

impl FromStr for Sort {
    type Err = String;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        match s {
            "score" => Ok(Sort::Score),
            "updated" => Ok(Sort::Updated),
            "deposited" => Ok(Sort::Deposited),
            "indexed" => Ok(Sort::Indexed),
            "published" => Ok(Sort::Published),
            "published-print" => Ok(Sort::PublishedPrint),
            "published-online" => Ok(Sort::PublishedOnline),
            "issued" => Ok(Sort::Issued),
            "is-referenced-by-count" => Ok(Sort::IsReferencedByCount),
            "references-count" => Ok(Sort::ReferenceCount),
            "created" => Ok(Sort::Created),
            "relevance" => Ok(Sort::Relevance),
            other => Err(format!("Unable to convert {} to Sort", other)),
        }
    }
}

impl CrossrefQueryParam for Sort {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        vec![(Cow::Borrowed("sort"), Cow::Borrowed(self.as_str()))]
    }
}

/// tells crossref how many items shall be returned or where to start
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultControl {
    /// limits the returned items per page
    Rows(usize),
    /// sets an offset where crossref begins to retrieve items
    /// high offsets (~10k) result in long response times
    Offset(usize),
    /// combines rows and offset: limit returned items per page, starting at the offset
    RowsOffset {
        /// row limit
        rows: usize,
        /// where to start
        offset: usize,
    },
    /// return random results
    Sample(usize),
}

impl CrossrefQueryParam for ResultControl {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        match self {
            ResultControl::Rows(rows) => {
                vec![(Cow::Borrowed("rows"), Cow::Owned(rows.to_string()))]
            }
            ResultControl::Offset(offset) => {
                vec![(Cow::Borrowed("offset"), Cow::Owned(offset.to_string()))]
            }
            ResultControl::RowsOffset { rows, offset } => vec![
                (Cow::Borrowed("rows"), Cow::Owned(rows.to_string())),
                (Cow::Borrowed("offset"), Cow::Owned(offset.to_string())),
            ],
            ResultControl::Sample(sample) => {
                vec![(Cow::Borrowed("sample"), Cow::Owned(sample.to_string()))]
            }
        }
    }
}

/// Major resource components supported by the Crossref API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Component {
    /// a list of all works (journal articles, conference proceedings, books, components, etc), 20 per page
    Works,
    /// a list of all funders in the [Funder Registry](https://github.com/Crossref/open-funder-registry)
    Funders,
    /// a list of DOI owner prefixes
    Prefixes,
    /// a list of all Crossref members (mostly publishers)
    Members,
    /// a list of valid work types
    Types,
    /// a list of journals in the Crossref database
    Journals,
    /// a list of the licenses works in the Crossref metadata are published under
    Licenses,
}

impl Component {
    /// identifier for the component route
    pub fn as_str(&self) -> &str {
        match self {
            Component::Works => "works",
            Component::Funders => "funders",
            Component::Prefixes => "prefixes",
            Component::Members => "members",
            Component::Types => "types",
            Component::Journals => "journals",
            Component::Licenses => "licenses",
        }
    }
}

impl CrossrefRoute for Component {
    fn route(&self) -> Result<String> {
        Ok(format!("/{}", self.as_str()))
    }
}

/// The components that also expose their works at `/{component}/{id}/works`.
///
/// A narrower [`Component`]: `/works` has no such sub-route of its own, and
/// neither does `/licenses`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorksComponent {
    /// works of one funder, at `/funders/{id}/works`
    Funders,
    /// works of one DOI prefix, at `/prefixes/{id}/works`
    Prefixes,
    /// works of one member, at `/members/{id}/works`
    Members,
    /// works of one type, at `/types/{id}/works`
    Types,
    /// works of one journal, at `/journals/{issn}/works`
    Journals,
}

impl WorksComponent {
    /// the route this component owns
    pub fn component(&self) -> Component {
        match self {
            WorksComponent::Funders => Component::Funders,
            WorksComponent::Prefixes => Component::Prefixes,
            WorksComponent::Members => Component::Members,
            WorksComponent::Types => Component::Types,
            WorksComponent::Journals => Component::Journals,
        }
    }
}

impl CrossrefRoute for WorksComponent {
    fn route(&self) -> Result<String> {
        self.component().route()
    }
}

/// bundles all available crossref api endpoints
#[derive(Debug, Clone)]
pub enum ResourceComponent {
    /// the `/works` route
    Works(Works),
    /// the `/funders` route
    Funders(Funders),
    /// the `/prefixes` route
    Prefixes(Prefixes),
    /// the `/members` route
    Members(Members),
    /// the `/types` route
    Types(Types),
    /// the `/journals` route
    Journals(Journals),
    /// the `/licenses` route
    Licenses(Licenses),
}

impl ResourceComponent {
    /// the starting crossref component that in the route `/{primary_component}/{id}/works`
    pub fn primary_component(&self) -> Component {
        match self {
            ResourceComponent::Works(_) => Component::Works,
            ResourceComponent::Funders(_) => Component::Funders,
            ResourceComponent::Prefixes(_) => Component::Prefixes,
            ResourceComponent::Members(_) => Component::Members,
            ResourceComponent::Types(_) => Component::Types,
            ResourceComponent::Journals(_) => Component::Journals,
            ResourceComponent::Licenses(_) => Component::Licenses,
        }
    }
}

impl fmt::Display for ResourceComponent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.route().map_err(|_| fmt::Error)?)
    }
}

impl CrossrefRoute for ResourceComponent {
    fn route(&self) -> Result<String> {
        match self {
            ResourceComponent::Works(c) => c.route(),
            ResourceComponent::Funders(c) => c.route(),
            ResourceComponent::Prefixes(c) => c.route(),
            ResourceComponent::Members(c) => c.route(),
            ResourceComponent::Types(c) => c.route(),
            ResourceComponent::Journals(c) => c.route(),
            ResourceComponent::Licenses(c) => c.route(),
        }
    }
}

impl CrossrefQuery for ResourceComponent {
    fn resource_component(self) -> ResourceComponent {
        self
    }
}

/// Helper trait to mark filters in the query string
pub trait Filter: ParamFragment {}

impl<T: Filter> CrossrefQueryParam for Vec<T> {
    /// filters share the `filter` key and are concat with `,`
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        vec![(
            Cow::Borrowed("filter"),
            Cow::Owned(
                self.iter()
                    .map(ParamFragment::fragment)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        )]
    }
}

/// represents a key value pair inside a multi value query string parameter
pub trait ParamFragment {
    /// the key, or name, of the fragment
    fn key(&self) -> Cow<'_, str>;

    /// the value of the fragment, if any
    fn value(&self) -> Option<Cow<'_, str>>;

    /// key and value are concat using `:`
    fn fragment(&self) -> Cow<'_, str> {
        if let Some(val) = self.value() {
            Cow::Owned(format!("{}:{}", self.key(), val))
        } else {
            self.key()
        }
    }
}

/// a trait used to capture parameters for the query string of the crossref api
pub trait CrossrefQueryParam {
    /// The key/value pairs this element contributes to the query string.
    ///
    /// Most elements contribute exactly one pair, but a few span two --
    /// [`ResultControl::RowsOffset`] and [`works::WorkResultControl::Cursor`]
    /// with a row limit. Yielding pairs rather than a rendered `key=value`
    /// fragment is what lets the route builder percent-encode keys and values
    /// separately.
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)>;
}

impl<T: AsRef<str>> CrossrefQueryParam for (T, T) {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        vec![(
            Cow::Borrowed(self.0.as_ref()),
            Cow::Borrowed(self.1.as_ref()),
        )]
    }
}

/// represents elements that constructs parts of the crossref request url
pub trait CrossrefRoute {
    /// constructs the route for the crossref api
    fn route(&self) -> Result<String>;
}

/// root level trait to construct full crossref api request urls
pub trait CrossrefQuery: CrossrefRoute + Clone {
    /// the resource component endpoint this route targets
    fn resource_component(self) -> ResourceComponent;

    /// constructs the full request url by concating the `base_path` with the `route`
    fn to_url(&self, base_path: &str) -> Result<String> {
        Ok(format!("{}{}", base_path, self.route()?))
    }
}

/// Refuses a filter value crossref would read as two filters.
///
/// Crossref splits the `filter` value on `,` *after* percent-decoding it, so a
/// value carrying one arrives as two fragments -- `container-title:A, B`
/// becomes the filter `container-title:A` plus a filter called ` B`, which the
/// api answers with a `400`. Neither the encoded nor the literal form survives,
/// so there is nothing to escape it with and the request is refused here
/// instead, where the error can say which filter is at fault.
pub(crate) fn reject_unsendable_filters<T: Filter>(filters: &[T]) -> Result<()> {
    for filter in filters {
        match filter.value() {
            Some(value) if value.contains(',') => {
                return Err(crate::Error::UnsendableFilterValue {
                    filter: filter.key().into_owned(),
                    value: value.into_owned(),
                });
            }
            _ => (),
        }
    }
    Ok(())
}

/// normalizes a query term by collapsing runs of whitespace into single spaces
///
/// Crossref reads the `query` value as a whitespace separated list of terms.
/// The separator is a plain space rather than the `+` it renders as on the wire,
/// because [`encode`] escapes the value afterwards and a literal `+` there would
/// come back out as one.
pub(crate) fn format_query<T: AsRef<str>>(topic: T) -> String {
    topic
        .as_ref()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// combines the individual topics of a query into a single `query` value
pub(crate) fn format_queries<T: AsRef<str>>(topics: &[T]) -> String {
    topics
        .iter()
        .flat_map(|topic| topic.as_ref().split_whitespace())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Pins this crate's idea of a route's vocabulary against crossref's.
///
/// Crossref answers an unrecognised filter, sort field, field query or `select`
/// element with a `400` that lists the ones it does know, so the lists below
/// are copied from those responses. Checking both directions catches a name
/// this crate would send and crossref would reject, *and* one crossref accepts
/// that this crate cannot express.
#[cfg(test)]
fn assert_matches_api(kind: &str, ours: &[&str], api: &[&str]) {
    use std::collections::BTreeSet;

    let mine: BTreeSet<&str> = ours.iter().copied().collect();
    assert_eq!(ours.len(), mine.len(), "duplicate {kind} in this crate");

    let theirs: BTreeSet<&str> = api.iter().copied().collect();
    assert_eq!(api.len(), theirs.len(), "duplicate {kind} in the api list");

    let rejected: Vec<_> = mine.difference(&theirs).collect();
    assert!(
        rejected.is_empty(),
        "{kind} this crate sends that crossref answers with a 400: {rejected:?}"
    );

    let unreachable: Vec<_> = theirs.difference(&mine).collect();
    assert!(
        unreachable.is_empty(),
        "{kind} crossref accepts that this crate cannot express: {unreachable:?}"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::funders::FundersFilter;
    use crate::query::members::MembersFilter;
    use crate::query::works::{FieldQuery, WorkElement, WorksFilter};

    /// The set crossref reports in its `400` response for an unknown sort field.
    const API_SORT_FIELDS: &[&str] = &[
        "score",
        "created",
        "issued",
        "indexed",
        "is-referenced-by-count",
        "relevance",
        "published",
        "published-print",
        "published-online",
        "updated",
        "references-count",
        "deposited",
    ];

    #[test]
    fn every_sort_key_is_accepted_by_the_api() {
        let names: Vec<&str> = Sort::ALL.iter().map(Sort::as_str).collect();

        assert_matches_api("/works sort fields", &names, API_SORT_FIELDS);
    }

    #[test]
    fn sort_round_trips_through_from_str() {
        for sort in Sort::ALL {
            assert_eq!(Ok(*sort), Sort::from_str(sort.as_str()));
        }
    }

    #[test]
    fn order_round_trips_through_from_str() {
        for order in [Order::Asc, Order::Desc] {
            assert_eq!(Ok(order), Order::from_str(order.as_str()));
        }
    }

    /// The filters crossref reports for `/works`.
    const API_WORKS_FILTERS: &[&str] = &[
        "alternative-id",
        "archive",
        "article-number",
        "assertion",
        "assertion-group",
        "award.funder",
        "award.number",
        "category-name",
        "clinical-trial-number",
        "container-title",
        "content-domain",
        "directory",
        "doi",
        "from-accepted-date",
        "from-approved-date",
        "from-awarded-date",
        "from-created-date",
        "from-deposit-date",
        "from-event-end-date",
        "from-event-start-date",
        "from-index-date",
        "from-issued-date",
        "from-online-pub-date",
        "from-posted-date",
        "from-print-pub-date",
        "from-pub-date",
        "from-update-date",
        "full-text.application",
        "full-text.type",
        "full-text.version",
        "funder",
        "funder-doi-asserted-by",
        "group-title",
        "gte-award-amount",
        "has-abstract",
        "has-affiliation",
        "has-affiliation-ror-id",
        "has-alias",
        "has-archive",
        "has-assertion",
        "has-authenticated-orcid",
        "has-award",
        "has-clinical-trial-number",
        "has-content-domain",
        "has-domain-restriction",
        "has-event",
        "has-full-text",
        "has-funder",
        "has-funder-doi",
        "has-funder-ror-id",
        "has-license",
        "has-orcid",
        "has-prime-doi",
        "has-references",
        "has-relation",
        "has-ror-id",
        "has-update",
        "has-update-policy",
        "is-update",
        "isbn",
        "issn",
        "license.delay",
        "license.url",
        "license.version",
        "lte-award-amount",
        "member",
        "orcid",
        "prefix",
        "relation.object",
        "relation.object-type",
        "relation.type",
        "ror-id",
        "type",
        "type-name",
        "until-accepted-date",
        "until-approved-date",
        "until-awarded-date",
        "until-created-date",
        "until-deposit-date",
        "until-event-end-date",
        "until-event-start-date",
        "until-index-date",
        "until-issued-date",
        "until-online-pub-date",
        "until-posted-date",
        "until-print-pub-date",
        "until-pub-date",
        "until-update-date",
        "update-type",
        "updates",
    ];

    /// The field queries crossref reports for `/works`.
    const API_FIELD_QUERIES: &[&str] = &[
        "affiliation",
        "author",
        "bibliographic",
        "chair",
        "container-title",
        "contributor",
        "degree",
        "description",
        "editor",
        "event-acronym",
        "event-location",
        "event-name",
        "event-sponsor",
        "event-theme",
        "funder-name",
        "publisher-location",
        "publisher-name",
        "standards-body-acronym",
        "standards-body-name",
        "title",
        "translator",
    ];

    /// The `select` elements crossref reports for `/works`.
    const API_WORK_ELEMENTS: &[&str] = &[
        "DOI",
        "ISBN",
        "ISSN",
        "URL",
        "abstract",
        "accepted",
        "alternative-id",
        "approved",
        "archive",
        "article-number",
        "assertion",
        "author",
        "chair",
        "clinical-trial-number",
        "container-title",
        "content-created",
        "content-domain",
        "contributor",
        "created",
        "degree",
        "deposited",
        "editor",
        "event",
        "funder",
        "group-title",
        "indexed",
        "is-referenced-by-count",
        "issn-type",
        "issue",
        "issued",
        "license",
        "link",
        "member",
        "original-title",
        "page",
        "posted",
        "prefix",
        "published",
        "published-online",
        "published-print",
        "publisher",
        "publisher-location",
        "reference",
        "references-count",
        "relation",
        "resource",
        "score",
        "short-container-title",
        "short-title",
        "standards-body",
        "subject",
        "subtitle",
        "title",
        "translator",
        "type",
        "update-policy",
        "update-to",
        "updated-by",
        "volume",
    ];

    /// The filters crossref reports for `/funders`.
    const API_FUNDERS_FILTERS: &[&str] = &["location"];

    /// The filters crossref reports for `/members`.
    const API_MEMBERS_FILTERS: &[&str] = &["prefix", "backfile-doi-count", "current-doi-count"];

    #[test]
    fn every_works_filter_is_accepted_by_the_api() {
        assert_matches_api("/works filters", WorksFilter::ALL_NAMES, API_WORKS_FILTERS);
    }

    #[test]
    fn every_field_query_is_accepted_by_the_api() {
        assert_matches_api(
            "/works field queries",
            FieldQuery::ALL_FIELDS,
            API_FIELD_QUERIES,
        );
    }

    #[test]
    fn every_selectable_element_is_accepted_by_the_api() {
        let names: Vec<&str> = WorkElement::ALL.iter().map(WorkElement::name).collect();

        assert_matches_api("/works select elements", &names, API_WORK_ELEMENTS);
    }

    #[test]
    fn every_funders_filter_is_accepted_by_the_api() {
        assert_matches_api(
            "/funders filters",
            FundersFilter::ALL_NAMES,
            API_FUNDERS_FILTERS,
        );
    }

    #[test]
    fn every_members_filter_is_accepted_by_the_api() {
        assert_matches_api(
            "/members filters",
            MembersFilter::ALL_NAMES,
            API_MEMBERS_FILTERS,
        );
    }

    #[test]
    fn a_marker_filter_renders_as_true_and_a_value_filter_as_its_value() {
        let filters = vec![
            WorksFilter::HasOrcid,
            WorksFilter::AlternativeId("abc".to_string()),
            WorksFilter::LicenseDelay(30),
            WorksFilter::GteAwardAmount(1_000),
        ];

        assert_eq!(
            vec![(
                Cow::Borrowed("filter"),
                Cow::Owned(
                    "has-orcid:true,alternative-id:abc,license.delay:30,gte-award-amount:1000"
                        .to_string()
                )
            )],
            filters.params()
        );
    }
}
