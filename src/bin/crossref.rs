use clap::{Args, Parser, Subcommand};
use crossref_client::{
    AsyncIterator, Crossref, FundersQuery, JournalsQuery, MembersQuery, Order, ResultControl, Sort,
    Type, WorkResultControl, WorksQuery,
};

use std::{fs, io::Write, path::PathBuf};

/// Applies the shared list options onto a query that uses the standard result control.
macro_rules! apply_opts {
    ($ty:ident, $opts:expr) => {
        $ty {
            queries: $opts.query_terms.clone(),
            sort: $opts.sort,
            order: $opts.order,
            result_control: $opts.result_control(),
            ..Default::default()
        }
    };
}

#[derive(Debug, Parser)]
#[command(
    name = "crossref",
    about = "Access the crossref API from the command line.",
    version
)]
struct App {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Query crossref works
    Works {
        /// Enable deep paging. If a limit is set, then the limit takes priority.
        #[arg(short, long)]
        deep_page: bool,
        #[command(flatten)]
        opts: Opts,
        #[command(subcommand)]
        combined: Option<Combined>,
    },
    /// Query crossref funders
    Funders {
        #[command(flatten)]
        opts: Opts,
    },
    /// Query crossref members
    Members {
        #[command(flatten)]
        opts: Opts,
    },
    /// Query crossref journals
    Journals {
        /// The id (ISSN) of the journal. Omit to search journals by query terms.
        #[arg(long)]
        id: Option<String>,
        #[command(flatten)]
        opts: Opts,
    },
    /// Query crossref prefixes
    Prefixes {
        /// The id of the prefix.
        #[arg(long)]
        id: String,
        #[command(flatten)]
        client_opts: ClientOpts,
        #[command(flatten)]
        out: Out,
    },
    /// Query crossref types
    Types {
        /// The id of the type. Omit to list all types.
        #[arg(long)]
        id: Option<Type>,
        #[command(flatten)]
        client_opts: ClientOpts,
        #[command(flatten)]
        out: Out,
    },
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
    #[arg(short, long)]
    output: Option<PathBuf>,
    /// if the output file already exists, append instead of overwriting the file
    #[arg(short, long)]
    append: bool,
}

#[derive(Debug, Args)]
struct ClientOpts {
    /// The user agent to use for the crossref client
    #[arg(long)]
    user_agent: Option<String>,
    /// The token to use for the crossref client
    #[arg(long)]
    token: Option<String>,
    /// The email to use to get into crossref's polite pool
    #[arg(long)]
    polite: Option<String>,
}

impl ClientOpts {
    fn create_client(&self) -> crossref_client::Result<Crossref> {
        let mut builder = Crossref::builder();

        if let Some(agent) = &self.user_agent {
            builder = builder.user_agent(agent);
        }
        if let Some(token) = &self.token {
            builder = builder.token(token);
        }
        if let Some(polite) = &self.polite {
            builder = builder.polite(polite);
        }
        builder.build()
    }
}

#[derive(Debug, Args)]
struct Opts {
    #[command(flatten)]
    out: Out,
    /// limit the amount of results
    #[arg(short, long)]
    limit: Option<usize>,
    /// The id of component.
    #[arg(short, long)]
    id: Option<String>,
    /// The free form terms for the query
    #[arg(short, long = "query")]
    query_terms: Vec<String>,
    /// How to sort the results, such as updated, indexed, published, issued
    #[arg(long)]
    sort: Option<Sort>,
    /// How to order the results: asc or desc
    #[arg(long)]
    order: Option<Order>,
    /// Request random elements. Overrides all other options.
    #[arg(long)]
    sample: Option<usize>,
    /// Sets an offset where crossref begins to retrieve items.
    #[arg(long)]
    offset: Option<usize>,
    #[command(flatten)]
    client_opts: ClientOpts,
}

impl Opts {
    /// Resolves the paging flags into a single `ResultControl`.
    ///
    /// `sample` wins over everything, then rows+offset, then each on its own.
    fn result_control(&self) -> Option<ResultControl> {
        match self.sample {
            Some(sample) => Some(ResultControl::Sample(sample)),
            None => self.paging(),
        }
    }

    /// The paging flags on their own, for routes that reject `sample`.
    fn paging(&self) -> Option<ResultControl> {
        match (self.limit, self.offset) {
            (Some(rows), Some(offset)) => Some(ResultControl::RowsOffset { rows, offset }),
            (Some(rows), None) => Some(ResultControl::Rows(rows)),
            (None, Some(offset)) => Some(ResultControl::Offset(offset)),
            (None, None) => None,
        }
    }
}

impl Command {
    fn client_opts(&self) -> &ClientOpts {
        match self {
            Command::Works { opts, .. }
            | Command::Funders { opts, .. }
            | Command::Members { opts, .. }
            | Command::Journals { opts, .. } => &opts.client_opts,
            Command::Prefixes { client_opts, .. } | Command::Types { client_opts, .. } => {
                client_opts
            }
        }
    }

    fn out(&self) -> &Out {
        match self {
            Command::Works { opts, .. }
            | Command::Funders { opts, .. }
            | Command::Members { opts, .. }
            | Command::Journals { opts, .. } => &opts.out,
            Command::Prefixes { out, .. } | Command::Types { out, .. } => out,
        }
    }

    async fn run<W: Write>(&self, writer: W, client: &Crossref) -> crossref_client::Result<()> {
        match self {
            Command::Types { id, .. } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.type_(id).await?)?,
                None => serde_json::to_writer_pretty(writer, &client.types().await?)?,
            },
            Command::Prefixes { id, .. } => {
                serde_json::to_writer_pretty(writer, &client.prefix(id).await?)?
            }
            Command::Journals { id, opts } => match id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.journal(id).await?)?,
                None => {
                    // `/journals` supports neither sort nor sample, so those flags are ignored here
                    let query = JournalsQuery {
                        queries: opts.query_terms.clone(),
                        result_control: opts.paging(),
                    };
                    let journals = client.journals(query).await?;
                    serde_json::to_writer_pretty(writer, &journals)?
                }
            },
            Command::Members { opts } => match &opts.id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.member(id).await?)?,
                None => {
                    let query = apply_opts!(MembersQuery, opts);
                    serde_json::to_writer_pretty(writer, &client.members(query).await?)?
                }
            },
            Command::Funders { opts } => match &opts.id {
                Some(id) => serde_json::to_writer_pretty(writer, &client.funder(id).await?)?,
                None => {
                    let query = apply_opts!(FundersQuery, opts);
                    serde_json::to_writer_pretty(writer, &client.funders(query).await?)?
                }
            },
            Command::Works {
                opts,
                combined,
                deep_page,
            } => {
                if let Some(id) = &opts.id {
                    serde_json::to_writer_pretty(writer, &client.work(id).await?)?;
                    return Ok(());
                }

                let query = WorksQuery {
                    free_form_queries: opts.query_terms.clone(),
                    sort: opts.sort,
                    order: opts.order,
                    result_control: opts.result_control().map(WorkResultControl::Standard),
                    ..Default::default()
                };

                let query = match combined {
                    Some(Combined::Journal { id }) => {
                        query.into_combined_query::<crossref_client::Journals>(id)
                    }
                    Some(Combined::Type { id }) => {
                        query.into_combined_query::<crossref_client::Types>(id)
                    }
                    Some(Combined::Funder { id }) => {
                        query.into_combined_query::<crossref_client::Funders>(id)
                    }
                    Some(Combined::Member { id }) => {
                        query.into_combined_query::<crossref_client::Members>(id)
                    }
                    Some(Combined::Prefix { id }) => {
                        query.into_combined_query::<crossref_client::Prefixes>(id)
                    }
                    None => query.into(),
                };

                if *deep_page {
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
    let client = app.command.client_opts().create_client()?;

    let out = app.command.out();
    match &out.output {
        Some(path) => {
            let file = if out.append && path.exists() {
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
