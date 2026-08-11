use crate::error::Result;
use crate::query::works::{WorksCombiner, WorksIdentQuery};
use crate::query::*;
use std::borrow::Cow;

define_filter! {
/// Narrows a `/funders` query.
///
/// `location` is the only filter this route accepts. It used to also sit on
/// [`WorksFilter`](crate::WorksFilter), where `/works` rejected it with a `400`.
FundersFilter;
markers {}
values {
    /// funders located in the named country
    Location(String) => "location",
}
}

impl_list_query!(
    /// Used to construct a query that targets crossref `Funder` elements
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::{FundersFilter, FundersQuery, ResultControl};
    ///
    /// let query = FundersQuery::new("NSF")
    ///     .filter(FundersFilter::Location("Norway".to_string()))
    ///     .result_control(ResultControl::Rows(10));
    /// ```
    ///
    /// The route takes terms, one filter and paging. It used to also offer
    /// `sort`, `order` and `facet`, which crossref answers with a `400`.
    FundersQuery,
    filter = FundersFilter
);

/// constructs the request payload for the `/funders` route
#[derive(Debug, Clone)]
pub enum Funders {
    /// target a specific funder at `/funder/{id}`
    Identifier(String),
    /// target all funders that match the query at `/funders?query...`
    Query(FundersQuery),
    /// target a `Work` for a specific funder at `/funders/{id}/works?query..`
    Works(WorksIdentQuery),
}

impl CrossrefRoute for Funders {
    fn route(&self) -> Result<String> {
        match self {
            Funders::Identifier(s) => Ok(format!("{}/{}", Component::Funders.route()?, s)),
            // `route` already carries its own `?` and is empty for an empty query
            Funders::Query(query) => {
                Ok(format!("{}{}", Component::Funders.route()?, query.route()?))
            }
            Funders::Works(combined) => Self::combined_route(combined),
        }
    }
}

impl CrossrefQuery for Funders {
    fn resource_component(self) -> ResourceComponent {
        ResourceComponent::Funders(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terms_filter_and_paging_render_as_separate_parameters() {
        let query = FundersQuery::new("NSF")
            .filter(FundersFilter::Location("Norway".to_string()))
            .result_control(ResultControl::Rows(10));

        assert_eq!(
            "/funders?query=NSF&filter=location:Norway&rows=10",
            &Funders::Query(query).route().unwrap()
        );
    }

    #[test]
    fn an_empty_query_targets_the_bare_route() {
        assert_eq!(
            "/funders",
            &Funders::Query(FundersQuery::empty()).route().unwrap()
        );
    }
}
