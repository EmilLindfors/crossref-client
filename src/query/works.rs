use crate::error::Result;
use crate::query::facet::FacetCount;
use crate::query::types::Type;
use crate::query::*;
use chrono::NaiveDate;
use std::borrow::Cow;

define_keyed_enum! {
/// A field of a [`Work`](crate::Work) that a query can ask crossref to return.
///
/// Passed to [`WorksQuery::elements`]; crossref returns only what was selected,
/// so every field left out is [`None`] on the resulting `Work`. Covers all 59
/// fields `/works` accepts, which `every_element_is_accepted_by_the_api` pins
/// against what the api reports.
WorkElement {
    DOI => "DOI",
    ISBN => "ISBN",
    ISSN => "ISSN",
    URL => "URL",
    Abstract_ => "abstract",
    Accepted => "accepted",
    AlternativeId => "alternative-id",
    Approved => "approved",
    Archive => "archive",
    ArticleNumber => "article-number",
    Assertion => "assertion",
    Author => "author",
    Chair => "chair",
    ClinicalTrialNumber => "clinical-trial-number",
    ContainerTitle => "container-title",
    ContentCreated => "content-created",
    ContentDomain => "content-domain",
    Contributor => "contributor",
    Created => "created",
    Degree => "degree",
    Deposited => "deposited",
    Editor => "editor",
    Event => "event",
    Funder => "funder",
    GroupTitle => "group-title",
    Indexed => "indexed",
    IsReferencedByCount => "is-referenced-by-count",
    IssnType => "issn-type",
    Issue => "issue",
    Issued => "issued",
    License => "license",
    Link => "link",
    Member => "member",
    OriginalTitle => "original-title",
    Page => "page",
    Posted => "posted",
    Prefix => "prefix",
    Published => "published",
    PublishedOnline => "published-online",
    PublishedPrint => "published-print",
    Publisher => "publisher",
    PublisherLocation => "publisher-location",
    Reference => "reference",
    ReferencesCount => "references-count",
    Relation => "relation",
    Resource => "resource",
    Score => "score",
    ShortContainerTitle => "short-container-title",
    ShortTitle => "short-title",
    StandardsBody => "standards-body",
    Subject => "subject",
    Subtitle => "subtitle",
    Title => "title",
    Translator => "translator",
    Type => "type",
    UpdatePolicy => "update-policy",
    UpdateTo => "update-to",
    UpdatedBy => "updated-by",
    Volume => "volume",
}
}


define_filter! {
/// Narrows a `/works` query. Every filter is ANDed with the others.
///
/// Covers all 90 filters `/works` accepts; crossref answers an unrecognised
/// one with [`Error::ValidationFailure`](crate::Error::ValidationFailure)
/// naming the ones it does know, which is what
/// `every_filter_is_accepted_by_the_api` pins this list against.
///
/// Note that `location` and `reference-visibility` are *not* among them --
/// `location` belongs to [`FundersFilter`](crate::query::funders::FundersFilter).
WorksFilter;
markers {
    /// metadata which includes one or more funder entry
    HasFunder => "has-funder",
    /// metadata where a funder entry carries a Funder Registry DOI
    HasFunderDoi => "has-funder-doi",
    /// metadata where a funder entry carries a ROR id
    HasFunderRorId => "has-funder-ror-id",
    /// metadata for records that have any affiliation information
    HasAffiliation => "has-affiliation",
    /// metadata where an affiliation carries a ROR id
    HasAffiliationRorId => "has-affiliation-ror-id",
    /// metadata carrying a ROR id anywhere
    HasRorId => "has-ror-id",
    /// metadata that includes any `<license_ref>` elements
    HasLicense => "has-license",
    /// metadata that includes any full text `<resource>` elements
    HasFullText => "has-full-text",
    /// metadata for works that have a list of references
    HasReferences => "has-references",
    /// metadata which include name of archive partner
    HasArchive => "has-archive",
    /// metadata which includes one or more ORCIDs
    HasOrcid => "has-orcid",
    /// metadata which includes one or more ORCIDs where the depositing publisher claims to have witness the ORCID owner authenticate with ORCID
    HasAuthenticatedOrcid => "has-authenticated-orcid",
    /// metadata for records which include an abstract
    HasAbstract => "has-abstract",
    /// metadata for records which include a clinical trial number
    HasClinicalTrialNumber => "has-clinical-trial-number",
    /// metadata where the publisher records a domain name location for Crossmark content
    HasContentDomain => "has-content-domain",
    /// metadata where the publisher restricts Crossmark usage to content domains
    HasDomainRestriction => "has-domain-restriction",
    /// metadata for records that either assert or are the object of a relation
    HasRelation => "has-relation",
    /// metadata for records that carry an editorial update
    HasUpdate => "has-update",
    /// metadata for records that include a link to an editorial update policy
    HasUpdatePolicy => "has-update-policy",
    /// metadata for records with any assertions
    HasAssertion => "has-assertion",
    /// metadata for records that carry award information
    HasAward => "has-award",
    /// metadata for records deposited under more than one DOI
    HasAlias => "has-alias",
    /// metadata for records that are the primary DOI of an alias group
    HasPrimeDoi => "has-prime-doi",
    /// metadata for records that describe an event
    HasEvent => "has-event",
    /// metadata for records that represent editorial updates
    IsUpdate => "is-update",
}
values {
    /// metadata which include the `id` in FundRef data
    Funder(String) => "funder",
    /// metadata belonging to a DOI owner prefix `{owner_prefix}` (e.g. 10.1016 )
    Prefix(String) => "prefix",
    /// metadata belonging to a Crossref member
    Member(String) => "member",
    /// metadata describing the DOI
    Doi(String) => "doi",
    /// metadata where record has an ISSN = the value. Format is xxxx-xxxx
    Issn(String) => "issn",
    /// metadata where record has an ISBN = the value
    Isbn(String) => "isbn",
    /// metadata where `<orcid>` element's value = the value
    Orcid(String) => "orcid",
    ///  metadata which where value of archive partner is the value
    Archive(String) => "archive",
    /// metadata records whose article or serial are mentioned in the given value.
    /// Currently the only supported value is `doaj`
    Directory(String) => "directory",
    /// metadata for records that represent editorial updates to the DOI
    Updates(String) => "updates",
    /// metadata for records with a publication title exactly with an exact match
    ContainerTitle(String) => "container-title",
    /// metadata for records with an exact matching category label.
    /// Category labels come from [this list](https://www.elsevier.com/solutions/scopus/content) published by Scopus
    CategoryName(String) => "category-name",
    /// metadata for records with an exacty matching type label
    TypeName(String) => "type-name",
    /// metadata records whose type = value.
    /// Type must be an ID value from the list of types returned by the `/types` resource
    Type(Type) => "type",
    /// metadata for records with a matching group title, as deposited for posted content
    GroupTitle(String) => "group-title",
    /// metadata where the publisher records a particular domain name as the location Crossmark content will appear
    ContentDomain(String) => "content-domain",
    /// metadata for records carrying the given clinical trial number
    ClinicalTrialNumber(String) => "clinical-trial-number",
    /// metadata for records with the given alternative ID,
    /// which may be a publisher-specific ID, or any other identifier a publisher may have provided
    AlternativeId(String) => "alternative-id",
    /// metadata for records with a given article number
    ArticleNumber(String) => "article-number",
    /// metadata carrying the given [ROR](https://ror.org) id, the identifier
    /// crossref now models affiliations and funders with
    RorId(String) => "ror-id",
    /// metadata for editorial updates of the given kind, e.g. `correction` or `retraction`
    UpdateType(String) => "update-type",
    /// metadata where a funder's DOI was asserted by the value, either
    /// `crossref` or `publisher`
    FunderDoiAssertedBy(String) => "funder-doi-asserted-by",
    /// metadata for records with a particular named assertion
    Assertion(String) => "assertion",
    /// metadata for records with an assertion in a particular group
    AssertionGroup(String) => "assertion-group",
    /// metadata for records with a matching award number.
    /// Optionally combine with `award.funder`
    AwardNumber(String) => "award.number",
    /// metadata for records with an award with matching funder.
    /// Optionally combine with `award.number`
    AwardFunder(String) => "award.funder",
    /// metadata for records with an award of at least this amount
    GteAwardAmount(u64) => "gte-award-amount",
    /// metadata for records with an award of at most this amount
    LteAwardAmount(u64) => "lte-award-amount",
    /// metadata where `<license_ref>` value equals the value
    LicenseUrl(String) => "license.url",
    /// metadata where the `<license_ref>`'s applies_to attribute is
    LicenseVersion(String) => "license.version",
    /// metadata where difference between publication date and the `<license_ref>`'s start_date attribute is <= value (in days)
    LicenseDelay(i32) => "license.delay",
    /// metadata where `<resource>` element's content_version attribute is the value
    FullTextVersion(String) => "full-text.version",
    /// metadata where `<resource>` element's content_type attribute is value (e.g. `application/pdf)`
    FullTextType(String) => "full-text.type",
    /// metadata where `<resource>` link has one of the following intended applications: `text-mining`, `similarity-checking` or `unspecified`
    FullTextApplication(String) => "full-text.application",
    /// One of the relation types from the Crossref relations schema
    /// (e.g. `is-referenced-by`, `is-parent-of`, `is-preprint-of`)
    RelationType(String) => "relation.type",
    /// Relations where the object identifier matches the identifier provided
    RelationObject(String) => "relation.object",
    /// One of the identifier types from the Crossref relations schema (e.g. `doi`, `issn`)
    RelationObjectType(String) => "relation.object-type",
    /// metadata indexed since (inclusive)
    FromIndexDate(NaiveDate) => "from-index-date",
    /// metadata indexed before (inclusive)
    UntilIndexDate(NaiveDate) => "until-index-date",
    /// metadata last (re)deposited since (inclusive)
    FromDepositDate(NaiveDate) => "from-deposit-date",
    /// metadata last (re)deposited before (inclusive)
    UntilDepositDate(NaiveDate) => "until-deposit-date",
    /// Metadata updated since (inclusive) {date}.
    /// Currently the same as `from-deposit-date`
    FromUpdateDate(NaiveDate) => "from-update-date",
    /// Metadata updated before (inclusive) {date}.
    /// Currently the same as `until-deposit-date`
    UntilUpdateDate(NaiveDate) => "until-update-date",
    /// metadata first deposited since (inclusive)
    FromCreatedDate(NaiveDate) => "from-created-date",
    /// metadata first deposited before (inclusive)
    UntilCreatedDate(NaiveDate) => "until-created-date",
    /// metadata where published date is since (inclusive)
    FromPubDate(NaiveDate) => "from-pub-date",
    /// metadata where published date is before (inclusive)
    UntilPubDate(NaiveDate) => "until-pub-date",
    /// metadata where online published date is since (inclusive)
    FromOnlinePubDate(NaiveDate) => "from-online-pub-date",
    /// metadata where online published date is before (inclusive)
    UntilOnlinePubDate(NaiveDate) => "until-online-pub-date",
    /// metadata where print published date is since (inclusive)
    FromPrintPubDate(NaiveDate) => "from-print-pub-date",
    /// metadata where print published date is before (inclusive)
    UntilPrintPubDate(NaiveDate) => "until-print-pub-date",
    /// metadata where posted date is since (inclusive)
    FromPostedDate(NaiveDate) => "from-posted-date",
    /// metadata where posted date is before (inclusive)
    UntilPostedDate(NaiveDate) => "until-posted-date",
    /// metadata where accepted date is since (inclusive)
    FromAcceptedDate(NaiveDate) => "from-accepted-date",
    /// metadata where accepted date is before (inclusive)
    UntilAcceptedDate(NaiveDate) => "until-accepted-date",
    /// metadata where approved date is since (inclusive)
    FromApprovedDate(NaiveDate) => "from-approved-date",
    /// metadata where approved date is before (inclusive)
    UntilApprovedDate(NaiveDate) => "until-approved-date",
    /// metadata where an award was made since (inclusive)
    FromAwardedDate(NaiveDate) => "from-awarded-date",
    /// metadata where an award was made before (inclusive)
    UntilAwardedDate(NaiveDate) => "until-awarded-date",
    /// metadata where issued date is since (inclusive)
    FromIssuedDate(NaiveDate) => "from-issued-date",
    /// metadata where issued date is before (inclusive)
    UntilIssuedDate(NaiveDate) => "until-issued-date",
    /// metadata for events starting since (inclusive)
    FromEventStartDate(NaiveDate) => "from-event-start-date",
    /// metadata for events starting before (inclusive)
    UntilEventStartDate(NaiveDate) => "until-event-start-date",
    /// metadata for events ending since (inclusive)
    FromEventEndDate(NaiveDate) => "from-event-end-date",
    /// metadata for events ending before (inclusive)
    UntilEventEndDate(NaiveDate) => "until-event-end-date",
}
}


define_field_queries! {
    /// titles, including the subtitle
    Title => "title" / title,
    /// the title of the containing work, aka `publication.name`
    ContainerTitle => "container-title" / container_title,
    /// author given and family names
    Author => "author" / author,
    /// editor given and family names
    Editor => "editor" / editor,
    /// chair given and family names
    Chair => "chair" / chair,
    /// translator given and family names
    Translator => "translator" / translator,
    /// author, editor, chair and translator given and family names
    Contributor => "contributor" / contributor,
    /// bibliographic information, useful for citation look up. Includes
    /// titles, authors, ISSNs and publication years
    Bibliographic => "bibliographic" / bibliographic,
    /// contributor affiliations
    Affiliation => "affiliation" / affiliation,
    /// the degree a dissertation was awarded for
    Degree => "degree" / degree,
    /// the description deposited for posted content
    Description => "description" / description,
    /// the short form of an event's name
    EventAcronym => "event-acronym" / event_acronym,
    /// where an event was held
    EventLocation => "event-location" / event_location,
    /// an event's name
    EventName => "event-name" / event_name,
    /// who sponsored an event
    EventSponsor => "event-sponsor" / event_sponsor,
    /// an event's theme
    EventTheme => "event-theme" / event_theme,
    /// the name of a funding body, for works whose funder carries no registry DOI
    FunderName => "funder-name" / funder_name,
    /// where the publisher is located
    PublisherLocation => "publisher-location" / publisher_location,
    /// the publisher's name
    PublisherName => "publisher-name" / publisher_name,
    /// the short form of a standards body's name
    StandardsBodyAcronym => "standards-body-acronym" / standards_body_acronym,
    /// the name of the body that issued a standard
    StandardsBodyName => "standards-body-name" / standards_body_name,
}

impl CrossrefQueryParam for FieldQuery {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        vec![(
            Cow::Borrowed(self.name()),
            Cow::Owned(format_query(self.value())),
        )]
    }
}

/// limits from where and how many `Work` items should be returned
#[derive(Debug, Clone)]
pub enum WorkResultControl {
    /// use the standard ResultControl available for all components
    Standard(ResultControl),
    /// If you are expecting results beyond 10K, then use a cursor to deep page through the results
    Cursor {
        /// the cursor token provided by crossref when initially set to a value of `*`
        token: Option<String>,
        /// limit the results
        rows: Option<usize>,
    },
}

impl WorkResultControl {
    /// set a cursor with `*` value, a new cursor will be provided in the `next-cursor` field of the result
    pub fn new_cursor() -> Self {
        WorkResultControl::Cursor {
            token: None,
            rows: None,
        }
    }

    /// create a new Cursor with only a token value
    pub fn cursor(token: &str) -> Self {
        WorkResultControl::Cursor {
            token: Some(token.to_string()),
            rows: None,
        }
    }
}

impl Default for WorkResultControl {
    fn default() -> Self {
        WorkResultControl::new_cursor()
    }
}

impl CrossrefQueryParam for WorkResultControl {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        match self {
            WorkResultControl::Standard(standard) => standard.params(),
            WorkResultControl::Cursor { token, rows } => {
                let mut params = vec![(
                    Cow::Borrowed("cursor"),
                    Cow::Borrowed(token.as_deref().unwrap_or("*")),
                )];
                if let Some(rows) = rows {
                    params.push((Cow::Borrowed("rows"), Cow::Owned(rows.to_string())));
                }
                params
            }
        }
    }
}
///
/// Retrieve a publication by DOI
///
/// # Example
///
/// ```no_run
/// use crossref_client::Works;
///
/// let works = Works::doi("10.1037/0003-066X.59.1.29");
/// ```
///
/// Target the agency of a specific publication, where the str supplied is corresponded to the publication's DOI
///
/// # Example
///
/// ```no_run
/// use crossref_client::Works;
///
/// let works = Works::agency_for_doi("10.1037/0003-066X.59.1.29");
/// ```
#[derive(Debug, Clone)]
pub enum Works {
    /// target a Work by a specific id
    Identifier(String),
    /// target Works by a query
    Query(WorksQuery),
    /// return the registration agency for a DOI
    Agency(String),
}

impl Works {
    /// create a new `Works::Identifier` by converting `doi` to a `String`
    pub fn doi(doi: &str) -> Self {
        Works::Identifier(doi.to_string())
    }
    /// create a new `Works::Agency` targeting the registration agency for the DOI
    pub fn agency_for_doi(doi: &str) -> Self {
        Works::Agency(doi.to_string())
    }
}

impl CrossrefRoute for Works {
    fn route(&self) -> Result<String> {
        match self {
            Works::Identifier(s) => Ok(format!("{}/{}", Component::Works.route()?, s)),
            Works::Agency(s) => Ok(format!("{}/{}/agency", Component::Works.route()?, s)),
            Works::Query(query) => query.route(),
        }
    }
}

impl CrossrefQuery for Works {
    fn resource_component(self) -> ResourceComponent {
        ResourceComponent::Works(self)
    }
}

/// Wraps queries that target `WorkList`, either directly or combined
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum WorkListQuery {
    /// Target `Works` directly
    Works(WorksQuery),
    /// Target the corresponding `Works` of a specific `Component`
    Combined {
        primary_component: Component,
        ident: WorksIdentQuery,
    },
}

impl WorkListQuery {
    /// the underlying `WorksQuery`
    pub fn query(&self) -> &WorksQuery {
        match self {
            WorkListQuery::Works(query) => query,
            WorkListQuery::Combined { ident, .. } => &ident.query,
        }
    }

    /// mut reference to the underlying `Worksquery`
    pub fn query_mut(&mut self) -> &mut WorksQuery {
        match self {
            WorkListQuery::Works(query) => query,
            WorkListQuery::Combined { ident, .. } => &mut ident.query,
        }
    }
}

impl From<WorksQuery> for WorkListQuery {
    fn from(val: WorksQuery) -> Self {
        WorkListQuery::Works(val)
    }
}

impl<T: ToString> From<T> for WorkListQuery {
    fn from(term: T) -> Self {
        WorkListQuery::Works(WorksQuery::new(term))
    }
}

impl CrossrefRoute for WorkListQuery {
    fn route(&self) -> Result<String> {
        match self {
            WorkListQuery::Works(query) => query.route(),
            WorkListQuery::Combined {
                primary_component,
                ident,
            } => Ok(format!(
                "{}/{}{}",
                primary_component.route()?,
                ident.id,
                ident.query.route()?
            )),
        }
    }
}

impl CrossrefQuery for WorkListQuery {
    fn resource_component(self) -> ResourceComponent {
        match self {
            WorkListQuery::Works(query) => ResourceComponent::Works(Works::Query(query)),
            WorkListQuery::Combined {
                primary_component,
                ident,
            } => match primary_component {
                Component::Funders => ResourceComponent::Funders(Funders::Works(ident)),
                Component::Journals => ResourceComponent::Journals(Journals::Works(ident)),
                Component::Members => ResourceComponent::Members(Members::Works(ident)),
                Component::Prefixes => ResourceComponent::Prefixes(Prefixes::Works(ident)),
                Component::Types => ResourceComponent::Types(Types::Works(ident)),
                Component::Works => ResourceComponent::Works(Works::Query(ident.query)),
            },
        }
    }
}

/// Target `Works` as secondary resource component
///
/// # Example
///
/// ```no_run
/// use crossref_client::{WorksIdentQuery, WorksQuery};
///
/// let combined = WorksIdentQuery::new("100000015", WorksQuery::new("ontologies"));
///
/// ```
/// Is equal to create a `WorksIdentQuery` from a `WorksQuery`
///
/// ```no_run
/// use crossref_client::WorksQuery;
///
/// let combined = WorksQuery::new("ontologies").into_ident("100000015");
///
/// ```
/// helper struct to capture an id for a `Component` other than `/works` and an additional query for the `/works` route
#[derive(Debug, Clone)]
pub struct WorksIdentQuery {
    /// the id of an component item
    pub id: String,
    /// the query to filter the works results
    pub query: WorksQuery,
}

impl WorksIdentQuery {
    /// create a new Ident Query for the `id`
    pub fn new<T: Into<String>>(id: T, query: WorksQuery) -> Self {
        WorksIdentQuery {
            id: id.into(),
            query,
        }
    }
}

/// Trait to determine that the type can be used in a combined query
pub trait WorksCombiner {
    /// the primary component of this type
    fn primary_component() -> Component;

    /// construct a new type
    fn ident_query(ident: WorksIdentQuery) -> Self;

    /// the combined crossref route
    fn combined_route(ident: &WorksIdentQuery) -> Result<String> {
        Ok(format!(
            "{}/{}{}",
            Self::primary_component().route()?,
            ident.id,
            ident.query.route()?
        ))
    }

    /// create a new combined `WorkListQuery` with the primary component
    fn work_list_query(ident: WorksIdentQuery) -> WorkListQuery {
        WorkListQuery::Combined {
            primary_component: Self::primary_component(),
            ident,
        }
    }
}

macro_rules! impl_combiner {
    ($($name:ident,)*) => {
        $(
        impl WorksCombiner for $name {
            fn primary_component() -> Component {
                Component::$name
            }

            fn ident_query(ident: WorksIdentQuery) -> Self {
                $name::Works(ident)
            }
        }
        )+
    };
}

impl_combiner!(Journals, Funders, Members, Prefixes, Types,);

impl WorksQuery {
    /// alias for creating an empty default element
    pub fn empty() -> Self {
        WorksQuery::default()
    }

    /// creates an new `WorksQuery` with the desired sample size that will result in
    /// a request for random dois
    pub fn random(len: usize) -> Self {
        WorksQuery::default().sample(len)
    }

    /// Convenience method to create a new `WorksQuery` with a term directly
    pub fn new<T: ToString>(query: T) -> Self {
        WorksQuery::empty().query(query)
    }

    /// Ask for `len` random works instead of a search, or for a search if
    /// given [`None`]. Crossref ignores every other parameter when it is set.
    pub fn sample(mut self, len: impl Into<Option<usize>>) -> Self {
        self.sample = len.into();
        self
    }

    /// add a new free form query
    pub fn query<T: ToString>(mut self, query: T) -> Self {
        self.free_form_queries.push(query.to_string());
        self
    }

    /// Create a new query for the topics renear+ontologies
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crossref_client::WorksQuery;
    ///
    /// let query = WorksQuery::default().queries(&["renear", "ontologies"]);
    /// ```
    /// add a bunch of free form query terms
    pub fn queries<T: ToString>(mut self, queries: &[T]) -> Self {
        self.free_form_queries
            .extend(queries.iter().map(T::to_string));
        self
    }

    /// add a new field query form query
    pub fn field_query(mut self, query: FieldQuery) -> Self {
        self.field_queries.push(query);
        self
    }

    /// Narrow the response to the given fields.
    ///
    /// Crossref returns only what is selected, so every field left out is
    /// [`None`] on the resulting [`Work`](crate::Work). Include
    /// [`WorkElement::DOI`] unless you have no use for it -- it is the one
    /// field [`Work`](crate::Work) still requires.
    pub fn elements(mut self, element: Vec<WorkElement>) -> Self {
        self.elements.extend(element);
        self
    }

    /// ```no_run
    /// use crossref_client::{FieldQuery,WorksQuery};
    ///
    /// let query = WorksQuery::default().field_queries(vec![FieldQuery::title("room at the bottom"), FieldQuery::author("richard feynman")]);
    /// ```
    /// add a bunch of free form query terms
    pub fn field_queries(mut self, queries: Vec<FieldQuery>) -> Self {
        self.field_queries.extend(queries);
        self
    }

    /// add a new filter to the query
    pub fn filter(mut self, filter: WorksFilter) -> Self {
        self.filter.push(filter);
        self
    }

    /// Sort the results by a field, or by relevance if given [`None`].
    pub fn sort(mut self, sort: impl Into<Option<Sort>>) -> Self {
        self.sort = sort.into();
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

    /// set the cursor for result control deep paging
    pub fn next_cursor(mut self, cursor: &str) -> Self {
        let rows = match self.result_control {
            Some(WorkResultControl::Standard(ResultControl::Rows(rows))) => Some(rows),
            _ => None,
        };
        self.result_control = Some(WorkResultControl::Cursor {
            token: Some(cursor.to_string()),
            rows,
        });
        self
    }

    /// set an empty cursor
    pub fn new_cursor(mut self) -> Self {
        self.result_control = Some(WorkResultControl::new_cursor());
        self
    }
    /// Limit the results, or take crossref's default page if given [`None`].
    pub fn result_control(
        mut self,
        result_control: impl Into<Option<WorkResultControl>>,
    ) -> Self {
        self.result_control = result_control.into();
        self
    }

    /// Wrap the query in a combined query.
    ///
    /// # Example
    /// Create a Funders Query that targets all works of a funder with id `funder id`.
    ///
    /// ```no_run
    /// # use crossref_client::{WorksQuery, Funders};
    /// let funders: Funders = WorksQuery::default().into_combined("funder id");
    /// ```
    pub fn into_combined<W: WorksCombiner>(self, id: &str) -> W {
        W::ident_query(self.into_ident(id))
    }

    /// Bind the query to a specific id of a primary endpoint element
    pub fn into_ident(self, id: &str) -> WorksIdentQuery {
        WorksIdentQuery::new(id, self)
    }

    /// wrap this query in new `WorkListQuery` that targets the `/works` route of a primary component with an id.
    /// The query will evaluate to the same as [`WorksQuery::into_combined`]
    ///
    /// # Example
    ///
    /// Create a query that targets all `Works` of a funder with id `funder id`
    ///
    /// ```no_run
    /// # use crossref_client::{WorksQuery, Funders};
    /// let query = WorksQuery::default()
    ///     .into_combined_query::<Funders>("funder id");
    ///
    /// ```
    pub fn into_combined_query<W: WorksCombiner>(self, id: &str) -> WorkListQuery {
        W::work_list_query(self.into_ident(id))
    }
}

/// Used to construct a query that targets crossref `Works` elements
///
/// # Example
///
/// ```no_run
/// use crossref_client::{Order, WorksQuery};
///
/// // create a new query for topcis machine+learning ordered desc
/// let query = WorksQuery::new("machine learning").order(Order::Desc);
/// ```
///
/// Each query parameter is ANDed
#[derive(Debug, Clone, Default)]
pub struct WorksQuery {
    /// search by non specific query
    pub free_form_queries: Vec<String>,
    /// match only particular fields of metadata
    pub field_queries: Vec<FieldQuery>,
    /// filter to apply while querying
    pub filter: Vec<WorksFilter>,
    /// sort results by a certain field and
    pub sort: Option<Sort>,
    /// set the sort order to `asc` or `desc`
    pub order: Option<Order>,
    /// elements to return
    pub elements: Vec<WorkElement>,
    /// enable facet information in responses
    pub facets: Vec<FacetCount>,
    /// deep page through `/works` result sets
    pub result_control: Option<WorkResultControl>,
    /// request random dois
    /// if set all other parameters are ignored
    pub sample: Option<usize>,
}

impl CrossrefRoute for WorksQuery {
    fn route(&self) -> Result<String> {
        let mut params: Vec<(Cow<'_, str>, Cow<'_, str>)> = Vec::new();

        if let Some(sample) = self.sample {
            params.push((Cow::Borrowed("sample"), Cow::Owned(sample.to_string())));
            return Ok(format!(
                "{}{}",
                Component::Works.route()?,
                encode::query_string(&params)
            ));
        }

        if !self.free_form_queries.is_empty() {
            params.push((
                Cow::Borrowed("query"),
                Cow::Owned(format_queries(&self.free_form_queries)),
            ));
        }
        params.extend(self.field_queries.iter().flat_map(CrossrefQueryParam::params));
        if !self.filter.is_empty() {
            params.extend(self.filter.params());
        }
        if !self.elements.is_empty() {
            params.push((
                Cow::Borrowed("select"),
                Cow::Owned(
                    self.elements
                        .iter()
                        .map(WorkElement::name)
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            ));
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

        Ok(format!(
            "{}{}",
            Component::Works.route()?,
            encode::query_string(&params)
        ))
    }
}

impl CrossrefParams for WorksQuery {
    type Filter = WorksFilter;

    fn query_terms(&self) -> &[String] {
        &self.free_form_queries
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
        if let Some(WorkResultControl::Standard(ref std)) = self.result_control {
            Some(std)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_works_ident() {
        let works = Works::doi("10.1037/0003-066X.59.1.29");

        assert_eq!("/works/10.1037/0003-066X.59.1.29", &works.route().unwrap())
    }

    #[test]
    fn sample_query_keeps_the_works_route() {
        // a bare `sample=n` used to be returned without the `/works?` prefix,
        // which concatenated straight onto the base url
        let route = WorksQuery::random(10).route().unwrap();

        assert_eq!("/works?sample=10", &route);

        let query: WorkListQuery = WorksQuery::random(10).into();
        assert_eq!(
            "https://api.crossref.org/works?sample=10",
            &query.to_url("https://api.crossref.org").unwrap()
        );
    }

    #[test]
    fn filter_names_have_no_stray_whitespace() {
        let route = WorksQuery::empty()
            .filter(WorksFilter::HasClinicalTrialNumber)
            .route()
            .unwrap();

        assert_eq!("/works?filter=has-clinical-trial-number:true", &route);
    }

    #[test]
    fn cursor_with_rows_renders_two_parameters() {
        let query = WorksQuery::empty().result_control(WorkResultControl::Cursor {
            token: Some("abc".to_string()),
            rows: Some(20),
        });

        assert_eq!("/works?cursor=abc&rows=20", &query.route().unwrap());
    }

    #[test]
    fn new_cursor_renders_the_wildcard_token() {
        let query = WorksQuery::empty().new_cursor();

        assert_eq!("/works?cursor=*", &query.route().unwrap());
    }

    #[test]
    fn a_query_term_can_no_longer_inject_a_parameter() {
        // `/works?query=R&D` reaches crossref as `query=R` plus a stray `D`
        assert_eq!(
            "/works?query=R%26D",
            &WorksQuery::new("R&D").route().unwrap()
        );
    }

    #[test]
    fn filter_and_field_query_values_are_encoded_too() {
        let route = WorksQuery::empty()
            .field_query(FieldQuery::container_title("Ecology & Evolution"))
            .filter(WorksFilter::ContainerTitle("Q&A".to_string()))
            .route()
            .unwrap();

        assert_eq!(
            "/works?query.container-title=Ecology%20%26%20Evolution&filter=container-title:Q%26A",
            &route
        );
    }

    #[test]
    fn an_empty_query_targets_the_bare_works_route() {
        assert_eq!("/works", &WorksQuery::empty().route().unwrap());
    }

    #[test]
    fn selected_elements_render_as_one_comma_separated_parameter() {
        let route = WorksQuery::empty()
            .elements(vec![WorkElement::DOI, WorkElement::Title])
            .route()
            .unwrap();

        assert_eq!("/works?select=DOI,title", &route);
    }

    #[test]
    fn rows_offset_renders_two_parameters() {
        let query = WorksQuery::empty().result_control(WorkResultControl::Standard(
            ResultControl::RowsOffset {
                rows: 10,
                offset: 20,
            },
        ));

        assert_eq!("/works?rows=10&offset=20", &query.route().unwrap());
    }

    #[test]
    fn referenced_by_count_uses_the_crossref_spelling() {
        // `is-reference-by-count` is rejected by the api with a 400
        assert_eq!("is-referenced-by-count", Sort::IsReferencedByCount.as_str());
    }
}
