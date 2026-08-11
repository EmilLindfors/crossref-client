use crate::error::Result;
use crate::query::encode;
use crate::query::works::{WorksCombiner, WorksIdentQuery};
use crate::query::{Component, CrossrefQuery, CrossrefQueryParam, CrossrefRoute, ResourceComponent, ResultControl, format_queries};
use std::borrow::Cow;

/// Used to construct a query that targets crossref `Journal` elements
///
/// # Example
///
/// ```no_run
/// use crossref_client::{JournalsQuery, ResultControl};
///
/// let query = JournalsQuery::new("Economic Geography")
///     .result_control(ResultControl::Rows(10));
/// ```
///
/// `/journals` is the narrowest of the list routes: it takes free form query
/// terms and paging, and rejects `filter`, `sort`, `order`, `facet`, `select`
/// and `sample` with a `400`. The query type mirrors that rather than offering
/// options the route cannot honour.
#[derive(Debug, Clone, Default)]
pub struct JournalsQuery {
    /// search by non specific query
    pub queries: Vec<String>,
    /// limit the returned items and set an offset
    ///
    /// Only `rows` and `offset` are supported by this route;
    /// [`ResultControl::Sample`] is rejected by crossref with a `400`.
    pub result_control: Option<ResultControl>,
}

impl JournalsQuery {
    /// alias for creating an empty default element
    pub fn empty() -> Self {
        JournalsQuery::default()
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

    /// set result control option to query
    pub fn result_control(mut self, result_control: ResultControl) -> Self {
        self.result_control = Some(result_control);
        self
    }
}

impl CrossrefRoute for JournalsQuery {
    fn route(&self) -> Result<String> {
        let mut params: Vec<(Cow<'_, str>, Cow<'_, str>)> = Vec::new();
        if !self.queries.is_empty() {
            params.push((
                Cow::Borrowed("query"),
                Cow::Owned(format_queries(&self.queries)),
            ));
        }
        if let Some(rc) = &self.result_control {
            params.extend(rc.params());
        }
        Ok(encode::query_string(&params))
    }
}

/// constructs the request payload for the `/journals` route
#[derive(Debug, Clone)]
pub enum Journals {
    /// target a specific journal at `/journals/{id}`
    Identifier(String),
    /// target a `Work` for a specific journal at `/journals/{id}/works?query..`
    Works(WorksIdentQuery),
    /// target all journals that match the query at `/journals?query...`
    Query(JournalsQuery),
}

impl CrossrefRoute for Journals {
    fn route(&self) -> Result<String> {
        match self {
            Journals::Identifier(s) => Ok(format!("{}/{}", Component::Journals.route()?, s)),
            // `route` already carries its own `?` and is empty for an empty query
            Journals::Query(query) => {
                Ok(format!("{}{}", Component::Journals.route()?, query.route()?))
            }
            Journals::Works(combined) => Self::combined_route(combined),
        }
    }
}

impl CrossrefQuery for Journals {
    fn resource_component(self) -> ResourceComponent {
        ResourceComponent::Journals(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_query_targets_the_bare_route() {
        assert_eq!(
            "/journals",
            &Journals::Query(JournalsQuery::empty()).route().unwrap()
        );
    }

    #[test]
    fn terms_and_paging_render_as_separate_parameters() {
        let query = JournalsQuery::new("Economic Geography")
            .result_control(ResultControl::RowsOffset { rows: 10, offset: 20 });

        assert_eq!(
            "/journals?query=Economic%20Geography&rows=10&offset=20",
            &Journals::Query(query).route().unwrap()
        );
    }
}
