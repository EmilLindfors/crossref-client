//! A command line front end to the crossref api.
//!
//! Each subcommand offers exactly the flags its route honours. `--sort`,
//! `--order` and `--sample` only exist under `works`, because `/funders`,
//! `/members`, `/journals` and `/licenses` all answer them with a `400`; the
//! flags used to be shared across every subcommand and silently dropped where
//! they could not be sent, so `crossref journals --sort score` returned results
//! in a different order than asked for and said nothing.
use clap::{Args, Parser, Subcommand, ValueEnum};
use crossref_client::{
    AsyncIterator, CnFormat, Crossref, FundersQuery, JournalsQuery, LicensesQuery, MembersQuery,
    Order, ResultControl, Sort, Type, WorkResultControl, WorksQuery,
};

use std::{fs, io::Write, path::PathBuf};

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
    /// Query crossref funders
    Funders {
        /// The id of a single funder. Omit to search by query terms.
        #[arg(short, long)]
        id: Option<String>,
        #[command(flatten)]
        opts: ListOpts,
    },
    /// Query crossref members
    Members {
        /// The id of a single member. Omit to search by query terms.
        #[arg(short, long)]
        id: Option<String>,
        #[command(flatten)]
        opts: ListOpts,
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

/// What `/works` honours on top of [`ListOpts`], and no other route does.
#[derive(Debug, Args)]
struct WorksOpts {
    #[command(flatten)]
    list: ListOpts,
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
    fn into_query(self) -> WorksQuery {
        WorksQuery::empty()
            .queries(&self.list.query_terms)
            .sort(self.sort)
            .order(self.order)
            .sample(self.sample)
            .result_control(self.list.result_control().map(WorkResultControl::Standard))
    }
}

impl Command {
    async fn run<W: Write>(
        self,
        mut writer: W,
        client: &Crossref,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                    let query = MembersQuery::empty()
                        .queries(&opts.query_terms)
                        .result_control(opts.result_control());
                    serde_json::to_writer_pretty(writer, &client.members(query).await?)?
                }
            },
            Command::Funders { id, opts } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.funder(&id).await?)?,
                None => {
                    let query = FundersQuery::empty()
                        .queries(&opts.query_terms)
                        .result_control(opts.result_control());
                    serde_json::to_writer_pretty(writer, &client.funders(query).await?)?
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
                    return Ok(());
                }

                let query = opts.into_query();
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
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            app.command.run(file, &client).await?;
        }
        None => app.command.run(std::io::stdout(), &client).await?,
    }
    Ok(())
}
