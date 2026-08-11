Crossref-rs - A rust client for the Crossref-API
=====================
[![Crates.io](https://img.shields.io/crates/v/crossref-rs.svg)](https://crates.io/crates/crossref-rs)
[![Documentation](https://docs.rs/crossref-rs/badge.svg)](https://docs.rs/crossref-rs)


[Crossref API docs](https://github.com/CrossRef/rest-api-doc)

This client is inspired by [sckott/habanero](https://github.com/sckott/habanero/).


`Crossref` - Crossref search API. The `Crossref` crate provides methods matching Crossref API routes:

* `works` - `/works` route
* `members` - `/members` route
* `prefixes` - `/prefixes` route
* `funders` - `/funders` route
* `journals` - `/journals` route
* `types` - `/types` route
* `agency` - `/works/{doi}/agency` get DOI minting agency


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

Anonymous clients share a pool limited to roughly one request per second and will
start returning `429`. Passing an email moves you to the polite pool, which
currently allows several requests per second. To get into it, include an email address

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
use crossref_rs::*;
async async fn run() -> Result<()> {
    let client = Crossref::builder().build()?;
    let works = client.member_works(WorksQuery::new("machine learning")
        .sort(Sort::Score).into_ident("member_id")).await?;
    Ok(())
}
```

This would be the same as using the [`Crossref::works`] method by supplying the combined type

```rust
use crossref_rs::*;
async async fn run() -> Result<()> {
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
use crossref_rs::{AsyncIterator, Crossref, WorksQuery, Work};
async fn run() -> Result<(), crossref_rs::Error> {
    let client = Crossref::builder().build()?;

    let mut pages = client.deep_page(WorksQuery::new("Machine Learning"));
    let mut all_works: Vec<Work> = Vec::new();
    while let Some(page) = pages.next().await {
        all_works.extend(page.items);
    }

    Ok(())
}
```

Which can be simplified to
```rust
use crossref_rs::{AsyncIterator, Crossref, WorksQuery, Work};
async fn run() -> Result<(), crossref_rs::Error> {
    let client = Crossref::builder().build()?;

    let mut works = client.deep_page("Machine Learning").into_work_iter();
    while let Some(work) = works.next().await {
        println!("{}", work.doi);
    }

    Ok(())
}
```


Iterate over all the pages (`WorkList`) of the funder with id `funder id` by using a combined query.
A single `WorkList` usually holds 20 `Work` items.

```rust
use crossref_rs::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
async fn run() -> Result<(), crossref_rs::Error> {
    let client = Crossref::builder().build()?;

    let mut pages = client.deep_page(
        WorksQuery::default().into_combined_query::<Funders>("funder id"),
    );
    let mut all_funder_work_list: Vec<WorkList> = Vec::new();
    while let Some(page) = pages.next().await {
        all_funder_work_list.push(page);
    }

    Ok(())
}
```

Iterate over all `Work` items of a specfic funder directly.

```rust
use crossref_rs::{AsyncIterator, Crossref, Funders, WorksQuery, Work, WorkList};
async fn run() -> Result<(), crossref_rs::Error> {
    let client = Crossref::builder().build()?;

    let mut works = client.deep_page(
        WorksQuery::default().into_combined_query::<Funders>("funder id"),
    ).into_work_iter();
    let mut all_works: Vec<Work> = Vec::new();
    while let Some(work) = works.next().await {
        all_works.push(work);
    }

    Ok(())
}
```


## Command Line Application

### Installation
```shell
cargo install crossref-rs --features cli
```

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

## License

Licensed under either of these:

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)
   
