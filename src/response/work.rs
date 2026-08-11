// see https://github.com/Crossref/rest-api-doc/blob/master/api_format.md

use std::collections::HashMap;

use crate::error::Error;
use crate::response::{FacetMap, QueryResponse};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::FacetItem;

/// A hashmap containing relation name, `Relation` pairs.
/// [crossref rest-api-doc](https://github.com/CrossRef/rest-api-doc/blob/master/api_format.md#relations)
/// However it seems, that the value of the relation name can also be an array.
/// Therefor the `serde_json::Value` type is used instead to prevent an invalid length error
pub type Relations = std::collections::HashMap<String, Value>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct WorkList {
    pub facets: FacetMap,
    /// the number of items that match the response
    pub total_results: usize,
    /// crossref responses for large number of items are divided in pages, number of elements to expect in `items`
    pub items_per_page: Option<usize>,
    /// if a query was set in the request, this will also be part in the response
    pub query: Option<QueryResponse>,
    /// all work items that are returned
    pub items: Vec<Work>,
    /// deep page through `/works` result sets
    pub next_cursor: Option<String>,
}

impl TryFrom<serde_json::Value> for WorkList {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
      match value {
        Value::Object(map) => {
          let facets: HashMap<String, FacetItem> = map
          .get("facets")
          .ok_or(Error::MissingField {
              msg: "facets".to_string(),
          })?
            .as_object()
            .ok_or(Error::InvalidTypeName {
                name: "facets".to_string(),
            })?
            .iter()
            .map(|(k, v)| (k.to_string(), FacetItem::try_from(v.clone()).unwrap()))
            .collect();

  

              
                
     

          let total_results = map
              .get("total-results")
              .ok_or(Error::MissingField {
                  msg: "total-results".to_string(),
              })?
                .as_u64()
                .map(|v| v as usize)
                .ok_or(Error::InvalidTypeName {
                    name: "total-results".to_string(),
                })?;


          let items_per_page = map
              .get("items-per-page")
              .and_then(|v| v.as_u64())
              .map(|v| v as usize);

          let query = map
              .get("query")
              .and_then(|v| v.as_object())
              .map(|v| QueryResponse::try_from(Value::Object(v.clone())).unwrap());

          let items: Vec<Work> = map
              .get("items")
                .ok_or(Error::MissingField {
                    msg: "items".to_string(),
                })?
                .as_array() 
                .ok_or(Error::InvalidTypeName {
                    name: "items".to_string(),
                })?
                .iter()
                .map(|v| Work::try_from(v.clone()).unwrap())
                .collect();

          let next_cursor = map
              .get("next-cursor")
              .and_then(|v| v.as_str())
              .map(|v| v.to_string());

          Ok(WorkList {
              facets,
              total_results,
              items_per_page,
              query,
              items,
              next_cursor,
          })
        }
        _ => Err(Error::InvalidMessageType {
            name: value.to_string(),
        })?,
    }
    }
}

/// the main return type of the crossref api
/// represents a publication
/// based on the [crossref rest-api-doc](https://github.com/CrossRef/rest-api-doc/blob/master/api_format.md#work)
/// with minor adjustments
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct Work {
    /// Name of work's publisher
    pub publisher: String,
    /// Work titles, including translated titles
    pub title: Vec<String>,
    /// Work titles in the work's original publication language
    pub original_title: Option<Vec<String>>,
    /// the language of this work
    pub language: Option<String>,
    /// Abstract as a JSON string or a JATS XML snippet encoded into a JSON string
    pub short_title: Option<Vec<String>>,
    /// Abstract as a JSON string or a JATS XML snippet encoded into a JSON string
    #[serde(rename = "abstract")]
    pub abstract_: Option<String>,
    /// Count of outbound references deposited with Crossref
    pub references_count: Option<i32>,
    /// Count of inbound references deposited with Crossref
    pub is_referenced_by_count: Option<i32>,
    /// Currently always `Crossref`
    pub source: Option<String>,
    pub journal_issue: Option<Issue>,
    /// DOI prefix identifier of the form `http://id.crossref.org/prefix/DOI_PREFIX`
    pub prefix: Option<String>,
    /// DOI of the work
    #[serde(rename = "DOI")]
    pub doi: String,
    /// URL form of the work's DOI
    #[serde(rename = "URL")]
    pub url: Option<String>,
    /// Member identifier of the form `http://id.crossref.org/member/MEMBER_ID`
    pub member: String,
    /// Enumeration, one of the type ids from `https://api.crossref.org/v1/types`
    #[serde(rename = "type")]
    pub type_: String,
    /// the day this work entry was created
    pub created: Date,
    /// Date on which the DOI was first registered
    pub date: Option<Date>,
    /// Date on which the work metadata was most recently updated
    pub deposited: Option<Date>,
    /// the score of the publication if any
    /// not included in the crossrif api spec
    pub score: Option<f32>,
    /// Date on which the work metadata was most recently indexed.
    /// Re-indexing does not imply a metadata change, see `deposited` for the most recent metadata change date
    pub indexed: Date,
    /// Earliest of `published-print` and `published-online`
    pub issued: Option<PartialDate>,
    /// ate on which posted content was made available online
    pub posted: Option<PartialDate>,
    /// Date on which a work was accepted, after being submitted, during a submission process
    pub accepted: Option<PartialDate>,
    /// Work subtitles, including original language and translated
    pub subtitle: Option<Vec<String>>,
    /// Full titles of the containing work (usually a book or journal)
    pub container_title: Option<Vec<String>>,
    /// Abbreviated titles of the containing work
    pub short_container_title: Option<Vec<String>>,
    /// Group title for posted content
    pub group_title: Option<String>,
    /// Issue number of an article's journal
    pub issue: Option<String>,
    /// Volume number of an article's journal
    pub volume: Option<String>,
    /// Pages numbers of an article within its journal
    pub page: Option<String>,
    /// the number of the corresponding article
    pub article_number: Option<String>,
    /// Date on which the work was published in print
    pub published_print: Option<PartialDate>,
    /// Date on which the work was published online
    pub published_online: Option<PartialDate>,
    /// Subject category names, a controlled vocabulary from Sci-Val.
    /// Available for most journal articles
    pub subject: Option<Vec<String>>,
    #[serde(rename = "ISSN")]
    pub issn: Option<Vec<String>>,
    /// List of ISSNs with ISSN type information
    pub issn_type: Option<Vec<ISSN>>,
    #[serde(rename = "ISBN")]
    pub isbn: Option<Vec<String>>,
    pub archive: Option<Vec<String>>,
    pub license: Option<Vec<License>>,
    pub funder: Option<Vec<FundingBody>>,
    pub assertion: Option<Vec<Assertion>>,
    pub author: Option<Vec<Contributor>>,
    pub editor: Option<Vec<Contributor>>,
    pub chair: Option<Vec<Contributor>>,
    pub translator: Option<Vec<Contributor>>,
    pub update_to: Option<Vec<Update>>,
    /// Link to an update policy covering Crossmark updates for this work
    pub update_policy: Option<String>,
    /// URLs to full-text locations
    pub link: Option<Vec<ResourceLink>>,
    pub clinical_trial_number: Option<Vec<ClinicalTrialNumber>>,
    /// Other identifiers for the work provided by the depositing member
    pub alternative_id: Option<Vec<String>>,
    /// List of references made by the work
    pub reference: Option<Vec<Reference>>,
    /// Information on domains that support Crossmark for this work
    pub content_domain: Option<ContentDomain>,
    /// Relations to other works
    pub relation: Option<Relations>,
    /// Peer review metadata
    pub review: Option<Relations>,
}

impl TryFrom<serde_json::Value> for Work {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let publisher = map
                    .get("publisher")
                    .ok_or(Error::MissingField {
                        msg: "publisher".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "publisher".to_string(),
                    })?
                    .to_string();

                let title: Vec<String> = map
                    .get("title")
                    .ok_or(Error::MissingField {
                        msg: "title".to_string(),
                    })?
                    .as_array()
                    .ok_or(Error::InvalidTypeName {
                        name: "title".to_string(),
                    })?
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect();

                let original_title: Option<Vec<String>> = map
                    .get("original-title")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let language = map
                    .get("language")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let short_title: Option<Vec<String>> = map
                    .get("short-title")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let abstract_: Option<String> = map
                    .get("abstract")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let references_count = map
                    .get("references-count")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                let is_referenced_by_count = map
                    .get("is-referenced-by-count")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);

                let source = map
                    .get("source")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let journal_issue = map
                    .get("journal-issue")
                    .and_then(|v| v.as_object())
                    .map(|v| Issue::try_from(Value::Object(v.clone())).unwrap());

                let prefix = map
                    .get("prefix")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let doi = map
                    .get("DOI")
                    .ok_or(Error::MissingField {
                        msg: "DOI".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "DOI".to_string(),
                    })?
                    .to_string();

                let url = map
                    .get("URL")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let member = map
                    .get("member")
                    .ok_or(Error::MissingField {
                        msg: "member".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "member".to_string(),
                    })?
                    .to_string();

                let type_ = map
                    .get("type")
                    .ok_or(Error::MissingField {
                        msg: "type".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "type".to_string(),
                    })?
                    .to_string();

                let created = map
                    .get("created")
                    .ok_or(
                        Error::MissingField {
                            msg: "created".to_string(),
                        },
                    )?
                    .as_object()
                    .ok_or(Error::InvalidTypeName {
                        name: "created".to_string(),
                    })
                    .map(|v| Date::try_from(Value::Object(v.clone())).unwrap())?;

                let date = map
                    .get("date")
                    .and_then(|v| v.as_object())
                    .map(|v| Date::try_from(Value::Object(v.clone())).unwrap());

                let deposited = map
                    .get("deposited")
                    .and_then(|v| v.as_object())
                    .map(|v| Date::try_from(Value::Object(v.clone())).unwrap());

                let score = map.get("score").and_then(|v| v.as_f64()).map(|v| v as f32);

                let indexed = map
                    .get("indexed")
                    .ok_or(Error::MissingField {
                        msg: "indexed".to_string(),
                    })?
                    .as_object()
                    .ok_or(Error::InvalidTypeName {
                        name: "indexed".to_string(),
                    })
                    .map(|v| Date::try_from(Value::Object(v.clone())).unwrap())?;

                let issued = map
                    .get("issued")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let posted = map
                    .get("posted")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let accepted = map
                    .get("accepted")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let subtitle = map
                    .get("subtitle")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let container_title = map
                    .get("container-title")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let short_container_title = map
                    .get("short-container-title")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let group_title = map
                    .get("group-title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let issue = map
                    .get("issue")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let volume = map
                    .get("volume")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let page = map
                    .get("page")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let article_number = map
                    .get("article-number")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let published_print = map
                    .get("published-print")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let published_online = map
                    .get("published-online")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let subject = map
                    .get("subject")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let issn = map
                    .get("ISSN")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let issn_type = map.get("issn-type").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| ISSN::try_from(v.clone()).unwrap())
                        .collect()
                });

                let isbn = map
                    .get("ISBN")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let archive = map
                    .get("archive")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let license = map.get("license").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| License::try_from(v.clone()).unwrap())
                        .collect()
                });

                let funder = map.get("funder").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| FundingBody::try_from(v.clone()).unwrap())
                        .collect()
                });

                let assertion = map.get("assertion").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Assertion::try_from(v.clone()).unwrap())
                        .collect()
                });

                let author = map.get("author").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Contributor::try_from(v.clone()).unwrap())
                        .collect()
                });

                let editor = map.get("editor").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Contributor::try_from(v.clone()).unwrap())
                        .collect()
                });

                let chair = map.get("chair").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Contributor::try_from(v.clone()).unwrap())
                        .collect()
                });

                let translator = map.get("translator").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Contributor::try_from(v.clone()).unwrap())
                        .collect()
                });

                let update_to = map.get("update-to").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Update::try_from(v.clone()).unwrap())
                        .collect()
                });

                let update_policy = map
                    .get("update-policy")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let link = map.get("link").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| ResourceLink::try_from(v.clone()).unwrap())
                        .collect()
                });

                let clinical_trial_number = map
                    .get("clinical-trial-number")
                    .and_then(|v| v.as_array())
                    .map(|v| {
                        v.iter()
                            .map(|v| ClinicalTrialNumber::try_from(v.clone()).unwrap())
                            .collect()
                    });

                let alternative_id = map
                    .get("alternative-id")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let reference = map.get("reference").and_then(|v| v.as_array()).map(|v| {
                    v.iter()
                        .map(|v| Reference::try_from(v.clone()).unwrap())
                        .collect()
                });

                let content_domain = map
                    .get("content-domain")
                    .and_then(|v| v.as_object())
                    .map(|v| ContentDomain::try_from(Value::Object(v.clone())).unwrap());

                let relation = map.get("relation").and_then(|v| v.as_object()).map(|v| {
                    let mut map = HashMap::new();
                    for (k, v) in v.iter() {
                        map.insert(k.to_string(), v.clone());
                    }
                    map
                });

                let review = map.get("review").and_then(|v| v.as_object()).map(|v| {
                    let mut map = HashMap::new();
                    for (k, v) in v.iter() {
                        map.insert(k.to_string(), v.clone());
                    }
                    map
                });

                Ok(Work {
                    publisher,
                    title,
                    original_title,
                    language,
                    short_title,
                    abstract_,
                    references_count,
                    is_referenced_by_count,
                    source,
                    journal_issue,
                    prefix,
                    doi,
                    url,
                    member,
                    type_,
                    created,
                    date,
                    deposited,
                    score,
                    indexed,
                    issued,
                    posted,
                    accepted,
                    subtitle,
                    container_title,
                    short_container_title,
                    group_title,
                    issue,
                    volume,
                    page,
                    article_number,
                    published_print,
                    published_online,
                    subject,
                    issn,
                    issn_type,
                    isbn,
                    archive,
                    license,
                    funder,
                    assertion,
                    author,
                    editor,
                    chair,
                    translator,
                    update_to,
                    update_policy,
                    link,
                    clinical_trial_number,
                    alternative_id,
                    reference,
                    content_domain,
                    relation,
                    review,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

impl Work {
    /// A short citation key for this work, in the common `AuthorYear` style.
    ///
    /// Built from the family names of the authors and the issued year:
    /// `Smith2019` for a single author, `Smith&Jones2019` for two, and
    /// `SmithEtAl2019` for three or more.
    ///
    /// Returns [`None`] if the work carries no author with a family name, or
    /// no issued date to take a year from.
    pub fn citekey(&self) -> Option<String> {
        let authors = self.author.as_deref()?;
        let family = |c: &Contributor| c.family.clone().unwrap_or_default();

        let names = match authors {
            [] => return None,
            [single] => single.family.clone()?,
            [first, second] => format!("{}&{}", family(first), family(second)),
            [first, ..] => format!("{}EtAl", family(first)),
        };

        let year = self.issued.as_ref()?.as_date_field()?.year();
        Some(format!("{}{}", names, year))
    }
}

/// Helper struct to represent dates in the cross ref api as nested arrays of numbers
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DateParts(pub Vec<Vec<Option<u32>>>);

impl DateParts {
    /// converts the nested array of numbers into the corresponding [DateField]
    /// standalone years are allowed.
    /// if an array is empty, [None] will be returned
    pub fn as_date(&self) -> Option<DateField> {
        /// converts an array of numbers into chrono [NaiveDate] if it contains at least a single value
        fn naive(v: &[Option<u32>]) -> Option<NaiveDate> {
            match v.len() {
                0 => None,
                1 => NaiveDate::from_ymd_opt(v[0]? as i32, 0, 0),
                2 => NaiveDate::from_ymd_opt(v[0]? as i32, v[1]?, 0),
                3 => NaiveDate::from_ymd_opt(v[0]? as i32, v[1]?, v[2]?),
                _ => None,
            }
        }

        match self.0.len() {
            0 => None,
            1 => Some(DateField::Single(naive(&self.0[0])?)),
            2 => Some(DateField::Range {
                from: naive(&self.0[0])?,
                to: naive(&self.0[1])?,
            }),
            _ => Some(DateField::Multi(
                self.0
                    .iter()
                    .map(|x| naive(x))
                    .collect::<Option<Vec<_>>>()?,
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct FundingBody {
    /// Funding body primary name
    pub name: String,
    /// Optional [Open Funder Registry](http://www.crossref.org/fundingdata/registry.html) DOI uniquely identifing the funding body
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    /// Award number(s) for awards given by the funding body
    pub award: Option<Vec<String>>,
    /// Either `crossref` or `publisher`
    #[serde(rename = "doi-asserted-by")]
    pub doi_asserted_by: Option<String>,
}

impl TryFrom<serde_json::Value> for FundingBody {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let name = map
                    .get("name")
                    .ok_or(Error::MissingField {
                        msg: "name".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "name".to_string(),
                    })?
                    .to_string();

                let doi = map
                    .get("DOI")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let award = map
                    .get("award")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let doi_asserted_by = map
                    .get("doi-asserted-by")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(FundingBody {
                    name,
                    doi,
                    award,
                    doi_asserted_by,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct ClinicalTrialNumber {
    /// Identifier of the clinical trial
    #[serde(rename = "clinical-trial-number")]
    pub clinical_trial_number: String,
    /// DOI of the clinical trial regsitry that assigned the trial number
    pub registry: String,
    /// One of `preResults`, `results` or `postResults`
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl TryFrom<serde_json::Value> for ClinicalTrialNumber {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let clinical_trial_number = map
                    .get("clinical-trial-number")
                    .ok_or(Error::MissingField {
                        msg: "clinical-trial-number".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "clinical-trial-number".to_string(),
                    })?
                    .to_string();

                let registry = map
                    .get("registry")
                    .ok_or(Error::MissingField {
                        msg: "registry".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "registry".to_string(),
                    })?
                    .to_string();

                let type_ = map
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(ClinicalTrialNumber {
                    clinical_trial_number,
                    registry,
                    type_,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct Contributor {
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub family: Option<String>,
    pub given: Option<String>,
    pub name: Option<String>,
    /// URL-form of an [ORCID](http://orcid.org) identifier
    #[serde(rename = "ORCID")]
    pub orcid: Option<String>,
    /// If true, record owner asserts that the ORCID user completed ORCID OAuth authentication
    #[serde(rename = "authenticated-orcid")]
    pub authenticated_orcid: Option<bool>,
    pub affiliation: Vec<Affiliation>,
    pub sequence: String
}

impl TryFrom<serde_json::Value> for Contributor {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {

                let prefix = map
                    .get("prefix")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let suffix = map
                    .get("suffix")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());



                let family = map
                    .get("family")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let given = map
                    .get("given")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let name = map
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let orcid = map
                    .get("ORCID")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let authenticated_orcid = map
                    .get("authenticated-orcid")
                    .and_then(|v| v.as_bool());

                let affiliation = map.get("affiliation").and_then(|v| v.as_array()).ok_or(
                    Error::MissingField {
                        msg: "affiliation".to_string(),
                    },
                ).map(|v| {
                    v.iter()
                        .map(|v| Affiliation::try_from(v.clone()).unwrap())
                        .collect()
                })?;

                let sequence = map
                    .get("sequence")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "sequence".to_string(),
                    })?;

                Ok(Contributor {
                    prefix,
                    suffix,
                    family,
                    given,
                    name,
                    orcid,
                    authenticated_orcid,
                    affiliation,
                    sequence
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct Affiliation {
    /// the affiliation's name
    pub name: String,
}

impl TryFrom<serde_json::Value> for Affiliation {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let name = map
                    .get("name")
                    .ok_or(Error::MissingField {
                        msg: "name".to_string(),
                    })?
                    .as_str()
                    .ok_or(Error::InvalidTypeName {
                        name: "name".to_string(),
                    })?
                    .to_string();

                Ok(Affiliation { name })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

/// represents full date information for an item
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Date {
    /// Contains an ordered array of year, month, day of month.
    /// Only year is required. Note that the field contains a nested array,
    /// e.g. [ [ 2006, 5, 19 ] ] to conform to citeproc JSON dates
    pub date_parts: DateParts,
    /// Seconds since UNIX epoch
    pub timestamp: usize,
    /// ISO 8601 date time
    pub date_time: String,
}

impl TryFrom<serde_json::Value> for Date {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let date_parts = map.get("date-parts").and_then(|v| v.as_array()).map(|v| {
                    // build a vector of vectors of three optional u32 values
                    let date_parts = v
                        .iter()
                        .map(|v| {
                            v.as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().map(|v| v as u32))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();

                    DateParts(date_parts)
                });

                let timestamp = map
                    .get("timestamp")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .ok_or(Error::MissingField {
                        msg: "timestamp".to_string(),
                    })?;

                let date_time = map
                    .get("date-time")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "date-time".to_string(),
                    })?;

                Ok(Date {
                    date_parts: date_parts.unwrap(),
                    timestamp,
                    date_time,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

impl Date {
    /// converts the nested array of numbers into the correct representation of chrono [NaiveDate]
    pub fn as_date_field(&self) -> Option<DateField> {
        self.date_parts.as_date()
    }

}

impl std::fmt::Display for Date {
    /// Renders the date as `%Y-%m-%d`; ranges as `from-to` and multi dates as a
    /// comma separated list. An unparsable date renders as the empty string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.as_date_field() {
            Some(DateField::Single(date)) => write!(f, "{}", date.format("%Y-%m-%d")),
            Some(DateField::Range { from, to }) => write!(
                f,
                "{}-{}",
                from.format("%Y-%m-%d"),
                to.format("%Y-%m-%d")
            ),
            Some(DateField::Multi(dates)) => {
                let joined = dates
                    .iter()
                    .map(|date| date.format("%Y-%m-%d").to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                f.write_str(&joined)
            }
            None => Ok(()),
        }
    }
}

/// represents an incomplete date only consisting of year or year and month
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct PartialDate {
    /// Contains an ordered array of year, month, day of month.
    /// Only year is required
    /// e.g. `[ [`2006`] ]` to conform to citeproc JSON dates
    #[serde(rename = "date-parts")]
    pub date_parts: DateParts,
}

impl TryFrom<serde_json::Value> for PartialDate {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let date_parts = map.get("date-parts").and_then(|v| v.as_array()).map(|v| {
                    // build a vector of vectors of three optional u32 values
                    let date_parts = v
                        .iter()
                        .map(|v| {
                            v.as_array()
                                .unwrap()
                                .iter()
                                .map(|v| v.as_u64().map(|v| v as u32))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();

                    DateParts(date_parts)
                });

                Ok(PartialDate {
                    date_parts: date_parts.unwrap(),
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

impl PartialDate {
    /// converts the nested array of numbers into the correct representation of chrono [NaiveDate]
    pub fn as_date_field(&self) -> Option<DateField> {
        self.date_parts.as_date()
    }
}

/// Helper struct to capture all possible occurrences of dates in the crossref api, a nested Vec of numbers
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum DateField {
    /// only a single date vector
    Single(NaiveDate),
    /// two date vectors represent a range
    Range {
        /// start date of the range
        from: NaiveDate,
        /// end date of the range
        to: NaiveDate,
    },
    /// more than two date vectors are present
    Multi(Vec<NaiveDate>),
}

impl DateField {
    pub fn year(&self) -> i32 {
        match self {
            DateField::Single(date) => date.year(),
            DateField::Range { from, to: _ } => from.year(),
            DateField::Multi(dates) => dates[0].year(),
        }
    }
}

/// metadata about when the `Work` entry was updated
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Update {
    /// Date on which the update was published
    pub updated: PartialDate,
    /// DOI of the updated work
    #[serde(rename = "DOI")]
    pub doi: String,
    /// The type of update, for example retraction or correction
    #[serde(rename = "type")]
    pub type_: String,
    /// A display-friendly label for the update type
    pub label: Option<String>,
}

impl TryFrom<serde_json::Value> for Update {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let updated = map
                    .get("updated")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let doi = map
                    .get("DOI")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "DOI".to_string(),
                    })?;

                let type_ = map
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "type".to_string(),
                    })?;

                let label = map
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(Update {
                    updated: updated.unwrap(),
                    doi,
                    type_,
                    label,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct Assertion {
    pub name: String,
    pub value: Option<String>,
    #[serde(rename = "URL")]
    pub url: Option<String>,
    pub explanation: Option<String>,
    pub label: Option<String>,
    pub order: Option<i32>,
    pub group: Option<AssertionGroup>,
}

impl TryFrom<serde_json::Value> for Assertion {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let name = map
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "name".to_string(),
                    })?;

                let value = map
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let url = map
                    .get("URL")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let explanation = map
                    .get("explanation")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let label = map
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let order = map.get("order").and_then(|v| v.as_i64()).map(|v| v as i32);

                let group = map
                    .get("group")
                    .and_then(|v| v.as_object())
                    .map(|v| AssertionGroup::try_from(Value::Object(v.clone())).unwrap());

                Ok(Assertion {
                    name,
                    value,
                    url,
                    explanation,
                    label,
                    order,
                    group,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct Issue {
    /// Date on which the work was published in print
    pub published_print: Option<PartialDate>,
    /// Date on which the work was published online
    pub published_online: Option<PartialDate>,
    /// Issue number of an article's journal
    pub issue: Option<String>,
}

impl TryFrom<serde_json::Value> for Issue {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let published_print = map
                    .get("published-print")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let published_online = map
                    .get("published-online")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap());

                let issue = map
                    .get("issue")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(Issue {
                    published_print,
                    published_online,
                    issue,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct AssertionGroup {
    pub name: String,
    pub label: Option<String>,
}

impl TryFrom<serde_json::Value> for AssertionGroup {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let name = map
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "name".to_string(),
                    })?;

                let label = map
                    .get("label")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(AssertionGroup { name, label })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[allow(missing_docs)]
pub struct Agency {
    pub id: String,
    pub label: Option<String>,
}

/// how the `Work` is licensed
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct License {
    /// Either `vor` (version of record,) `am` (accepted manuscript) or `unspecified`
    pub content_version: String,
    /// Number of days between the publication date of the work and the start date of this license
    pub delay_in_days: i32,
    /// Date on which this license begins to take effect
    pub start: PartialDate,
    /// Link to a web page describing this license
    #[serde(rename = "URL")]
    pub url: String,
}

impl TryFrom<serde_json::Value> for License {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let content_version = map
                    .get("content-version")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "content-version".to_string(),
                    })?;

                let delay_in_days = map
                    .get("delay-in-days")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32)
                    .ok_or(Error::MissingField {
                        msg: "delay-in-days".to_string(),
                    })?;

                let start = map
                    .get("start")
                    .and_then(|v| v.as_object())
                    .map(|v| PartialDate::try_from(Value::Object(v.clone())).unwrap())
                    .ok_or(Error::MissingField {
                        msg: "start".to_string(),
                    })?;

                let url = map
                    .get("URL")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "URL".to_string(),
                    })?;

                Ok(License {
                    content_version,
                    delay_in_days,
                    start,
                    url,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

/// metadata about a related resource
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResourceLink {
    /// Either `text-mining`, `similarity-checking` or `unspecified`
    pub intended_application: String,
    /// Either `vor` (version of record,) `am` (accepted manuscript) or `unspecified`
    pub content_version: String,
    /// Direct link to a full-text download location
    #[serde(rename = "URL")]
    pub url: String,
    /// Content type (or MIME type) of the full-text object
    pub content_type: Option<String>,
}

impl TryFrom<serde_json::Value> for ResourceLink {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let intended_application = map
                    .get("intended-application")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "intended-application".to_string(),
                    })?;

                let content_version = map
                    .get("content-version")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "content-version".to_string(),
                    })?;

                let url = map
                    .get("URL")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "URL".to_string(),
                    })?;

                let content_type = map
                    .get("content-type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(ResourceLink {
                    intended_application,
                    content_version,
                    url,
                    content_type,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct Reference {
    pub key: String,
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    /// One of `crossref` or `publisher`
    pub doi_asserted_by: Option<String>,
    pub issue: Option<String>,
    pub first_page: Option<String>,
    pub volume: Option<String>,
    pub edition: Option<String>,
    pub component: Option<String>,
    pub standard_designator: Option<String>,
    pub standards_body: Option<String>,
    pub author: Option<String>,
    pub year: Option<String>,
    pub unstructured: Option<String>,
    pub journal_title: Option<String>,
    pub article_title: Option<String>,
    pub series_title: Option<String>,
    pub volume_title: Option<String>,
    #[serde(rename = "ISSN")]
    pub issn: Option<String>,
    /// One of `pissn` or `eissn`
    pub issn_type: Option<String>,
    #[serde(rename = "ISBN")]
    pub isbn: Option<String>,
    pub isbn_type: Option<String>,
}

impl TryFrom<serde_json::Value> for Reference {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let key = map
                    .get("key")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "key".to_string(),
                    })?;

                let doi = map
                    .get("DOI")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let doi_asserted_by = map
                    .get("doi-asserted-by")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let issue = map
                    .get("issue")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let first_page = map
                    .get("first-page")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let volume = map
                    .get("volume")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let edition = map
                    .get("edition")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let component = map
                    .get("component")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let standard_designator = map
                    .get("standard-designator")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let standards_body = map
                    .get("standards-body")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let author = map
                    .get("author")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let year = map
                    .get("year")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let unstructured = map
                    .get("unstructured")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let journal_title = map
                    .get("journal-title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let article_title = map
                    .get("article-title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let series_title = map
                    .get("series-title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let volume_title = map
                    .get("volume-title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let issn = map
                    .get("ISSN")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let issn_type = map
                    .get("issn-type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let isbn = map
                    .get("ISBN")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let isbn_type = map
                    .get("isbn-type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(Reference {
                    key,
                    doi,
                    doi_asserted_by,
                    issue,
                    first_page,
                    volume,
                    edition,
                    component,
                    standard_designator,
                    standards_body,
                    author,
                    year,
                    unstructured,
                    journal_title,
                    article_title,
                    series_title,
                    volume_title,
                    issn,
                    issn_type,
                    isbn,
                    isbn_type,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

/// ISSN info for the `Work`
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ISSN {
    /// identifier
    pub value: String,
    /// One of `eissn`, `pissn` or `lissn`
    #[serde(rename = "type")]
    pub type_: String,
}

impl TryFrom<serde_json::Value> for ISSN {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let value = map
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "value".to_string(),
                    })?;

                let type_ = map
                    .get("type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .ok_or(Error::MissingField {
                        msg: "type".to_string(),
                    })?;

                Ok(ISSN { value, type_ })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct ContentDomain {
    pub domain: Vec<String>,
    pub crossmark_restriction: bool,
}

impl TryFrom<serde_json::Value> for ContentDomain {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let domain = map
                    .get("domain")
                    .and_then(|v| v.as_array())
                    .map(|v| v.iter().map(|v| v.as_str().unwrap().to_string()).collect());

                let crossmark_restriction = map
                    .get("crossmark-restriction")
                    .and_then(|v| v.as_bool())
                    .ok_or(Error::MissingField {
                        msg: "crossmark-restriction".to_string(),
                    })?;

                Ok(ContentDomain {
                    domain: domain.unwrap(),
                    crossmark_restriction,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct Relation {
    pub id_type: Option<String>,
    pub id: Option<String>,
    pub asserted_by: Option<String>,
}

impl TryFrom<serde_json::Value> for Relation {
    type Error = Error;

    fn try_from(value: serde_json::Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let id_type = map
                    .get("id-type")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let id = map
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                let asserted_by = map
                    .get("asserted-by")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string());

                Ok(Relation {
                    id_type,
                    id,
                    asserted_by,
                })
            }
            _ => Err(Error::InvalidMessageType {
                name: value.to_string(),
            })?,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct Review {
    pub running_number: Option<String>,
    pub revision_round: Option<String>,
    /// One of `pre-publication` or `post-publication`
    pub stage: Option<String>,
    /// One of `major-revision` or `minor-revision` or `reject` or `reject-with-resubmit` or `accept`
    pub recommendation: Option<String>,
    /// One of `referee-report` or `editor-report` or `author-comment` or `community-comment` or `aggregate`
    #[serde(rename = "type")]
    pub type_: String,
    pub competing_interest_statement: Option<String>,
    pub language: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::*;
    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Demo {
        pub date_parts: DateParts,
    }
    #[test]
    fn date_parts_serde() {
        let demo = Demo {
            date_parts: DateParts(vec![vec![Some(2017), Some(10), Some(11)]]),
        };
        let expected = r##"{"date_parts":[[2017,10,11]]}"##;
        assert_eq!(expected, &to_string(&demo).unwrap());
        assert_eq!(demo, from_str::<Demo>(expected).unwrap());
    }

    #[test]
    fn serialize_work() {
        let work_str = r##"{
    "indexed": {
      "date-parts": [
        [
          2019,
          2,
          26
        ]
      ],
      "date-time": "2019-02-26T10:43:14Z",
      "timestamp": 1551177794515
    },
    "reference-count": 105,
    "publisher": "American Psychological Association (APA)",
    "issue": "1",
    "content-domain": {
      "domain": [],
      "crossmark-restriction": false
    },
    "short-container-title": [
      "American Psychologist"
    ],
    "DOI": "10.1037/0003-066x.59.1.29",
    "type": "journal-article",
    "created": {
      "date-parts": [
        [
          2004,
          1,
          21
        ]
      ],
      "date-time": "2004-01-21T14:31:19Z",
      "timestamp": 1074695479000
    },
    "page": "29-40",
    "source": "Crossref",
    "is-referenced-by-count": 84,
    "title": [
      "How the Mind Hurts and Heals the Body."
    ],
    "prefix": "10.1037",
    "volume": "59",
    "author": [
      {
        "given": "Oakley",
        "family": "Ray",
        "sequence": "first",
        "affiliation": []
      }
    ],
    "member": "15",
    "published-online": {
      "date-parts": [
        [
          2004
        ]
      ]
    },
    "container-title": [
      "American Psychologist"
    ],
    "original-title": [],
    "language": "en",
    "link": [
      {
        "URL": "http://psycnet.apa.org/journals/amp/59/1/29.pdf",
        "content-type": "unspecified",
        "content-version": "vor",
        "intended-application": "similarity-checking"
      }
    ],
    "deposited": {
      "date-parts": [
        [
          2018,
          4,
          8
        ]
      ],
      "date-time": "2018-04-08T18:56:17Z",
      "timestamp": 1523213777000
    },
    "score": 1,
    "subtitle": [],
    "short-title": [],
    "issued": {
      "date-parts": [
        [
          null
        ]
      ]
    },
    "references-count": 105,
    "journal-issue": {
      "published-online": {
        "date-parts": [
          [
            2004
          ]
        ]
      },
      "issue": "1"
    },
    "alternative-id": [
      "2004-10043-004",
      "14736318"
    ],
    "URL": "http://dx.doi.org/10.1037/0003-066x.59.1.29",
    "relation": {},
    "ISSN": [
      "1935-990X",
      "0003-066X"
    ],
    "issn-type": [
      {
        "value": "0003-066X",
        "type": "print"
      },
      {
        "value": "1935-990X",
        "type": "electronic"
      }
    ]
  }
"##;

        let _work: Work = from_str(work_str).unwrap();
    }
}
