use crate::error::Result;
use crate::query::facet::FacetCount;
use crate::query::types::Type;
use crate::query::*;
use chrono::NaiveDate;
use std::borrow::Cow;

/// Filters allow you to narrow queries. All filter results are lists
#[derive(Debug, Clone)]
pub enum WorkElement {
    DOI,
    ISBN,
    ISSN,
    URL,
    Abstract_,
    Accepted,
    AlternativeId,
    Approved,
    Archive,
    ArticleNumber,
    Assertion,
    Author,
    Chair,
    ClinicalTrialNumber,
    ContainerTitle,
    ContentCreated,
    ContentDomain,
    Created,
    Degree,
    Deposited,
    Editor,
    Event,
    Funder,
    GroupTitle,
    Indexed,
    IsReferencedByCount,
    IssnType,
    Issue,
    Issued,
    License,
    Link,
    Member,
    OriginalTitle,
    Page,
    Posted,
    Prefix,
    Published,
    PublishedOnline,
    PublishedPrint,
    Publisher,
    PublisherLocation,
    Reference,
    ReferencesCount,
    Relation,
    Score,
    ShortContainerTitle,
    ShortTitle,
    StandardsBody,
    Subject,
    Subtitle,
    Title,
    Translator,
    Type,
    UpdatePolicy,
    UpdateTo,
    UpdatedBy,
    Volume,
}

impl WorkElement {
    /// the identifier for a the query key
    pub fn name(&self) -> &str {
        match self {
            WorkElement::DOI => "DOI",
            WorkElement::ISBN => "ISBN",
            WorkElement::ISSN => "ISSN",
            WorkElement::URL => "URL",
            WorkElement::Abstract_ => "abstract",
            WorkElement::Accepted => "accepted",
            WorkElement::AlternativeId => "alternative-id",
            WorkElement::Approved => "approved",
            WorkElement::Archive => "archive",
            WorkElement::ArticleNumber => "article-number",
            WorkElement::Assertion => "assertion",
            WorkElement::Author => "author",
            WorkElement::Chair => "chair",
            WorkElement::ClinicalTrialNumber => "clinical-trial-number",
            WorkElement::ContainerTitle => "container-title",
            WorkElement::ContentCreated => "content-created",
            WorkElement::ContentDomain => "content-domain",
            WorkElement::Created => "created",
            WorkElement::Degree => "degree",
            WorkElement::Deposited => "deposited",
            WorkElement::Editor => "editor",
            WorkElement::Event => "event",
            WorkElement::Funder => "funder",
            WorkElement::GroupTitle => "group-title",
            WorkElement::Indexed => "indexed",
            WorkElement::IsReferencedByCount => "is-referenced-by-count",
            WorkElement::IssnType => "issn-type",
            WorkElement::Issue => "issue",
            WorkElement::Issued => "issued",
            WorkElement::License => "license",
            WorkElement::Link => "link",
            WorkElement::Member => "member",
            WorkElement::OriginalTitle => "original-title",
            WorkElement::Page => "page",
            WorkElement::Posted => "posted",
            WorkElement::Prefix => "prefix",
            WorkElement::Published => "published",
            WorkElement::PublishedOnline => "published-online",
            WorkElement::PublishedPrint => "published-print",
            WorkElement::Publisher => "publisher",
            WorkElement::PublisherLocation => "publisher-location",
            WorkElement::Reference => "reference",
            WorkElement::ReferencesCount => "references-count",
            WorkElement::Relation => "relation",
            WorkElement::Score => "score",
            WorkElement::ShortContainerTitle => "short-container-title",
            WorkElement::ShortTitle => "short-title",
            WorkElement::StandardsBody => "standards-body",
            WorkElement::Subject => "subject",
            WorkElement::Subtitle => "subtitle",
            WorkElement::Title => "title",
            WorkElement::Translator => "translator",
            WorkElement::Type => "type",
            WorkElement::UpdatePolicy => "update-policy",
            WorkElement::UpdateTo => "update-to",
            WorkElement::UpdatedBy => "updated-by",
            WorkElement::Volume => "volume",
        }
    }
}


/// Filters allow you to narrow queries. All filter results are lists
#[derive(Debug, Clone)]
pub enum WorksFilter {
    /// metadata which includes one or more funder entry
    HasFunder,
    /// metadata which include the `id` in FundRef data
    Funder(String),
    /// funder records where location = `{country name}`.
    /// Only works on `/funders` route
    Location(String),
    /// metadata belonging to a DOI owner prefix `{owner_prefix}` (e.g. 10.1016 )
    Prefix(String),
    /// metadata belonging to a Crossref member
    Member(String),
    /// metadata indexed since (inclusive)
    FromIndexDate(NaiveDate),
    /// metadata indexed before (inclusive)
    UntilIndexDate(NaiveDate),
    /// metadata last (re)deposited since (inclusive)
    FromDepositDate(NaiveDate),
    /// metadata last (re)deposited before (inclusive)
    UntilDepositDate(NaiveDate),
    /// Metadata updated since (inclusive) {date}.
    /// Currently the same as `from-deposit-date`
    FromUpdateDate(NaiveDate),
    /// Metadata updated before (inclusive) {date}.
    /// Currently the same as `until-deposit-date`
    UntilUpdateDate(NaiveDate),
    /// metadata first deposited since (inclusive)
    FromCreatedDate(NaiveDate),
    /// metadata first deposited before (inclusive)
    UntilCreatedDate(NaiveDate),
    /// metadata where published date is since (inclusive)
    FromPubDate(NaiveDate),
    /// metadata where published date is before (inclusive)
    UntilPubDate(NaiveDate),
    /// metadata where online published date is since (inclusive)
    FromOnlinePubDate(NaiveDate),
    /// metadata where online published date is before (inclusive)
    UntilOnlinePubDate(NaiveDate),
    /// metadata where print published date is since (inclusive)
    FromPrintPubDate(NaiveDate),
    /// metadata where print published date is before (inclusive)
    UntilPrintPubDate(NaiveDate),
    /// metadata where posted date is since (inclusive)
    FromPostedDate(NaiveDate),
    /// metadata where posted date is before (inclusive)
    UntilPostedDate(NaiveDate),
    /// metadata where accepted date is since (inclusive)
    FromAcceptedDate(NaiveDate),
    /// metadata where accepted date is before (inclusive)
    UntilAcceptedDate(NaiveDate),
    /// metadata that includes any `<license_ref>` elements.
    HasLicense,
    /// metadata where `<license_ref> value equals the value
    LicenseUrl(String),
    /// metadata where the `<license_ref>`'s applies_to attribute is
    LicenseVersion(String),
    /// metadata where difference between publication date and the `<license_ref>`'s start_date attribute is <= value (in days)
    LicenseDelay(i32),
    /// metadata that includes any full text `<resource>` elements
    HasFullText,
    /// metadata where `<resource>` element's content_version attribute is the value
    FullTextVersion(String),
    /// metadata where `<resource>` element's content_type attribute is value (e.g. `application/pdf)`
    FullTextType(String),
    /// metadata where `<resource>` link has one of the following intended applications: `text-mining`, `similarity-checking` or `unspecified`
    FullTextApplication(String),
    /// metadata for works that have a list of references
    HasReferences,
    /// metadata for works where references are either `open`, `limited` (to Metadata Plus subscribers) or `closed`
    ReferenceVisibility(Visibility),
    /// metadata which include name of archive partner
    HasArchive,
    ///  metadata which where value of archive partner is the value
    Archive(String),
    /// metadata which includes one or more ORCIDs
    HasOrcid,
    /// metadata which includes one or more ORCIDs where the depositing publisher claims to have witness the ORCID owner authenticate with ORCID
    HasAuthenticatedOrcid,
    /// metadata where `<orcid>` element's value = the value
    Orcid(String),
    /// metadata where record has an ISSN = the value. Format is xxxx-xxxx
    Issn(String),
    /// metadata where record has an ISBN = the value
    Isbn(String),
    /// metadata records whose type = value.
    /// Type must be an ID value from the list of types returned by the `/types` resource
    Type(Type),
    /// metadata records whose article or serial are mentioned in the given value.
    /// Currently the only supported value is `doaj`
    Directory(String),
    /// metadata describing the DOI
    Doi(String),
    /// metadata for records that represent editorial updates to the DOI
    Updates(String),
    /// metadata for records that represent editorial updates
    IsUpdate,
    /// metadata for records that include a link to an editorial update policy
    HasUpdatePolicy,
    /// metadata for records with a publication title exactly with an exact match
    ContainerTitle(String),
    /// metadata for records with an exact matching category label.
    /// Category labels come from [this list](https://www.elsevier.com/solutions/scopus/content) published by Scopus
    CategoryName(String),
    /// metadata for records with an exacty matching type label
    TypeName(String),
    /// metadata for records with a matching award number.
    /// Optionally combine with `award.funder`
    AwardNumber(String),
    /// metadata for records with an award with matching funder.
    /// Optionally combine with `award.number`
    AwardFunder(String),
    /// metadata for records with any assertions
    HasAssertion,
    /// metadata for records with an assertion in a particular group
    AssertionGroup(String),
    /// metadata for records with a particular named assertion
    Assertion(String),
    /// metadata for records that have any affiliation information
    HasAffiliation,
    /// metadata for records with the given alternative ID,
    /// which may be a publisher-specific ID, or any other identifier a publisher may have provided
    AlternativeId,
    /// metadata for records with a given article number
    ArticleNumber,
    /// metadata for records which include an abstract
    HasAbstract,
    /// metadata for records which include a clinical trial number
    HasClinicalTrialNumber,
    /// metadata where the publisher records a particular domain name as the location Crossmark content will appear
    ContentDomain(String),
    /// metadata where the publisher records a domain name location for Crossmark content
    HasContentDomain,
    /// metadata where the publisher restricts Crossmark usage to content domains
    HasDomainRestriction,
    /// metadata for records that either assert or are the object of a relation
    HasRelation,
    /// One of the relation types from the Crossref relations schema
    /// (e.g. `is-referenced-by`, `is-parent-of`, `is-preprint-of`)
    RelationType,
    /// Relations where the object identifier matches the identifier provided
    RelationObject,
    /// One of the identifier types from the Crossref relations schema (e.g. `doi`, `issn`)
    RelationObjectType(String),
}

impl WorksFilter {
    /// the identifier for a the query key
    pub fn name(&self) -> &str {
        match self {
            WorksFilter::HasFunder => "has-funder",
            WorksFilter::Funder(_) => "funder",
            WorksFilter::Location(_) => "location",
            WorksFilter::Prefix(_) => "prefix",
            WorksFilter::Member(_) => "member",
            WorksFilter::FromIndexDate(_) => "from-index-date",
            WorksFilter::UntilIndexDate(_) => "until-index-date",
            WorksFilter::FromDepositDate(_) => "from-deposit-date",
            WorksFilter::UntilDepositDate(_) => "until-deposit-date",
            WorksFilter::FromUpdateDate(_) => "from-update-date",
            WorksFilter::UntilUpdateDate(_) => "until-update-date",
            WorksFilter::FromCreatedDate(_) => "from-created-date",
            WorksFilter::UntilCreatedDate(_) => "until-created-date",
            WorksFilter::FromPubDate(_) => "from-pub-date",
            WorksFilter::UntilPubDate(_) => "until-pub-date",
            WorksFilter::FromOnlinePubDate(_) => "from-online-pub-date",
            WorksFilter::UntilOnlinePubDate(_) => "until-online-pub-date",
            WorksFilter::FromPrintPubDate(_) => "from-print-pub-date",
            WorksFilter::UntilPrintPubDate(_) => "until-print-pub-date",
            WorksFilter::FromPostedDate(_) => "from-posted-date",
            WorksFilter::UntilPostedDate(_) => "until-posted-date",
            WorksFilter::FromAcceptedDate(_) => "from-accepted-date",
            WorksFilter::UntilAcceptedDate(_) => "until-accepted-date",
            WorksFilter::HasLicense => "has-license",
            WorksFilter::LicenseUrl(_) => "license.url",
            WorksFilter::LicenseVersion(_) => "license.version",
            WorksFilter::LicenseDelay(_) => "license.delay",
            WorksFilter::HasFullText => "has-full-text",
            WorksFilter::FullTextVersion(_) => "full-text.version",
            WorksFilter::FullTextType(_) => "full-text.type",
            WorksFilter::FullTextApplication(_) => "full-text.application",
            WorksFilter::HasReferences => "has-references",
            WorksFilter::ReferenceVisibility(_) => "reference-visibility",
            WorksFilter::HasArchive => "has-archive",
            WorksFilter::Archive(_) => "archive",
            WorksFilter::HasOrcid => "has-orcid",
            WorksFilter::HasAuthenticatedOrcid => "has-authenticated-orcid",
            WorksFilter::Orcid(_) => "orcid",
            WorksFilter::Issn(_) => "issn",
            WorksFilter::Isbn(_) => "isbn",
            WorksFilter::Type(_) => "type",
            WorksFilter::Directory(_) => "directory",
            WorksFilter::Doi(_) => "doi",
            WorksFilter::Updates(_) => "updates",
            WorksFilter::IsUpdate => "is-update",
            WorksFilter::HasUpdatePolicy => "has-update-policy",
            WorksFilter::ContainerTitle(_) => "container-title",
            WorksFilter::CategoryName(_) => "category-name",
            WorksFilter::TypeName(_) => "type-name",
            WorksFilter::AwardNumber(_) => "award.number",
            WorksFilter::AwardFunder(_) => "award.funder",
            WorksFilter::HasAssertion => "has-assertion",
            WorksFilter::AssertionGroup(_) => "assertion-group",
            WorksFilter::Assertion(_) => "assertion",
            WorksFilter::HasAffiliation => "has-affiliation",
            WorksFilter::AlternativeId => "alternative-id",
            WorksFilter::ArticleNumber => "article-number",
            WorksFilter::HasAbstract => "has-abstract",
            WorksFilter::HasClinicalTrialNumber => "has-clinical-trial-number",
            WorksFilter::ContentDomain(_) => "content-domain",
            WorksFilter::HasContentDomain => "has-content-domain",
            WorksFilter::HasDomainRestriction => "has-domain-restriction",
            WorksFilter::HasRelation => "has-relation",
            WorksFilter::RelationType => "relation.type",
            WorksFilter::RelationObject => "relation.object",
            WorksFilter::RelationObjectType(_) => "relation.object-type",
        }
    }
}

impl ParamFragment for WorksFilter {
    fn key(&self) -> Cow<'_, str> {
        Cow::Borrowed(self.name())
    }

    fn value(&self) -> Option<Cow<'_, str>> {
        match self {
            WorksFilter::Funder(s)
            | WorksFilter::Location(s)
            | WorksFilter::Prefix(s)
            | WorksFilter::Member(s)
            | WorksFilter::LicenseUrl(s)
            | WorksFilter::LicenseVersion(s)
            | WorksFilter::FullTextVersion(s)
            | WorksFilter::FullTextType(s)
            | WorksFilter::FullTextApplication(s)
            | WorksFilter::Archive(s)
            | WorksFilter::Orcid(s)
            | WorksFilter::Issn(s)
            | WorksFilter::Isbn(s)
            | WorksFilter::Directory(s)
            | WorksFilter::Doi(s)
            | WorksFilter::Updates(s)
            | WorksFilter::ContainerTitle(s)
            | WorksFilter::CategoryName(s)
            | WorksFilter::AwardNumber(s)
            | WorksFilter::TypeName(s)
            | WorksFilter::AwardFunder(s)
            | WorksFilter::AssertionGroup(s)
            | WorksFilter::Assertion(s)
            | WorksFilter::ContentDomain(s)
            | WorksFilter::RelationObjectType(s) => Some(Cow::Borrowed(s.as_str())),
            WorksFilter::ReferenceVisibility(vis) => Some(Cow::Borrowed(vis.as_str())),
            WorksFilter::FromIndexDate(d)
            | WorksFilter::UntilIndexDate(d)
            | WorksFilter::FromDepositDate(d)
            | WorksFilter::UntilDepositDate(d)
            | WorksFilter::FromUpdateDate(d)
            | WorksFilter::UntilUpdateDate(d)
            | WorksFilter::FromCreatedDate(d)
            | WorksFilter::UntilCreatedDate(d)
            | WorksFilter::FromPubDate(d)
            | WorksFilter::UntilPubDate(d)
            | WorksFilter::FromOnlinePubDate(d)
            | WorksFilter::UntilOnlinePubDate(d)
            | WorksFilter::FromPrintPubDate(d)
            | WorksFilter::UntilPrintPubDate(d)
            | WorksFilter::FromPostedDate(d)
            | WorksFilter::UntilPostedDate(d)
            | WorksFilter::FromAcceptedDate(d)
            | WorksFilter::UntilAcceptedDate(d) => {
                Some(Cow::Owned(d.format("%Y-%m-%d").to_string()))
            }
            WorksFilter::Type(t) => Some(Cow::Borrowed(t.id())),
            _ => Some(Cow::Borrowed("true")),
        }
    }
}

impl Filter for WorksFilter {}

/// Field queries are available on the `/works` route and allow for queries that match only particular fields of metadata.
#[derive(Debug, Clone)]
pub struct FieldQuery {
    /// match any only particular fields of metadata.
    pub name: String,
    /// the value of the query
    pub value: String,
}

impl FieldQuery {
    /// creates a new `Field` query for `title` and `subtitle`
    pub fn title(title: &str) -> Self {
        Self {
            name: "query.title".to_string(),
            value: title.to_string(),
        }
    }

    /// creates a new `Field` query for `container-title` aka `publication.name`
    pub fn container_title(container_title: &str) -> Self {
        Self {
            name: "query.container-title".to_string(),
            value: container_title.to_string(),
        }
    }
    /// creates a new `Field` query author given and family names
    pub fn author(author: &str) -> Self {
        Self {
            name: "query.author".to_string(),
            value: author.to_string(),
        }
    }
    /// creates a new `Field` query for editor given and family names
    pub fn editor(editor: &str) -> Self {
        Self {
            name: "query.editor".to_string(),
            value: editor.to_string(),
        }
    }
    /// creates a new `Field` query for chair given and family names
    pub fn chair(chair: &str) -> Self {
        Self {
            name: "query.chair".to_string(),
            value: chair.to_string(),
        }
    }
    /// creates a new `Field` query for translator given and family names
    pub fn translator(translator: &str) -> Self {
        Self {
            name: "query.translator".to_string(),
            value: translator.to_string(),
        }
    }
    /// creates a new `Field` query for author, editor, chair and translator given and family names
    pub fn contributor(contributor: &str) -> Self {
        Self {
            name: "query.contributor".to_string(),
            value: contributor.to_string(),
        }
    }
    /// creates a new `Field` query for bibliographic information, useful for citation look up.
    /// Includes titles, authors, ISSNs and publication years
    pub fn bibliographic(bibliographic: &str) -> Self {
        Self {
            name: "query.bibliographic".to_string(),
            value: bibliographic.to_string(),
        }
    }
    /// creates a new `Field` query for contributor affiliations
    pub fn affiliation(affiliation: &str) -> Self {
        Self {
            name: "query.affiliation".to_string(),
            value: affiliation.to_string(),
        }
    }
}

impl CrossrefQueryParam for FieldQuery {
    fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)> {
        vec![(
            Cow::Borrowed(&self.name),
            Cow::Owned(format_query(&self.value)),
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

    /// add a new free form query
    pub fn sample(mut self, len: usize) -> Self {
        self.sample = Some(len);
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

    /// set sort option to the query
    pub fn sort(mut self, sort: Sort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// set order option to query
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
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
    /// set result control option to query
    pub fn result_control(mut self, result_control: WorkResultControl) -> Self {
        self.result_control = Some(result_control);
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
