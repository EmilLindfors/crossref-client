use crate::error::{Error, Result};
use crate::query::works::{WorksCombiner, WorksIdentQuery};
use crate::query::{Component, CrossrefQuery, CrossrefRoute, ResourceComponent};

#[derive(Debug, Clone)]
pub struct JournalResultControl {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub sample: Option<bool>,
    pub sort: Option<String>,
}

impl JournalResultControl {
    pub fn new(limit: Option<usize>, offset: Option<usize>, sample: Option<bool>, sort: Option<String>) -> Self {
        JournalResultControl {
            limit,
            offset,
            sample,
            sort,
        }
    }

    pub fn new_from_limit(limit: usize) -> Self {
        JournalResultControl {
            limit: Some(limit),
            offset: None,
            sample: None,
            sort: None,
        }
    }

    pub fn new_from_offset(offset: usize) -> Self {
        JournalResultControl {
            limit: None,
            offset: Some(offset),
            sample: None,
            sort: None,
        }
    }

    pub fn new_from_sample(sample: bool) -> Self {
        JournalResultControl {
            limit: None,
            offset: None,
            sample: Some(sample),
            sort: None,
        }
    }

    pub fn new_from_sort(sort: String) -> Self {
        JournalResultControl {
            limit: None,
            offset: None,
            sample: None,
            sort: Some(sort),
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn sample(mut self, sample: bool) -> Self {
        self.sample = Some(sample);
        self
    }

    pub fn sort(mut self, sort: String) -> Self {
        self.sort = Some(sort);
        self
    }

}

impl std::fmt::Display for JournalResultControl {
    /// Renders the result control as a query string fragment, e.g. `rows=10&offset=20`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut params = Vec::new();
        if let Some(l) = self.limit {
            params.push(format!("rows={}", l));
        }
        if let Some(o) = self.offset {
            params.push(format!("offset={}", o));
        }
        if let Some(s) = self.sample {
            params.push(format!("sample={}", s));
        }
        if let Some(s) = &self.sort {
            params.push(format!("sort={}", s));
        }
        f.write_str(&params.join("&"))
    }
}

impl TryFrom<String> for JournalResultControl {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        
    
        let mut limit = None;
        let mut offset = None;
        let mut sample = None;
        let mut sort = None;

        let parts = value.split('&').collect::<Vec<&str>>();
        for part in parts {
            let kv = part.split('=').collect::<Vec<&str>>();
            if kv.len() != 2 {
                return Err(Error::InvalidResultControl {
                    error: value.clone(),
                });
            }
            match kv[0] {
                "rows" => {
                    limit = Some(kv[1].parse().map_err(|e| Error::InvalidResultControl {
                        error: format!("Invalid limit: {}", e),
                    })?);
                }
                "offset" => {
                    offset = Some(kv[1].parse().map_err(|e| Error::InvalidResultControl {
                        error: format!("Invalid offset: {}", e),
                    })?);
                }
                "sample" => {
                    sample = Some(kv[1].parse().map_err(|e| Error::InvalidResultControl {
                        error: format!("Invalid sample: {}", e),
                    })?);
                }
                "sort" => {
                    sort = Some(kv[1].to_string());
                }
                _ => return Err(Error::InvalidResultControl { error: value.clone() }),
            }
        }

        Ok(JournalResultControl {
            limit,
            offset,
            sample,
            sort,
        })
    }
}






/// constructs the request payload for the `/journals` route
#[derive(Debug, Clone)]
pub enum Journals {
    /// target a specific journal at `/journals/{id}`
    Identifier(String),
    /// target a `Work` for a specific funder at `/journals/{id}/works?query..`
    Works(WorksIdentQuery),
    /// free form query for `/journals?query...`
    Query(String, Option<JournalResultControl>),
}

impl CrossrefRoute for Journals {
    fn route(&self) -> Result<String> {
        match self {
            Journals::Identifier(s) => Ok(format!("{}/{}", Component::Journals.route()?, s)),
            Journals::Query(query, result_control) => {
                let q = query.split(' ').collect::<Vec<&str>>().join("+");
                if let Some(rc) = result_control {
                    if query.is_empty() {
                        Ok(format!("{}/?{}", Component::Journals.route()?, rc))
                    } else {
                        Ok(format!("{}/?query={}&{}", Component::Journals.route()?, q, rc))
                    }
                } else {
                    Ok(format!("{}/?query={}", Component::Journals.route()?, q))
                }
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
