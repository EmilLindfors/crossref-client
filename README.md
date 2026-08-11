crossref-client - An async Rust client for the Crossref REST API
=====================
[![Crates.io](https://img.shields.io/crates/v/crossref-client.svg)](https://crates.io/crates/crossref-client)
[![Documentation](https://docs.rs/crossref-client/badge.svg)](https://docs.rs/crossref-client)

[Crossref API docs](https://api.crossref.org/swagger-ui/index.html)

> **Fork notice.** `crossref-client` is a hard fork of
> [MattsSe/crossref-rs](https://github.com/MattsSe/crossref-rs), which is no
> longer maintained. It has diverged substantially — the client is async,
> targets Rust 2024, and carries breaking changes to the query and response
> types. Changes here are **not** upstreamed. The original work is retained
> under its MIT/Apache-2.0 licence; see [LICENSE-MIT](LICENSE-MIT).
>
> Upstream's design is in turn inspired by
> [sckott/habanero](https://github.com/sckott/habanero/).

The `Crossref` client provides methods matching the Crossref API routes:

* `works` - `/works` route
* `members` - `/members` route
* `prefixes` - `/prefixes` route
* `funders` - `/funders` route
* `journals` - `/journals` route
* `licenses` - `/licenses` route
* `types` - `/types` route
* `styles` - `/styles` route, the CSL styles a citation can be rendered in
* `agency` - `/works/{doi}/agency` get DOI minting agency
* `transform` - `/works/{doi}/transform`, content negotiation

The client paces itself against the rate limit crossref reports on every
response and retries a `429`, so a `Crossref` -- and every clone of it, which
shares the limiter -- stays inside the budget it was granted.


## Install

### As a library

```shell
cargo add crossref-client
```

### As a command line tool

Every release carries a prebuilt `crossref` binary, so this needs no rust
toolchain. Each archive holds the binary and the two licences, and is named
after the platform alone, so these commands always fetch the newest release.

**Linux and macOS**

```shell
# x86_64-unknown-linux-gnu | aarch64-apple-darwin (apple silicon) | x86_64-apple-darwin (intel)
target=x86_64-unknown-linux-gnu

curl -fsSL "https://github.com/EmilLindfors/crossref-client/releases/latest/download/crossref-$target.tar.gz" | tar xz
sudo install "crossref-$target/crossref" /usr/local/bin/
```

The binary is unsigned. Downloaded with `curl` it runs as it is; downloaded
through a browser, macOS quarantines it until
`xattr -d com.apple.quarantine /usr/local/bin/crossref`.

**Windows** (PowerShell)

```powershell
$target = "x86_64-pc-windows-msvc"

Invoke-WebRequest "https://github.com/EmilLindfors/crossref-client/releases/latest/download/crossref-$target.zip" -OutFile crossref.zip
Expand-Archive crossref.zip -DestinationPath .
# then move crossref-$target\crossref.exe somewhere on your PATH
```

**From source**, which needs rust 1.85 or newer:

```shell
cargo install crossref-client --features cli
```

Either way, `crossref --version` says which release you ended up with. What
the binary can do is under [Command Line Application](#command-line-application).


## Usage

### Create a `Crossref` client:

```rust
let client = Crossref::builder().build()?;
```

If you have an [Authorization token for Crossref's Plus service](https://github.com/CrossRef/rest-api-doc#authorization-token-for-plus-service):

```rust
let client = Crossref::builder()
    .token("token")
    .build()?;
```

Encouraged to use the **The Polite Pool**:

[Good manners = more reliable service](https://github.com/CrossRef/rest-api-doc#good-manners--more-reliable-service)

Anonymous clients share a pool limited to one request per second and will start
returning `429`. Passing an email moves you to the polite pool, which currently
allows three per second. To get into it, include an email address

```rust
let client = Crossref::builder()
     .polite("polite@example.com")
     .token("your token")
     .build()?;
```

### Constructing Queries
Not all components support queries and there are custom available parameters for each route that supports querying.
For each resource components that supports querying there exist a Query struct: `WorksQuery`, `MembersQuery`, `FundersQuery`. The `WorksQuery` also differs from the others by supporting [deep paging with cursors](https://github.com/CrossRef/rest-api-doc#deep-paging-with-cursors) and [field queries](https://github.com/CrossRef/rest-api-doc#works-field-queries). 

otherwise creating queries works the same for all resource components:

```rust

let query = WorksQuery::new("Machine Learning")
    // field queries supported for `Works`
    .field_query(FieldQuery::author("Some Author"))
    // filters are specific for each resource component
    .filter(WorksFilter::HasOrcid)
    .order(Order::Asc)
    .sort(Sort::Score);
```

Note that `sort`, `order`, `facet`, `select` and `sample` are `/works`-only:
`/funders`, `/members`, `/journals` and `/licenses` answer them with a `400`,
so `FundersQuery` and the rest offer terms, paging and -- where the route takes
one -- a filter, and nothing that cannot be sent.

Two more things a query cannot ask for, for reasons on crossref's side:

* A filter value cannot contain a `,`. Crossref splits the `filter` parameter
  on it *after* percent-decoding, so `container-title:Ecology, Evolution`
  arrives as one filter plus another called ` Evolution`, and neither the
  encoded nor the literal form survives. Such a query is refused with
  `Error::UnsendableFilterValue` rather than sent to be misread.
* `select` always includes `DOI`, whether or not it was asked for. It is the
  one field a `Work` requires, so a page selected without it would come back as
  works that cannot be deserialized at all.

### Other formats

Crossref will re-serialize a registered work, so a DOI can be pulled out as
BibTeX, RIS, RDF or a citation formatted in any of the ~2 900
[CSL styles](https://citationstyles.org) without going through `Work`:

```rust
let bibtex = client.transform(doi, &CnFormat::BibTex).await?;
let citation = client.transform(doi, &CnFormat::bibliography("apa")).await?;
```

### Examples

* [`examples/peer_review.rs`](examples/peer_review.rs) reconstructs a paper's
  open peer review history from the reviews crossref registers against it.
* [`examples/check_pool.rs`](examples/check_pool.rs) reports which rate-limit
  pool a client lands in.


### Get Records

See [this table](https://github.com/CrossRef/rest-api-doc#resource-components) for a detailed overview of the major components.

There are 3 different targets:

* **standalone resource components**: `/works`, `/members`, `funders`, `prefixes`, `types` that return a list list of the corresponding items and can be specified with queries
* **Resource component with identifiers**: `/works/{doi}?<query>`,`/members/{member_id}?<query>`, etc. that returns a single item if found.
* **combined with the `works` route**: The works component can be appended to other resources: `/members/{member_id}/works?<query>` etc. that returns a list of matching `Work` items as `WorkList`.

This resembles in the enums of the resource components, eg. for `Members`:

```rust
pub enum Members {
    /// target a specific member at `/members/{id}`
    Identifier(String),
    /// target all members that match the query at `/members?query...`
    Query(MembersQuery),
    /// target a `Work` for a specific member at `/members/{id}/works?query..`
    Works(WorksIdentQuery),
}
```

### Examples

All options are supported by the client:

**Query Single Item by DOI or ID**

Analogous methods exist for all resource components

```rust
let work = client.work("10.1037/0003-066X.59.1.29").await?;

let agency = client.work_agency("10.1037/0003-066X.59.1.29").await?;

let funder = client.funder("funder_id").await?;

let member = client.member("member_id").await?;
```

**Query**

```rust
let query = WorksQuery::new("Machine Learning");

// one page of the matching results
let works = client.works(query).await?;
```

Alternatively insert a free form query term directly

```rust
let works = client.works("Machine Learning").await?;
```

 **Combining Routes with the `Works` route**

For each resource component other than `Works` there exist methods to append a `WorksQuery` with the ID option `/members/{member_id}/works?<query>?`

```
use crossref_client::*;
async fn run() -> Result<()> {
    let client = Crossref::builder().build()?;
    let works = client.member_works(WorksQuery::new("machine learning")
        .sort(Sort::Score).into_ident("member_id")).await?;
    Ok(())
}
```

This would be the same as using the [`Crossref::works`] method by supplying the combined type

```rust
use crossref_client::*;
async fn run() -> Result<()> {
    let client = Crossref::builder().build()?;
    let works = client.works(WorksQuery::new("machine learning")
        .sort(Sort::Score)
        .into_combined_query::<Members>("member_id")).await?;
    Ok(())
}
```

** Deep paging for `Works` **
[Deep paging results](https://github.com/CrossRef/rest-api-doc#deep-paging-with-cursors)
Deep paging is supported for all queries, that return a list of `Work`, `WorkList`.
This function returns a new iterator over pages of `Work`, which is returned as bulk of items as a `WorkList` by crossref.
Usually a single page `WorkList` contains 20 items.

Example

Iterate over all `Works` linked to search term `Machine Learning`

```rust
use crossref_client::{AsyncIterator, Crossref, WorksQuery, Work};
async fn run() -> Result<(), crossref_client::Error> {
    let client = Crossref::builder().build()?;

    let mut pages = client.deep_page(WorksQuery::new("Machine Learning"));
    let mut all_works: Vec<Work> = Vec::new();
    while let Some(page) = pages.next().await {
        let page = page?;
        all_works.extend(page.items);
    }

    Ok(())
}
```

Which can be simplified to
```rust
use crossref_client::{AsyncIterator, Crossref, WorksQuery, Work};
async fn run() -> Result<(), crossref_client::Error> {
    let client = Crossref::builder().build()?;

    let mut works = client.deep_page("Machine Learning").into_work_iter();
    while let Some(work) = works.next().await {
        let work = work?;
        println!("{}", work.doi);
    }

    Ok(())
}
```


Iterate over all the pages (`WorkList`) of the funder with id `funder id` by using a combined query.
A single `WorkList` usually holds 20 `Work` items.

```rust
use crossref_client::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
async fn run() -> Result<(), crossref_client::Error> {
    let client = Crossref::builder().build()?;

    let mut pages = client.deep_page(
        WorksQuery::default().into_combined_query::<Funders>("funder id"),
    );
    let mut all_funder_work_list: Vec<WorkList> = Vec::new();
    while let Some(page) = pages.next().await {
        let page = page?;
        all_funder_work_list.push(page);
    }

    Ok(())
}
```

Iterate over all `Work` items of a specfic funder directly.

```rust
use crossref_client::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
async fn run() -> Result<(), crossref_client::Error> {
    let client = Crossref::builder().build()?;

    let mut works = client.deep_page(
        WorksQuery::default().into_combined_query::<Funders>("funder id"),
    ).into_work_iter();
    let mut all_works: Vec<Work> = Vec::new();
    while let Some(work) = works.next().await {
        let work = work?;
        all_works.push(work);
    }

    Ok(())
}
```


## Command Line Application

A prebuilt binary per platform, or `cargo install crossref-client --features cli`
-- see [Install](#as-a-command-line-tool).

### Usage

Top level subcommands
```text
Usage: crossref <COMMAND>

Commands:
  works     Query crossref works
  funders   Query crossref funders
  members   Query crossref members
  journals  Query crossref journals
  prefixes  Query crossref prefixes
  types     Query crossref types
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

The `works` subcommand
```text
Usage: crossref works [OPTIONS] [COMMAND]

Commands:
  member   Get Works of a specific Member
  funder   Get Works of a specific Funder
  journal  Get Works of a specific Journal
  prefix   Get Works of a specific Prefix
  type     Get Works of a specific Type

Options:
  -d, --deep-page                Enable deep paging. If a limit is set, then the limit takes priority
  -o, --output <OUTPUT>          output path where the results shall be stored
  -a, --append                   if the output file already exists, append instead of overwriting the file
  -l, --limit <LIMIT>            limit the amount of results
  -i, --id <ID>                  The id of component
  -q, --query <QUERY_TERMS>      The free form terms for the query
      --sort <SORT>              How to sort the results, such as updated, indexed, published, issued
      --order <ORDER>            How to order the results: asc or desc
      --sample <SAMPLE>          Request random elements. Overrides all other options
      --offset <OFFSET>          Sets an offset where crossref begins to retrieve items
      --user-agent <USER_AGENT>  The user agent to use for the crossref client
      --token <TOKEN>            The token to use for the crossref client
      --polite <POLITE>          The email to use to get into crossref's polite pool
```

### Examples

Be polite: pass your email so requests are routed to crossref's polite pool
(anonymous requests are limited to roughly 1 request/second and return `429`).

```shell
crossref works --polite you@example.com --limit 10 --query "machine learning"
```

Get a single work by DOI

```shell
crossref works --id "10.1037/0003-066X.59.1.29"
```

Store the output in a file instead of printing to stdout

```shell
crossref works --id "10.1037/0003-066X.59.1.29" -o output.json
```

This works for every subcommand

```shell
crossref <works|journals|members|prefixes|types> --id "10.1037/0003-066X.59.1.29" -o output.json
```

Query with paging and ordering

```shell
crossref <works|funders|members> --query "machine learning" --limit 10 --offset 200 --order asc
```

Get the works of a specific member

```shell
crossref works member 98
```

By default deep paging is disabled, so at most a single crossref page (20 `Work` items)
is returned. Pass `--deep-page` to page through the whole result set with a cursor.

```shell
crossref works --deep-page --query "machine learning" -o all.json
```

## Where the api's own behaviour is written down

When a route answers something this crate did not expect, these are the places
to look before assuming the bug is here:

* [The swagger UI](https://api.crossref.org/swagger-ui/index.html) — the
  routes and their parameters, as the api reports them today.
* [`CrossRef/rest-api-doc`](https://github.com/CrossRef/rest-api-doc) — the
  prose documentation, including the
  [work record format](https://github.com/CrossRef/rest-api-doc/blob/master/api_format.md)
  every response type here is modelled on.
* [Crossref's public issue board](https://crossref.atlassian.net/jira/software/c/projects/CR/list/?jql=project%20%3D%20CR%20ORDER%20BY%20cf%5B10019%5D%20ASC)
  — the `CR` project, open to read without an account. Api bugs, indexing
  gaps and in-flight changes are tracked here, so a filter that stopped
  matching or a field that changed shape usually has a ticket before it has an
  explanation anywhere else.

The live suite (`cargo test --test integration`) asks the api the same
questions and is the fastest way to tell a change on crossref's side from a
change on this one.

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
   
