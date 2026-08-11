use crate::error::Result;
use crate::query::encode;
use crate::query::works::{WorksCombiner, WorksIdentQuery};
use crate::query::{Component, CrossrefQuery, CrossrefQueryParam, CrossrefRoute, ResourceComponent, ResultControl, format_queries};
use std::borrow::Cow;

impl_terms_query!(
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
    /// `/journals` is one of the narrowest list routes: it takes free form
    /// query terms and paging, and rejects `filter`, `sort`, `order`, `facet`,
    /// `select` and `sample` with a `400`. The query type mirrors that rather
    /// than offering options the route cannot honour.
    JournalsQuery
);

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
