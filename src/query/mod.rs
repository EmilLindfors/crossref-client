use crate::error::Result;
use crate::query::facet::FacetCount;
pub use crate::query::funders::{Funders, FundersQuery};
pub use crate::query::journals::{Journals, JournalsQuery};
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

/// Helper trait for unified interface
pub trait CrossrefParams {
    /// the filter applied
    type Filter: Filter;
    /// all string queries
    fn query_terms(&self) -> &[String];
    /// the filters this object can use
    fn filters(&self) -> &[Self::Filter];
    /// the sort if set
    fn sort(&self) -> Option<&Sort>;
    /// the order if set
    fn order(&self) -> Option<&Order>;
    /// all facets this objects addresses
    fn facets(&self) -> &[FacetCount];
    /// the configured result control, if any
    fn result_control(&self) -> Option<&ResultControl>;
}

macro_rules! impl_common_query {
    ($i:ident, $filter:ident) => {
        /// Each query parameter is ANDed
        #[derive(Debug, Clone, Default)]
        pub struct $i {
            /// search by non specific query
            pub queries: Vec<String>,
            /// filter to apply while querying
            pub filter: Vec<$filter>,
            /// sort results by a certain field and
            pub sort: Option<Sort>,
            /// set the sort order to `asc` or `desc`
            pub order: Option<Order>,
            /// enable facet information in responses
            pub facets: Vec<FacetCount>,
            /// deep page through `/works` result sets
            pub result_control: Option<ResultControl>,
        }

        impl $i {
            /// alias for creating an empty default element
            pub fn empty() -> Self {
                $i::default()
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

            /// add a new filter to the query
            pub fn filter(mut self, filter: $filter) -> Self {
                self.filter.push(filter);
                self
            }

            /// Sort the results by a field, or by relevance if given [`None`].
            pub fn sort(mut self, sort: impl Into<Option<Sort>>) -> Self {
                self.sort = sort.into();
                self
            }

            /// set order to asc
            pub fn order_asc(mut self) -> Self {
                self.order = Some(Order::Asc);
                self
            }
            /// set order to desc
            pub fn order_desc(mut self) -> Self {
                self.order = Some(Order::Desc);
                self
            }

            /// Order the results, or leave the order to crossref if given [`None`].
            pub fn order(mut self, order: impl Into<Option<Order>>) -> Self {
                self.order = order.into();
                self
            }

            /// add another facet to query
            pub fn facet(mut self, facet: FacetCount) -> Self {
                self.facets.push(facet);
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

        impl CrossrefParams for $i {
            type Filter = $filter;

            fn query_terms(&self) -> &[String] {
                &self.queries
            }
            fn filters(&self) -> &[Self::Filter] {
                &self.filter
            }
            fn sort(&self) -> Option<&Sort> {
                self.sort.as_ref()
            }
            fn order(&self) -> Option<&Order> {
                self.order.as_ref()
            }
            fn facets(&self) -> &[FacetCount] {
                &self.facets
            }
            fn result_control(&self) -> Option<&ResultControl> {
                self.result_control.as_ref()
            }
        }

        impl CrossrefRoute for $i {
            fn route(&self) -> Result<String> {
                let mut params: Vec<(Cow<'_, str>, Cow<'_, str>)> = Vec::new();
                if !self.queries.is_empty() {
                    params.push((
                        Cow::Borrowed("query"),
                        Cow::Owned(format_queries(&self.queries)),
                    ));
                }
                if !self.filter.is_empty() {
                    params.extend(self.filter.params());
                }
                if !self.facets.is_empty() {
                    params.extend(self.facets.params());
                }
                if let Some(sort) = &self.sort {
                    params.extend(sort.params());
                }
                if let Some(order) = &self.order {
                    params.extend(order.params());
                }
                if let Some(rc) = &self.result_control {
                    params.extend(rc.params());
                }
                Ok(encode::query_string(&params))
            }
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
/// provides support to query the `/journals` route
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
#[allow(missing_docs)]
pub enum Visibility {
    Open,
    Limited,
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
            Sort::Relevance => "relevance"
            
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
#[derive(Debug, Clone)]
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
            ResultControl::Rows(rows) => vec![(Cow::Borrowed("rows"), Cow::Owned(rows.to_string()))],
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Component {
    /// returns a list of all works (journal articles, conference proceedings, books, components, etc), 20 per page
    Works,
    /// returns a list of all funders in the [Funder Registry](https://github.com/Crossref/open-funder-registry)
    Funders,
    /// returns a list of all Crossref members (mostly publishers)
    Prefixes,
    /// returns a list of valid work types
    Members,
    /// return a list of licenses applied to works in Crossref metadata
    Types,
    /// return a list of journals in the Crossref database
    Journals,
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
        }
    }
}

impl CrossrefRoute for Component {
    fn route(&self) -> Result<String> {
        Ok(format!("/{}", self.as_str()))
    }
}

/// bundles all available crossref api endpoints
#[derive(Debug, Clone)]
pub enum ResourceComponent {
    /// returns a list of all works (journal articles, conference proceedings, books, components, etc), 20 per page
    Works(Works),
    /// returns a list of all funders in the [Funder Registry](https://github.com/Crossref/open-funder-registry)
    Funders(Funders),
    /// returns a list of all Crossref members (mostly publishers)
    Prefixes(Prefixes),
    /// returns a list of valid work types
    Members(Members),
    /// return a list of licenses applied to works in Crossref metadata
    Types(Types),
    /// return a list of journals in the Crossref database
    Journals(Journals),
    
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

#[cfg(test)]
mod tests {
    use super::*;

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

    const ALL_SORTS: &[Sort] = &[
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

    #[test]
    fn every_sort_key_is_accepted_by_the_api() {
        for sort in ALL_SORTS {
            assert!(
                API_SORT_FIELDS.contains(&sort.as_str()),
                "`{:?}` renders as `{}`, which crossref rejects with a 400",
                sort,
                sort.as_str()
            );
        }
    }

    #[test]
    fn sort_round_trips_through_from_str() {
        for sort in ALL_SORTS {
            assert_eq!(Ok(*sort), Sort::from_str(sort.as_str()));
        }
    }

    #[test]
    fn order_round_trips_through_from_str() {
        for order in [Order::Asc, Order::Desc] {
            assert_eq!(Ok(order), Order::from_str(order.as_str()));
        }
    }
}
