use crate::error::Result;
use crate::query::encode;
use crate::query::{
    Component, CrossrefQuery, CrossrefQueryParam, CrossrefRoute, ResourceComponent, ResultControl,
    format_queries,
};
use std::borrow::Cow;

impl_terms_query!(
    /// Used to construct a query that targets the licenses crossref works are
    /// published under
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::{LicensesQuery, ResultControl};
    ///
    /// let query = LicensesQuery::new("creative commons")
    ///     .result_control(ResultControl::Rows(10));
    /// ```
    ///
    /// Like `/journals`, this route takes free form query terms and paging and
    /// nothing else -- it rejects `filter`, `sort`, `order`, `facet`, `select`
    /// and `sample` with a `400`.
    LicensesQuery
);

/// constructs the request payload for the `/licenses` route
///
/// The route lists license urls with the number of works published under each,
/// and has no per-license or combined `works` sub-route, so a query is the only
/// way to address it.
#[derive(Debug, Clone)]
pub enum Licenses {
    /// target all licenses that match the query at `/licenses?query...`
    Query(LicensesQuery),
}

impl CrossrefRoute for Licenses {
    fn route(&self) -> Result<String> {
        match self {
            // `route` already carries its own `?` and is empty for an empty query
            Licenses::Query(query) => {
                Ok(format!("{}{}", Component::Licenses.route()?, query.route()?))
            }
        }
    }
}

impl CrossrefQuery for Licenses {
    fn resource_component(self) -> ResourceComponent {
        ResourceComponent::Licenses(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_query_targets_the_bare_route() {
        assert_eq!(
            "/licenses",
            &Licenses::Query(LicensesQuery::empty()).route().unwrap()
        );
    }

    #[test]
    fn terms_and_paging_render_as_separate_parameters() {
        let query = LicensesQuery::new("creative commons").result_control(ResultControl::Rows(10));

        assert_eq!(
            "/licenses?query=creative%20commons&rows=10",
            &Licenses::Query(query).route().unwrap()
        );
    }
}
