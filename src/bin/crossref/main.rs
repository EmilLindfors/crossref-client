//! A command line front end to the crossref api.
//!
//! Each subcommand offers exactly the flags its route honours. `--sort`,
//! `--order` and `--sample` only exist under `works`, because `/funders`,
//! `/members`, `/journals` and `/licenses` all answer them with a `400`; the
//! flags used to be shared across every subcommand and silently dropped where
//! they could not be sent, so `crossref journals --sort score` returned results
//! in a different order than asked for and said nothing.
//!
//! The `--query-*` field queries live under `works` for the same reason: they
//! render as `query.title`, `query.author` and the rest, which only `/works`
//! reads.
//!
//! `--filter` goes further and is a different flag on each of the three routes
//! that take one, since the filters `/works`, `/funders` and `/members` narrow
//! by have nothing in common -- `location` belongs to `/funders` alone, and
//! `/works` answers it with a `400`. `/journals` and `/licenses` narrow by
//! nothing and so offer no such flag at all.
mod citekey;

use chrono::NaiveDate;
use citekey::{CiteKey, Mismatch};
use clap::{Args, Parser, Subcommand, ValueEnum};
use crossref_client::{
    AsyncIterator, CnFormat, Crossref, FieldQuery, FilterParseError, FundersFilter, FundersQuery,
    JournalsQuery, LicensesQuery, MembersFilter, MembersQuery, Order, ResultControl, Sort, Type,
    Work, WorkResultControl, WorksFilter, WorksQuery,
};
use serde::Serialize;

use std::{fs, io::Write, path::PathBuf, process::ExitCode, str::FromStr};

#[derive(Debug, Parser)]
#[command(
    name = "crossref",
    about = "Access the crossref API from the command line.",
    version
)]
struct App {
    #[command(flatten)]
    client: ClientOpts,
    #[command(flatten)]
    out: Out,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Query crossref works
    Works {
        /// The DOI of a single work. Omit to search by query terms.
        #[arg(short, long)]
        id: Option<String>,
        /// Enable deep paging. If a limit is set, then the limit takes priority.
        #[arg(short, long)]
        deep_page: bool,
        #[command(flatten)]
        opts: WorksOpts,
        #[command(subcommand)]
        combined: Option<Combined>,
    },
    /// Resolve bibtex citation keys, e.g. `@LindforsJakobsen2022`, to the works they cite
    ///
    /// Exits non-zero unless every key resolved to exactly one work, so a
    /// bibliography can be checked in a script. The report is written either
    /// way: a key that resolved to nothing, or to several works, is an answer
    /// rather than an error.
    Cite {
        /// The citation keys, with or without their leading `@`
        #[arg(required = true, value_name = "KEY")]
        keys: Vec<CiteKey>,
        #[command(flatten)]
        opts: CiteOpts,
    },
    /// Query crossref funders
    Funders {
        /// The id of a single funder. Omit to search by query terms.
        #[arg(short, long)]
        id: Option<String>,
        #[command(flatten)]
        opts: FundersOpts,
    },
    /// Query crossref members
    Members {
        /// The id of a single member. Omit to search by query terms.
        #[arg(short, long)]
        id: Option<String>,
        #[command(flatten)]
        opts: MembersOpts,
    },
    /// Query crossref journals
    Journals {
        /// The id (ISSN) of the journal. Omit to search journals by query terms.
        #[arg(short, long)]
        id: Option<String>,
        #[command(flatten)]
        opts: ListOpts,
    },
    /// List the licenses crossref works are published under
    Licenses {
        #[command(flatten)]
        opts: ListOpts,
    },
    /// Query crossref prefixes
    Prefixes {
        /// The id of the prefix.
        #[arg(short, long)]
        id: String,
    },
    /// Query crossref types
    Types {
        /// The id of the type. Omit to list all types.
        #[arg(short, long)]
        id: Option<Type>,
    },
    /// Re-serialize a work into another format through content negotiation
    Transform {
        /// The DOI of the work.
        doi: String,
        /// The format to render.
        #[arg(long, value_enum, default_value_t = Format::Bibtex)]
        format: Format,
        /// The CSL style to render a `citation` in. See the `styles` command.
        #[arg(long, default_value = "apa")]
        style: String,
        /// The locale to render a `citation` in, e.g. `de-DE`.
        #[arg(long)]
        locale: Option<String>,
    },
    /// List the CSL styles a citation can be rendered in
    Styles,
}

/// The [`CnFormat`] variants, as command line values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    /// RDF/XML
    RdfXml,
    /// RDF in the Turtle syntax
    Turtle,
    /// CSL JSON, the standard citeproc shape
    CiteprocJson,
    /// crossref's older, non-standard citeproc flavour
    CiteprocJsonIsh,
    /// RIS, for reference managers
    Ris,
    /// BibTeX
    Bibtex,
    /// crossref's own deposit schema
    CrossrefXml,
    /// crossref's text and data mining schema, carrying the full-text links
    CrossrefTdm,
    /// a citation formatted for reading; see `--style` and `--locale`
    Citation,
}

impl Format {
    fn into_cn_format(self, style: String, locale: Option<String>) -> CnFormat {
        match self {
            Format::RdfXml => CnFormat::RdfXml,
            Format::Turtle => CnFormat::Turtle,
            Format::CiteprocJson => CnFormat::CiteProcJson,
            Format::CiteprocJsonIsh => CnFormat::CiteProcJsonIsh,
            Format::Ris => CnFormat::Ris,
            Format::Bibtex => CnFormat::BibTex,
            Format::CrossrefXml => CnFormat::CrossrefXml,
            Format::CrossrefTdm => CnFormat::CrossrefTdm,
            Format::Citation => CnFormat::Bibliography { style, locale },
        }
    }
}

#[derive(Debug, Subcommand)]
enum Combined {
    /// Get Works of a specific Member
    Member { id: String },
    /// Get Works of a specific Funder
    Funder { id: String },
    /// Get Works of a specific Journal
    Journal { id: String },
    /// Get Works of a specific Prefix
    Prefix { id: String },
    /// Get Works of a specific Type
    Type { id: String },
}

#[derive(Debug, Args)]
struct Out {
    /// output path where the results shall be stored
    #[arg(short, long, global = true)]
    output: Option<PathBuf>,
    /// if the output file already exists, append instead of overwriting the file
    #[arg(short, long, global = true)]
    append: bool,
}

#[derive(Debug, Args)]
struct ClientOpts {
    /// The user agent to use for the crossref client
    #[arg(long, global = true)]
    user_agent: Option<String>,
    /// The token to use for the crossref client
    #[arg(long, global = true)]
    token: Option<String>,
    /// The email to use to get into crossref's polite pool
    #[arg(long, global = true)]
    polite: Option<String>,
}

impl ClientOpts {
    fn create_client(&self) -> crossref_client::Result<Crossref> {
        Crossref::builder()
            .user_agent(self.user_agent.as_deref())
            .token(self.token.as_deref())
            // set last so it wins over a bare `--user-agent`
            .polite(self.polite.as_deref())
            .build()
    }
}

/// The options every list route honours: free form terms and paging.
#[derive(Debug, Args)]
struct ListOpts {
    /// The free form terms for the query
    #[arg(short, long = "query")]
    query_terms: Vec<String>,
    /// limit the amount of results
    #[arg(short, long)]
    limit: Option<usize>,
    /// Sets an offset where crossref begins to retrieve items.
    #[arg(long)]
    offset: Option<usize>,
}

impl ListOpts {
    /// Resolves the paging flags into a single `ResultControl`.
    fn result_control(&self) -> Option<ResultControl> {
        match (self.limit, self.offset) {
            (Some(rows), Some(offset)) => Some(ResultControl::RowsOffset { rows, offset }),
            (Some(rows), None) => Some(ResultControl::Rows(rows)),
            (None, Some(offset)) => Some(ResultControl::Offset(offset)),
            (None, None) => None,
        }
    }
}

/// A `field=term` pair, naming one of the fields `/works` can be queried
/// against. The parse is where an unknown field is caught, so everything
/// downstream holds a query crossref will read.
#[derive(Debug, Clone)]
struct FieldQueryArg(FieldQuery);

impl FromStr for FieldQueryArg {
    type Err = String;

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        let Some((field, term)) = arg.split_once('=') else {
            return Err(format!("`{arg}` is not a `field=term` pair"));
        };
        if term.is_empty() {
            return Err(format!("`{field}` was given no term to match"));
        }
        FieldQuery::from_field(field, term)
            .map(FieldQueryArg)
            .ok_or_else(|| {
                format!(
                    "`{field}` is not a field crossref can be queried against. Try one of: {}",
                    FieldQuery::ALL_FIELDS.join(", ")
                )
            })
    }
}

/// A filter enum that can be named at runtime, which is what lets one
/// `--filter` flag be written once for every route that takes filters.
///
/// The library gives each of them these as inherent methods; this only gathers
/// them under a bound. Each route keeps its own vocabulary, so `/funders`
/// refuses a `/works` filter here rather than at crossref.
trait NamedFilter: Sized + Clone + Send + Sync + 'static {
    /// every filter name the route accepts
    const NAMES: &'static [&'static str];

    /// the filter of that name carrying that value
    fn from_pair(name: &str, value: Option<&str>) -> Result<Self, FilterParseError>;
}

impl NamedFilter for WorksFilter {
    const NAMES: &'static [&'static str] = WorksFilter::ALL_NAMES;

    fn from_pair(name: &str, value: Option<&str>) -> Result<Self, FilterParseError> {
        WorksFilter::from_name(name, value)
    }
}

impl NamedFilter for FundersFilter {
    const NAMES: &'static [&'static str] = FundersFilter::ALL_NAMES;

    fn from_pair(name: &str, value: Option<&str>) -> Result<Self, FilterParseError> {
        FundersFilter::from_name(name, value)
    }
}

impl NamedFilter for MembersFilter {
    const NAMES: &'static [&'static str] = MembersFilter::ALL_NAMES;

    fn from_pair(name: &str, value: Option<&str>) -> Result<Self, FilterParseError> {
        MembersFilter::from_name(name, value)
    }
}

/// A `name:value` filter pair, naming one of the filters a route narrows by.
/// As with [`FieldQueryArg`], the parse is what catches a filter crossref
/// would answer with a `400`, and a value it could not read.
#[derive(Debug, Clone)]
struct FilterArg<F>(F);

impl<F: NamedFilter> FromStr for FilterArg<F> {
    type Err = String;

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        let (name, value) = match arg.split_once(':') {
            Some((name, value)) => (name, Some(value)),
            None => (arg, None),
        };

        F::from_pair(name, value)
            .map(FilterArg)
            .map_err(|err| match err {
                // the one error a caller cannot fix without the vocabulary,
                // which is what crossref itself answers a bad filter with
                FilterParseError::UnknownName { .. } => {
                    format!("{err}. Try one of: {}", F::NAMES.join(", "))
                }
                err => err.to_string(),
            })
    }
}

/// Folds the filters a flag was repeated for onto a query. Repeats are meant:
/// crossref ANDs them, so a date range is two filters and two types are two.
fn narrow<Q, F>(query: Q, filters: Vec<FilterArg<F>>, filter: impl Fn(Q, F) -> Q) -> Q {
    filters
        .into_iter()
        .fold(query, |query, arg| filter(query, arg.0))
}

/// What `/works` honours on top of [`ListOpts`], and no other route does.
#[derive(Debug, Args)]
struct WorksOpts {
    #[command(flatten)]
    list: ListOpts,
    /// Match a whole reference against titles, authors, ISSNs and publication
    /// years at once. Crossref's own citation look up.
    #[arg(long, value_name = "CITATION")]
    query_bibliographic: Option<String>,
    /// Match against a work's title, including its subtitle
    #[arg(long, value_name = "TERM")]
    query_title: Option<String>,
    /// Match against author given and family names
    #[arg(long, value_name = "TERM")]
    query_author: Option<String>,
    /// Match against any other field, e.g. `--query-field container-title=Nature`.
    /// Repeat for more fields; an unknown field lists every one crossref takes.
    #[arg(long, value_name = "FIELD=TERM")]
    query_field: Vec<FieldQueryArg>,
    /// Narrow the results, e.g. `--filter from-pub-date:2020-01-01` or
    /// `--filter has-abstract`. Repeat to narrow further; every filter is
    /// ANDed. An unknown one lists every filter crossref takes.
    #[arg(long, value_name = "NAME[:VALUE]")]
    filter: Vec<FilterArg<WorksFilter>>,
    /// How to sort the results, such as updated, indexed, published, issued
    #[arg(long)]
    sort: Option<Sort>,
    /// How to order the results: asc or desc
    #[arg(long)]
    order: Option<Order>,
    /// Request random works. Crossref ignores every other option when set.
    #[arg(long)]
    sample: Option<usize>,
}

impl WorksOpts {
    /// The field queries the flags add up to, the named ones first.
    ///
    /// A field asked for twice is an error rather than a request: crossref
    /// answers a repeated `query.title` with a single one of the two terms,
    /// and which one it keeps is not ours to guess.
    fn field_queries(&self) -> Result<Vec<FieldQuery>, String> {
        let named = [
            self.query_bibliographic
                .as_deref()
                .map(FieldQuery::bibliographic),
            self.query_title.as_deref().map(FieldQuery::title),
            self.query_author.as_deref().map(FieldQuery::author),
        ];
        let queries: Vec<_> = named
            .into_iter()
            .flatten()
            .chain(self.query_field.iter().map(|arg| arg.0.clone()))
            .collect();

        for (at, query) in queries.iter().enumerate() {
            if queries[..at]
                .iter()
                .any(|seen| seen.field() == query.field())
            {
                return Err(format!(
                    "`{}` was asked for twice; crossref matches a field against one term",
                    query.name()
                ));
            }
        }
        Ok(queries)
    }

    fn into_query(self) -> Result<WorksQuery, String> {
        let field_queries = self.field_queries()?;
        let query = WorksQuery::empty()
            .queries(&self.list.query_terms)
            .field_queries(field_queries)
            .sort(self.sort)
            .order(self.order)
            .sample(self.sample)
            .result_control(self.list.result_control().map(WorkResultControl::Standard));

        Ok(narrow(query, self.filter, WorksQuery::filter))
    }
}

/// What `/funders` honours on top of [`ListOpts`]. `location` is the only
/// filter it takes, and the one filter `/works` does not.
#[derive(Debug, Args)]
struct FundersOpts {
    #[command(flatten)]
    list: ListOpts,
    /// Narrow the results, e.g. `--filter location:Norway`. An unknown filter
    /// lists every one this route takes.
    #[arg(long, value_name = "NAME[:VALUE]")]
    filter: Vec<FilterArg<FundersFilter>>,
}

impl FundersOpts {
    fn into_query(self) -> FundersQuery {
        let query = FundersQuery::empty()
            .queries(&self.list.query_terms)
            .result_control(self.list.result_control());

        narrow(query, self.filter, FundersQuery::filter)
    }
}

/// What `/members` honours on top of [`ListOpts`]: the prefix a member owns
/// and the size of what they have deposited.
#[derive(Debug, Args)]
struct MembersOpts {
    #[command(flatten)]
    list: ListOpts,
    /// Narrow the results, e.g. `--filter prefix:10.1016` or
    /// `--filter current-doi-count:1000`. Repeat to narrow further; every
    /// filter is ANDed. An unknown one lists every filter this route takes.
    #[arg(long, value_name = "NAME[:VALUE]")]
    filter: Vec<FilterArg<MembersFilter>>,
}

impl MembersOpts {
    fn into_query(self) -> MembersQuery {
        let query = MembersQuery::empty()
            .queries(&self.list.query_terms)
            .result_control(self.list.result_control());

        narrow(query, self.filter, MembersQuery::filter)
    }
}

/// How hard `cite` tries before it says it does not know.
#[derive(Debug, Args)]
struct CiteOpts {
    /// How many works to weigh against the key per attempt
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u16).range(1..=1000))]
    candidates: u16,
    /// How many years either side of the key's year still count as the same
    /// publication. A work published online one year and in an issue the next
    /// is cited by either.
    #[arg(long, default_value_t = 1, value_name = "YEARS", value_parser = clap::value_parser!(u8).range(0..=50))]
    year_window: u8,
    /// How many accented spellings to guess at when the key as written finds
    /// nothing. Citation keys drop diacritics and crossref folds none, so
    /// `Floysand` reaches the works of `Fløysand` no other way.
    #[arg(long, default_value_t = 12, value_parser = clap::value_parser!(u16).range(0..=500))]
    spellings: u16,
}

/// What resolving one citation key came to.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Resolution {
    /// the key, as it was written
    key: String,
    /// what the key was read as
    surnames: Vec<String>,
    year: i32,
    et_al: bool,
    verdict: Verdict,
    /// the spelling that found the work, when the key's diacritics had to be
    /// guessed back
    #[serde(skip_serializing_if = "Option::is_none")]
    matched_as: Option<Vec<String>>,
    /// what the answer cost in requests
    requests: usize,
    /// the answer, when there is one, repeated here so a caller need not reach
    /// into the work for it
    #[serde(skip_serializing_if = "Option::is_none")]
    doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    work: Option<Box<Work>>,
    /// the works that answered when more than one did, or the nearest misses
    /// when none did
    #[serde(skip_serializing_if = "Vec::is_empty")]
    candidates: Vec<Candidate>,
}

/// What a key came to say about the works crossref returned.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Verdict {
    /// one work, and the key vouches for it
    Matched,
    /// several works the key vouches for, and nothing in the key to choose
    /// between them
    Ambiguous,
    /// nothing the key vouches for
    Unmatched,
}

/// A work a key was weighed against, cut down to what makes it recognisable.
#[derive(Debug, Serialize)]
struct Candidate {
    #[serde(rename = "DOI")]
    doi: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    score: Option<f32>,
    /// why the key does not vouch for it, empty when it does
    #[serde(skip_serializing_if = "Vec::is_empty")]
    mismatches: Vec<Mismatch>,
}

impl Candidate {
    fn new(work: &Work, mismatches: Vec<Mismatch>) -> Self {
        Candidate {
            doi: work.doi.clone(),
            title: work.title.first().cloned(),
            authors: citekey::families(work)
                .into_iter()
                .map(str::to_string)
                .collect(),
            year: citekey::published_year(work),
            score: work.score,
            mismatches,
        }
    }
}

impl Resolution {
    /// The half of the answer that is the key itself, whatever crossref said.
    fn about(key: &CiteKey, requests: usize, verdict: Verdict) -> Self {
        Resolution {
            key: key.key().to_string(),
            surnames: key.surnames().to_vec(),
            year: key.year(),
            et_al: key.et_al(),
            verdict,
            matched_as: None,
            requests,
            doi: None,
            work: None,
            candidates: Vec::new(),
        }
    }
}

/// One request `cite` is willing to spend on a key, and the spelling it spends
/// it on.
struct Attempt {
    /// the guessed spelling this asks about, or [`None`] for the key as written
    spelling: Option<Vec<String>>,
    query: WorksQuery,
}

/// Everything worth asking crossref about a key, in the order it is worth
/// asking: the surnames against the author field, then the key as the whole
/// citation it stands in for, then one guessed spelling at a time.
///
/// The guesses come last because they are only ever needed when the name was
/// written with a mark the key could not keep, and cost a request each.
fn attempts(key: &CiteKey, opts: &CiteOpts) -> Vec<Attempt> {
    let as_written = [
        author_query(key.surnames(), key, opts),
        bibliographic_query(key, opts),
    ]
    .map(|query| Attempt {
        spelling: None,
        query,
    });

    as_written
        .into_iter()
        .chain(
            key.spellings()
                .into_iter()
                .take(usize::from(opts.spellings))
                .map(|spelling| Attempt {
                    query: author_query(&spelling, key, opts),
                    spelling: Some(spelling),
                }),
        )
        .collect()
}

/// The works whose authors the key could be naming.
fn author_query(surnames: &[String], key: &CiteKey, opts: &CiteOpts) -> WorksQuery {
    let authors = FieldQuery::author(surnames.join(" "));
    published_around(WorksQuery::empty().field_query(authors), key, opts)
}

/// The key as the whole citation it stands in for, which crossref matches
/// against titles, authors, ISSNs and years at once.
///
/// The year goes into the terms rather than beside them: crossref answers a
/// `query.author` and a `query.bibliographic` sent together with nothing at
/// all, so this has to be a request of its own.
fn bibliographic_query(key: &CiteKey, opts: &CiteOpts) -> WorksQuery {
    let citation = format!("{} {}", key.surnames().join(" "), key.year());
    let citation = FieldQuery::bibliographic(citation);
    published_around(WorksQuery::empty().field_query(citation), key, opts)
}

/// Narrows a query to the years a work cited as this key's could carry, and to
/// as many works as are worth weighing.
///
/// Without the window a bare surname matches a career; `query.author` ORs its
/// terms, so even two surnames leave hundreds of works that share one of them.
fn published_around(query: WorksQuery, key: &CiteKey, opts: &CiteOpts) -> WorksQuery {
    let within = i32::from(opts.year_window);
    let date = |year, month, day| {
        NaiveDate::from_ymd_opt(year, month, day).expect("a year that parsed bounds a real date")
    };

    query
        .filter(WorksFilter::FromPubDate(date(key.year() - within, 1, 1)))
        .filter(WorksFilter::UntilPubDate(date(key.year() + within, 12, 31)))
        .result_control(WorkResultControl::Standard(ResultControl::Rows(
            usize::from(opts.candidates),
        )))
}

/// Asks crossref for the works a key could stand for and keeps the ones it
/// vouches for, stopping at the first attempt that finds any.
///
/// Crossref answers every query with something, so the deciding is done here
/// rather than there: a work is the key's only if the key's surnames are
/// credited, in order, from the first author, and the year is close enough.
async fn resolve(
    key: &CiteKey,
    opts: &CiteOpts,
    client: &Crossref,
) -> crossref_client::Result<Resolution> {
    let within = i32::from(opts.year_window);
    let mut requests = 0;
    let mut turned_down: Vec<Candidate> = Vec::new();

    for attempt in attempts(key, opts) {
        let found = client.works(attempt.query).await?;
        requests += 1;

        let mut vouched: Vec<Work> = Vec::new();
        for work in found.items {
            let mismatches = key.mismatches(&work, within);
            if mismatches.is_empty() {
                vouched.push(work);
            } else if !turned_down.iter().any(|seen| seen.doi == work.doi) {
                turned_down.push(Candidate::new(&work, mismatches));
            }
        }

        if vouched.len() == 1 {
            let work = vouched.pop().expect("the one work that was vouched for");
            let mut resolution = Resolution::about(key, requests, Verdict::Matched);
            resolution.matched_as = attempt.spelling;
            resolution.doi = Some(work.doi.clone());
            resolution.work = Some(Box::new(work));
            return Ok(resolution);
        }
        if !vouched.is_empty() {
            let mut resolution = Resolution::about(key, requests, Verdict::Ambiguous);
            resolution.matched_as = attempt.spelling;
            resolution.candidates = vouched
                .iter()
                .map(|work| Candidate::new(work, Vec::new()))
                .collect();
            return Ok(resolution);
        }
    }

    let mut resolution = Resolution::about(key, requests, Verdict::Unmatched);
    // in crossref's own order, so the nearest misses are the ones it ranked
    // highest for the key as written
    turned_down.truncate(3);
    resolution.candidates = turned_down;
    Ok(resolution)
}

impl Command {
    /// Runs the command, answering with what the shell should exit on. Only
    /// `cite` has anything to say there; everything else that went wrong is an
    /// [`Err`].
    async fn run<W: Write>(
        self,
        mut writer: W,
        client: &Crossref,
    ) -> Result<ExitCode, Box<dyn std::error::Error>> {
        match self {
            Command::Types { id } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.type_(&id).await?)?,
                None => serde_json::to_writer_pretty(writer, &client.types().await?)?,
            },
            Command::Prefixes { id } => {
                serde_json::to_writer_pretty(writer, &client.prefix(&id).await?)?
            }
            Command::Styles => serde_json::to_writer_pretty(writer, &client.styles().await?)?,
            Command::Transform {
                doi,
                format,
                style,
                locale,
            } => {
                let body = client
                    .transform(&doi, &format.into_cn_format(style, locale))
                    .await?;
                // the body is whatever crossref rendered, not json to re-encode
                writer.write_all(body.as_bytes())?;
            }
            Command::Cite { keys, opts } => {
                let mut resolutions = Vec::with_capacity(keys.len());
                for key in &keys {
                    resolutions.push(resolve(key, &opts, client).await?);
                }
                let resolved = resolutions
                    .iter()
                    .all(|resolution| matches!(resolution.verdict, Verdict::Matched));

                serde_json::to_writer_pretty(writer, &resolutions)?;
                return Ok(if resolved {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                });
            }
            Command::Journals { id, opts } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.journal(&id).await?)?,
                None => {
                    let query = JournalsQuery::empty()
                        .queries(&opts.query_terms)
                        .result_control(opts.result_control());
                    serde_json::to_writer_pretty(writer, &client.journals(query).await?)?
                }
            },
            Command::Licenses { opts } => {
                let query = LicensesQuery::empty()
                    .queries(&opts.query_terms)
                    .result_control(opts.result_control());
                serde_json::to_writer_pretty(writer, &client.licenses(query).await?)?
            }
            Command::Members { id, opts } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.member(&id).await?)?,
                None => {
                    let members = client.members(opts.into_query()).await?;
                    serde_json::to_writer_pretty(writer, &members)?
                }
            },
            Command::Funders { id, opts } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.funder(&id).await?)?,
                None => {
                    let funders = client.funders(opts.into_query()).await?;
                    serde_json::to_writer_pretty(writer, &funders)?
                }
            },
            Command::Works {
                id,
                opts,
                combined,
                deep_page,
            } => {
                if let Some(id) = &id {
                    serde_json::to_writer_pretty(writer, &client.work(id).await?)?;
                    return Ok(ExitCode::SUCCESS);
                }

                let query = opts.into_query()?;
                let query = match combined {
                    Some(Combined::Journal { id }) => {
                        query.into_combined_query::<crossref_client::Journals>(&id)
                    }
                    Some(Combined::Type { id }) => {
                        query.into_combined_query::<crossref_client::Types>(&id)
                    }
                    Some(Combined::Funder { id }) => {
                        query.into_combined_query::<crossref_client::Funders>(&id)
                    }
                    Some(Combined::Member { id }) => {
                        query.into_combined_query::<crossref_client::Members>(&id)
                    }
                    Some(Combined::Prefix { id }) => {
                        query.into_combined_query::<crossref_client::Prefixes>(&id)
                    }
                    None => query.into(),
                };

                if deep_page {
                    let mut pages = client.deep_page(query);
                    let mut works = Vec::new();
                    while let Some(page) = pages.next().await {
                        works.extend(page?.items);
                    }
                    serde_json::to_writer_pretty(writer, &works)?
                } else {
                    serde_json::to_writer_pretty(writer, &client.works(query).await?)?
                }
            }
        }
        Ok(ExitCode::SUCCESS)
    }
}

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app = App::parse();
    let client = app.client.create_client()?;

    match &app.out.output {
        Some(path) => {
            let file = if app.out.append && path.exists() {
                fs::OpenOptions::new().append(true).open(path)?
            } else {
                fs::File::create(path)?
            };
            app.command.run(file, &client).await
        }
        None => app.command.run(std::io::stdout(), &client).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossref_client::CrossrefRoute;

    /// The route a `crossref works ...` command line renders to, or the error
    /// the flags add up to.
    fn works_route(args: &[&str]) -> Result<String, String> {
        let app = App::try_parse_from(args).map_err(|err| err.to_string())?;
        let Command::Works { opts, .. } = app.command else {
            panic!("`{args:?}` did not parse into a works command");
        };
        let query = opts.into_query()?;
        Ok(query.route().expect("a works query always renders"))
    }

    #[test]
    fn a_bibliographic_query_reaches_crossrefs_reference_matching() {
        let route = works_route(&[
            "crossref",
            "works",
            "--query-bibliographic",
            "Feynman 1960 There's Plenty of Room at the Bottom",
        ])
        .unwrap();

        assert!(
            route.contains("query.bibliographic=Feynman%201960"),
            "{route}"
        );
    }

    #[test]
    fn the_named_field_flags_render_their_own_fields() {
        let route = works_route(&[
            "crossref",
            "works",
            "--query-title",
            "room at the bottom",
            "--query-author",
            "feynman",
        ])
        .unwrap();

        assert!(
            route.contains("query.title=room%20at%20the%20bottom"),
            "{route}"
        );
        assert!(route.contains("query.author=feynman"), "{route}");
    }

    #[test]
    fn a_field_query_can_name_any_field_the_flags_do_not() {
        let route = works_route(&[
            "crossref",
            "works",
            "--query-field",
            "container-title=Nature",
            "--query-field",
            "publisher-name=Elsevier",
        ])
        .unwrap();

        assert!(route.contains("query.container-title=Nature"), "{route}");
        assert!(route.contains("query.publisher-name=Elsevier"), "{route}");
    }

    #[test]
    fn field_queries_sit_alongside_the_free_form_terms() {
        let route = works_route(&[
            "crossref",
            "works",
            "--query",
            "nanotechnology",
            "--query-author",
            "feynman",
            "--limit",
            "5",
        ])
        .unwrap();

        assert!(route.contains("query=nanotechnology"), "{route}");
        assert!(route.contains("query.author=feynman"), "{route}");
        assert!(route.contains("rows=5"), "{route}");
    }

    #[test]
    fn a_works_query_without_field_flags_sends_none() {
        let route = works_route(&["crossref", "works", "--query", "nanotechnology"]).unwrap();

        assert!(!route.contains("query."), "{route}");
    }

    /// The field is checked while parsing, so `query.pubisher-name` never
    /// leaves for crossref to answer with an unfiltered result set.
    #[test]
    fn an_unknown_field_is_rejected_with_the_fields_that_do_exist() {
        let err = works_route(&[
            "crossref",
            "works",
            "--query-field",
            "pubisher-name=Elsevier",
        ])
        .unwrap_err();

        assert!(err.contains("pubisher-name"), "{err}");
        assert!(err.contains("publisher-name"), "{err}");
    }

    #[test]
    fn a_field_query_without_a_term_is_rejected() {
        let err = works_route(&["crossref", "works", "--query-field", "author"]).unwrap_err();
        assert!(err.contains("`author` is not a `field=term` pair"), "{err}");

        let err = works_route(&["crossref", "works", "--query-field", "author="]).unwrap_err();
        assert!(err.contains("no term to match"), "{err}");
    }

    #[test]
    fn one_field_asked_for_twice_is_rejected_rather_than_guessed_at() {
        let err = works_route(&[
            "crossref",
            "works",
            "--query-author",
            "feynman",
            "--query-field",
            "author=gell-mann",
        ])
        .unwrap_err();

        assert!(err.contains("query.author"), "{err}");

        let err = works_route(&[
            "crossref",
            "works",
            "--query-field",
            "editor=hey",
            "--query-field",
            "editor=walther",
        ])
        .unwrap_err();

        assert!(err.contains("query.editor"), "{err}");
    }

    #[test]
    fn filters_narrow_a_works_query_and_and_together() {
        let route = works_route(&[
            "crossref",
            "works",
            "--query",
            "salmon",
            "--filter",
            "from-pub-date:2020-01-01",
            "--filter",
            "until-pub-date:2021-12-31",
            "--filter",
            "has-abstract",
        ])
        .unwrap();

        assert!(route.contains("from-pub-date"), "{route}");
        assert!(route.contains("2020-01-01"), "{route}");
        assert!(route.contains("until-pub-date"), "{route}");
        assert!(route.contains("2021-12-31"), "{route}");
        // a marker asks only whether the record has one
        assert!(route.contains("has-abstract"), "{route}");
        assert!(route.contains("true"), "{route}");
    }

    #[test]
    fn a_filter_the_route_does_not_take_is_rejected_with_the_ones_it_does() {
        let err = works_route(&["crossref", "works", "--filter", "has-astract"]).unwrap_err();

        assert!(err.contains("has-astract"), "{err}");
        assert!(err.contains("has-abstract"), "{err}");
    }

    #[test]
    fn a_filter_value_crossref_could_not_read_is_rejected_before_it_is_sent() {
        let err =
            works_route(&["crossref", "works", "--filter", "from-pub-date:2020"]).unwrap_err();
        assert!(err.contains("`2020` is not a value"), "{err}");

        let err = works_route(&["crossref", "works", "--filter", "issn"]).unwrap_err();
        assert!(err.contains("needs a value"), "{err}");

        let err =
            works_route(&["crossref", "works", "--filter", "has-abstract:false"]).unwrap_err();
        assert!(err.contains("takes no value"), "{err}");
    }

    /// A filter value carrying a comma cannot be sent at all -- crossref
    /// splits the `filter` parameter on it -- and the query says so rather
    /// than sending something that means another filter.
    #[test]
    fn a_filter_value_that_cannot_be_sent_is_reported_by_the_query() {
        let parsed = App::try_parse_from(["crossref", "works", "--filter", "container-title:a,b"]);
        let Ok(App {
            command: Command::Works { opts, .. },
            ..
        }) = parsed
        else {
            panic!("a filter with a comma parses; it is the sending that fails")
        };

        assert!(opts.into_query().unwrap().route().is_err());
    }

    #[test]
    fn the_routes_that_narrow_by_nothing_take_no_filter_at_all() {
        for route in ["journals", "licenses"] {
            let parsed = App::try_parse_from(["crossref", route, "--filter", "has-abstract"]);
            assert!(parsed.is_err(), "`crossref {route}` took --filter");
        }
    }

    #[test]
    fn funders_and_members_narrow_by_their_own_filters() {
        let route = |args: &[&str]| {
            let app = App::try_parse_from(args).expect("a list command line");
            match app.command {
                Command::Funders { opts, .. } => opts.into_query().route(),
                Command::Members { opts, .. } => opts.into_query().route(),
                _ => panic!("`{args:?}` is neither funders nor members"),
            }
            .expect("a list query renders")
        };

        let funders = route(&[
            "crossref",
            "funders",
            "-q",
            "research",
            "--filter",
            "location:Norway",
        ]);
        assert!(funders.contains("query=research"), "{funders}");
        assert!(funders.contains("location"), "{funders}");
        assert!(funders.contains("Norway"), "{funders}");

        let members = route(&[
            "crossref",
            "members",
            "--filter",
            "prefix:10.1016",
            "--filter",
            "current-doi-count:1000",
        ]);
        assert!(members.contains("prefix"), "{members}");
        assert!(members.contains("10.1016"), "{members}");
        assert!(members.contains("current-doi-count"), "{members}");
    }

    /// Each route keeps its own vocabulary, so a filter that belongs to
    /// another one is refused here rather than by crossref -- which is the
    /// whole reason the three of them do not share a flag.
    #[test]
    fn a_filter_belonging_to_another_route_is_refused_with_this_routes_own() {
        let err = App::try_parse_from(["crossref", "funders", "--filter", "has-abstract"])
            .expect_err("a works filter on /funders")
            .to_string();
        assert!(err.contains("has-abstract"), "{err}");
        assert!(err.contains("location"), "{err}");

        // `location` used to sit on WorksFilter, where /works answers it with a 400
        let err = works_route(&["crossref", "works", "--filter", "location:Norway"]).unwrap_err();
        assert!(err.contains("not a filter this route takes"), "{err}");

        let err = App::try_parse_from(["crossref", "members", "--filter", "location:Norway"])
            .expect_err("a funders filter on /members")
            .to_string();
        assert!(err.contains("backfile-doi-count"), "{err}");
    }

    #[test]
    fn a_members_filter_counts_dois_and_will_not_take_a_word_for_it() {
        let err =
            App::try_parse_from(["crossref", "members", "--filter", "current-doi-count:many"])
                .expect_err("a count that is not a number")
                .to_string();

        assert!(err.contains("`many` is not a value"), "{err}");
    }

    /// The `/works` routes a `crossref cite ...` command line would ask for,
    /// in the order it would ask.
    fn cite_attempts(args: &[&str]) -> Vec<String> {
        let app = App::try_parse_from(args).expect("a cite command line");
        let Command::Cite { keys, opts } = app.command else {
            panic!("`{args:?}` did not parse into a cite command");
        };
        let key = keys.first().expect("clap requires a key");

        attempts(key, &opts)
            .into_iter()
            .map(|attempt| attempt.query.route().expect("a works query renders"))
            .collect()
    }

    #[test]
    fn a_key_is_first_asked_for_as_authors_published_around_the_year() {
        let routes = cite_attempts(&["crossref", "cite", "@LindforsJakobsen2022"]);

        assert!(
            routes[0].contains("query.author=Lindfors%20Jakobsen"),
            "{}",
            routes[0]
        );
        assert!(routes[0].contains("2021-01-01"), "{}", routes[0]);
        assert!(routes[0].contains("2023-12-31"), "{}", routes[0]);
        assert!(routes[0].contains("rows=20"), "{}", routes[0]);
    }

    #[test]
    fn the_year_window_widens_both_ends_of_the_search() {
        let routes = cite_attempts(&["crossref", "cite", "@Lindfors2022", "--year-window", "3"]);

        assert!(routes[0].contains("2019-01-01"), "{}", routes[0]);
        assert!(routes[0].contains("2025-12-31"), "{}", routes[0]);
    }

    /// The year rides along in the terms rather than beside them, because
    /// crossref answers a `query.author` and a `query.bibliographic` sent
    /// together with nothing at all.
    #[test]
    fn the_key_is_then_asked_for_as_a_whole_citation() {
        let routes = cite_attempts(&["crossref", "cite", "@LindforsJakobsen2022"]);

        assert!(
            routes[1].contains("query.bibliographic=Lindfors%20Jakobsen%202022"),
            "{}",
            routes[1]
        );
        assert!(!routes[1].contains("query.author"), "{}", routes[1]);
    }

    #[test]
    fn the_accented_spellings_are_guessed_at_last_and_likeliest_first() {
        let routes = cite_attempts(&["crossref", "cite", "@Floysand2021"]);

        assert_eq!(2 + 12, routes.len());
        // %C3%B8 is the ø that the key could not carry
        assert!(
            routes[2].contains("query.author=Fl%C3%B8ysand"),
            "{}",
            routes[2]
        );
    }

    #[test]
    fn the_guessing_can_be_turned_off() {
        let routes = cite_attempts(&["crossref", "cite", "@Floysand2021", "--spellings", "0"]);

        assert_eq!(2, routes.len());
    }

    #[test]
    fn a_key_that_names_no_one_is_rejected_before_a_request_is_spent() {
        let err = App::try_parse_from(["crossref", "cite", "@2022"])
            .expect_err("a key naming nobody")
            .to_string();

        assert!(err.contains("names no author"), "{err}");
    }

    #[test]
    fn a_verdict_reads_as_the_word_for_it() {
        assert_eq!(
            "\"matched\"",
            serde_json::to_string(&Verdict::Matched).unwrap()
        );
        assert_eq!(
            "\"ambiguous\"",
            serde_json::to_string(&Verdict::Ambiguous).unwrap()
        );
        assert_eq!(
            "\"unmatched\"",
            serde_json::to_string(&Verdict::Unmatched).unwrap()
        );
    }

    /// The flags are `/works`-only because `query.x` is, and clap is what
    /// keeps them there.
    #[test]
    fn the_other_routes_do_not_take_field_queries() {
        for route in ["journals", "members", "funders", "licenses"] {
            let parsed = App::try_parse_from(["crossref", route, "--query-title", "ecology"]);
            assert!(parsed.is_err(), "`crossref {route}` took --query-title");
        }
    }

    #[test]
    fn every_field_is_reachable_through_query_field() {
        for field in FieldQuery::ALL_FIELDS {
            let arg = format!("{field}=term");
            let route = works_route(&["crossref", "works", "--query-field", &arg])
                .unwrap_or_else(|err| panic!("`--query-field {arg}` was rejected: {err}"));
            assert!(route.contains(&format!("query.{field}=term")), "{route}");
        }
    }
}
