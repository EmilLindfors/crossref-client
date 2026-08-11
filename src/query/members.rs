use crate::error::Result;
use crate::query::works::{WorksCombiner, WorksIdentQuery};
use crate::query::*;
use std::borrow::Cow;

define_filter! {
/// Narrows a `/members` query.
///
/// `has-public-references` and `reference-visibility` used to be here; the
/// route rejects both with a `400`. `blackfile-doi-count` was a typo for
/// `backfile-doi-count` and never matched anything either.
MembersFilter;
markers {}
values {
    /// members that own the given DOI prefix
    Prefix(String) => "prefix",
    /// count of DOIs for material published more than two years ago
    BackfileDoiCount(i32) => "backfile-doi-count",
    /// count of DOIs for material published within the last two years
    CurrentDoiCount(i32) => "current-doi-count",
}
}

impl_common_query!(MembersQuery, MembersFilter);

/// constructs the request payload for the `/members` route
#[derive(Debug, Clone)]
pub enum Members {
    /// target a specific member at `/members/{id}`
    Identifier(String),
    /// target all members that match the query at `/members?query...`
    Query(MembersQuery),
    /// target a `Work` for a specific funder at `/members/{id}/works?query..`
    Works(WorksIdentQuery),
}

impl CrossrefRoute for Members {
    fn route(&self) -> Result<String> {
        match self {
            Members::Identifier(s) => Ok(format!("{}/{}", Component::Members.route()?, s)),
            // `route` already carries its own `?` and is empty for an empty query
            Members::Query(query) => {
                Ok(format!("{}{}", Component::Members.route()?, query.route()?))
            }
            Members::Works(combined) => Self::combined_route(combined),
        }
    }
}

impl CrossrefQuery for Members {
    fn resource_component(self) -> ResourceComponent {
        ResourceComponent::Members(self)
    }
}
