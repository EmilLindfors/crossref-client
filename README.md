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
  works      Query crossref works
  cite       Resolve bibtex citation keys, e.g. `@LindforsJakobsen2022`, to the works they cite
  funders    Query crossref funders
  members    Query crossref members
  journals   Query crossref journals
  licenses   List the licenses crossref works are published under
  prefixes   Query crossref prefixes
  types      Query crossref types
  transform  Re-serialize a work into another format through content negotiation
  styles     List the CSL styles a citation can be rendered in
  help       Print this message or the help of the given subcommand(s)

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
  -i, --id <ID>                         The DOI of a single work. Omit to search by query terms
      --user-agent <USER_AGENT>         The user agent to use for the crossref client
  -d, --deep-page                       Enable deep paging. If a limit is set, then the limit takes priority
      --token <TOKEN>                   The token to use for the crossref client
      --polite <POLITE>                 The email to use to get into crossref's polite pool
  -q, --query <QUERY_TERMS>             The free form terms for the query
  -l, --limit <LIMIT>                   limit the amount of results
  -o, --output <OUTPUT>                 output path where the results shall be stored
  -a, --append                          if the output file already exists, append instead of overwriting the file
      --offset <OFFSET>                 Sets an offset where crossref begins to retrieve items
      --query-bibliographic <CITATION>  Match a whole reference against titles, authors, ISSNs and publication years at once. Crossref's own citation look up
      --query-title <TERM>              Match against a work's title, including its subtitle
      --query-author <TERM>             Match against author given and family names
      --query-field <FIELD=TERM>        Match against any other field, e.g. `--query-field container-title=Nature`. Repeat for more fields; an unknown field lists every one crossref takes
      --filter <NAME[:VALUE]>           Narrow the results, e.g. `--filter from-pub-date:2020-01-01` or `--filter has-abstract`. Repeat to narrow further; every filter is ANDed
      --sort <SORT>                     How to sort the results, such as updated, indexed, published, issued
      --order <ORDER>                   How to order the results: asc or desc
      --sample <SAMPLE>                 Request random works. Crossref ignores every other option when set
```

The `cite` subcommand
```text
Usage: crossref cite [OPTIONS] <KEY>...

Arguments:
  <KEY>...  The citation keys, with or without their leading `@`

Options:
      --candidates <CANDIDATES>  How many works to weigh against the key per attempt [default: 20]
      --year-window <YEARS>      How many years either side of the key's year still count as the same publication [default: 1]
      --spellings <SPELLINGS>    How many accented spellings to guess at when the key as written finds nothing [default: 12]
      --user-agent <USER_AGENT>  The user agent to use for the crossref client
      --token <TOKEN>            The token to use for the crossref client
      --polite <POLITE>          The email to use to get into crossref's polite pool
  -o, --output <OUTPUT>          output path where the results shall be stored
  -a, --append                   if the output file already exists, append instead of overwriting the file
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

Query with paging

```shell
crossref <works|funders|members> --query "machine learning" --limit 10 --offset 200
```

Sorting and ordering are `works` only; the other routes answer them with a `400`.

```shell
crossref works --query "machine learning" --limit 10 --sort issued --order asc
```

Look a reference up by its citation. `--query-bibliographic` is crossref's own
reference matching: it reads titles, authors, ISSNs and publication years out of
the string together, and finds the work where the same words split across
`--query` would not.

```shell
crossref works --limit 1 \
  --query-bibliographic "Feynman, R. (1960). There's Plenty of Room at the Bottom. Engineering and Science, 23(5)."
```

Match single fields instead of the whole record. `--query-title` and
`--query-author` are the two that have their own flags; every other field
`/works` takes goes through `--query-field field=term`, which can be repeated.

```shell
crossref works --query-title "room at the bottom" --query-author feynman
crossref works --query-author feynman --query-field container-title="Engineering and Science"
```

A field can only be matched against one term, so asking for the same one twice
is an error rather than a silently dropped flag.

Narrow what comes back with `--filter`, which takes any of the 90 filters
`/works` accepts, as `name` for the ones that ask whether a record has
something and `name:value` for the ones that ask what it is. Repeat the flag to
narrow further; crossref ANDs them.

```shell
crossref works --query-title salmon \
  --filter from-pub-date:2023-01-01 --filter until-pub-date:2023-12-31 \
  --filter has-abstract --filter type:journal-article
```

The name and the value are both read before anything is sent, so a misspelled
filter lists the ones that exist and `--filter from-pub-date:2020` says that a
year is not a date, rather than crossref answering with a `400` or, worse, an
unnarrowed result set.

`funders` and `members` take `--filter` too, each narrowing by its own
vocabulary -- `location` belongs to `/funders` alone, and `/works` answers it
with a `400`. `journals` and `licenses` narrow by nothing, so they offer no
such flag.

```shell
crossref funders --filter location:Norway
crossref members --filter prefix:10.1016 --filter current-doi-count:1000
```

A filter belonging to another route is refused with the ones this route does
take:

```console
$ crossref funders --filter has-abstract
error: invalid value 'has-abstract' for '--filter <NAME[:VALUE]>':
`has-abstract` is not a filter this route takes. Try one of: location
```

### Resolving citation keys

`cite` takes the bibtex keys a bibliography is written in and finds the works
they stand for.

```shell
crossref cite @LindforsJakobsen2022 @Hopp_Coffay_Lindfors_2023
```

A key carries no title, only surnames and a year, so crossref cannot be trusted
to have returned the right work -- it answers every query with something. Each
candidate is therefore checked against the key before it is reported: the key's
surnames have to be credited, in the order the key gives them, starting at the
first author, and the year has to be within `--year-window` (one year either
way by default, since a work published online in one year and in an issue the
next is cited by both).

The verdict says how much the answer is worth.

| verdict | what it means |
| --- | --- |
| `matched` | one work, and the key vouches for it. `doi` and the whole `work` are reported |
| `ambiguous` | several works the key vouches for, and nothing in the key to choose between them. All of them are listed |
| `unmatched` | nothing the key vouches for. The nearest misses are listed with the reason each was turned down |

```json
[
  {
    "key": "LindforsJakobsen2022",
    "surnames": ["Lindfors", "Jakobsen"],
    "year": 2022,
    "et-al": false,
    "verdict": "matched",
    "requests": 1,
    "doi": "10.1016/j.marpol.2021.104855",
    "work": { "...": "the whole work" }
  }
]
```

So the DOI of a key is

```shell
crossref cite @LindforsJakobsen2022 | jq -r '.[] | select(.verdict == "matched") | .doi'
```

`cite` exits `0` only when every key resolved to exactly one work, so a
bibliography can be checked in a script. The report is written either way -- a
key that resolved to nothing, or to several works, is an answer rather than an
error.

```shell
# every key in a .bib, checked against crossref
grep -o '@[a-z]*{[^,]*' refs.bib | cut -d'{' -f2 | xargs crossref cite --polite you@example.com \
  || echo "some keys did not resolve"
```

#### Keys that lost their diacritics

Citation keys are written in ascii and crossref folds nothing, so
`query.author=Floysand` finds the works of a different person and
`query.author=Fløysand` is the only way to reach `@FloysandEtAl2021`. When the
key as written finds nothing, `cite` guesses the marks back one letter at a
time, likeliest first, and reports the spelling that worked as `matched-as`.

```shell
crossref cite @FloysandEtAl2021        # three requests: as written, as a citation, then Fløysand
```

Each guess costs a request, so `--spellings` caps how many are made (twelve by
default, `0` to make none). Verification folds both spellings together, so a key
written either way is checked against metadata written either way.

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
   
