---
name: crossref
description: Look up DOIs, references and publication metadata from the command line with the `crossref` CLI — resolve a DOI to its work, find the DOI behind a reference string or a bibtex citation key, check that a bibliography's citations are real, render a work as BibTeX, RIS or a formatted citation in any CSL style, and query journals, publishers, funders and licences. Use whenever a claim carries a DOI, whenever a citation or reference list has to be verified rather than trusted, whenever a bibliography needs DOIs or formatting, and whenever the question is what a journal or a publisher has registered.
---

# crossref

A single binary over the crossref REST api — the registry publishers deposit
their metadata into when they mint a DOI. It resolves DOIs, searches the
registered metadata, resolves bibtex citation keys to the works they cite, and
re-serializes a work into BibTeX, RIS or a formatted citation. It paces itself
against the budget crossref reports on every response and retries a `429`, so a
sequence of calls is safe to make.

## Before the first call

```shell
crossref --version
```

If it is not on `PATH`: inside a checkout of this repo use
`cargo run --features cli --` in place of `crossref`; otherwise
`cargo install crossref-client --features cli`, or take a prebuilt binary from
the releases page (see the repo README).

**Identify yourself on every call.** Anonymous requests share a pool of roughly
one request per second and start coming back `429`; an email moves you to
crossref's polite pool, which currently grants three.

```shell
crossref --polite you@example.com works ...
```

Ask the user for an address once, or reuse one already visible in the repo
(`git config user.email`). Everything below omits the flag for brevity — keep
it on.

## The one thing to know first

**Crossref answers every text query with something.** There is no such thing as
a miss:

```console
$ crossref works --query-title "zzqqxx nonexistent phrase kkjj" --limit 3
  "total-results": 7860,      # ...and the first item is titled "Nonexistent cycles"
```

`score` ranks the answers against each other, not against the truth, so the top
hit of a search is **never** evidence on its own that the thing you searched for
exists. Either check the returned `title`, `author` and `issued` against what
you were looking for, or let `cite` do that checking for you. A DOI lookup
(`--id`) is the only form that can genuinely fail, and it is the one to prefer
whenever a DOI is at hand.

## The commands

| Command | For |
|---|---|
| `crossref works [-i DOI] [flags]` | one work by DOI, or search the registered metadata |
| `crossref cite <KEY>...` | resolve bibtex citation keys to the works they cite, **checked** |
| `crossref transform <DOI> --format ...` | render a work as BibTeX, RIS, CSL JSON or a citation |
| `crossref journals [-i ISSN]` | journals, by ISSN or by name |
| `crossref members [-i ID]` | publishers |
| `crossref funders [-i ID]` | funding bodies |
| `crossref prefixes -i 10.1016` | who owns a DOI prefix |
| `crossref types` / `crossref licenses` / `crossref styles` | the vocabularies |

Global: `--polite`, `--token` (Plus), `--user-agent`, `-o <PATH>`, `-a`
(append). Output is pretty JSON everywhere except `transform`, which writes
whatever crossref rendered.

`works` also takes a subcommand naming what to search inside. **It ends the
command line** — every flag goes before it, or clap refuses the lot.

```shell
crossref works --query-title salmon member 78        # Elsevier's works
crossref works --limit 5 journal 0308-597X           # a journal's works
crossref works --limit 5 funder 501100005416         # works acknowledging a funder
crossref works ... prefix 10.1016   /   crossref works ... type dataset
```

## Recipes

**Resolve a DOI.** The one call that fails honestly when the thing is not there.

```shell
crossref works --id 10.1016/j.marpol.2021.104855
```

**Find the work behind a reference string.** `--query-bibliographic` is
crossref's own reference matching: it reads titles, authors, ISSNs and years out
of the string together and finds what the same words split across `--query`
would not.

```shell
crossref works --limit 1 \
  --query-bibliographic "Feynman, R. (1960). There's Plenty of Room at the Bottom. Engineering and Science, 23(5)."
```

Still check the answer — see the warning above.

**Check the citation keys in a bibliography**, which is the checked form of the
recipe above.

```shell
crossref cite @LindforsJakobsen2022 @Hopp_Coffay_Lindfors_2023
```

**Search by field.** `--query-title` and `--query-author` have their own flags;
every other field goes through `--query-field field=term`, repeatable.

```shell
crossref works --query-title "room at the bottom" --query-author feynman --limit 5
crossref works --query-author feynman --query-field container-title="Engineering and Science"
```

**Narrow to a window, a type, a journal.** Filters are ANDed, so a date range is
two of them.

```shell
crossref works --query-title salmon \
  --filter from-pub-date:2023-01-01 --filter until-pub-date:2023-12-31 \
  --filter type:journal-article --filter has-abstract \
  --sort issued --order desc --limit 20
```

The count that comes back (`total-results`) is the real number of registered
works matching the filters — that is the figure to quote when asked how much has
been published on something, with the caveat in *Verifying claims* below.

**Cite it.** `transform` goes straight to crossref's rendering, so nothing is
formatted here from a parsed `Work`.

```shell
crossref transform 10.1016/j.marpol.2021.104855 --format bibtex >> references.bib
crossref transform 10.1016/j.marpol.2021.104855 --format citation --style apa
crossref transform 10.1016/j.marpol.2021.104855 --format ris -o ref.ris
```

`--style` takes any of the ~2 900 names `crossref styles` lists; `--locale`
(e.g. `de-DE`) goes with it. Other formats: `citeproc-json`, `crossref-xml`,
`crossref-tdm` (carries the full-text links), `rdf-xml`, `turtle`.

**Walk a paper's reference list.** A single-work fetch carries `reference[]`
when the publisher deposited it, each entry often with its own `DOI`.

```shell
crossref works --id 10.1016/j.marpol.2021.104855 | jq -r '.reference[]?.DOI // empty'
```

**Who owns this prefix / what has this publisher registered.**

```shell
crossref prefixes --id 10.1016
crossref members --filter prefix:10.1016
crossref funders --filter location:Norway --limit 20
```

## Query semantics — the parts that surprise people

* **Every flag is `AND`ed.** There is no `OR` anywhere in the api. Two terms for
  the same field is an error rather than a request, because crossref keeps one
  of them and does not say which.
* **The queryable fields are these and no others**: `title`,
  `container-title`, `author`, `editor`, `chair`, `translator`, `contributor`,
  `bibliographic`, `affiliation`, `degree`, `description`, `event-acronym`,
  `event-location`, `event-name`, `event-sponsor`, `event-theme`, `funder-name`,
  `publisher-location`, `publisher-name`, `standards-body-acronym`,
  `standards-body-name`. Reachable as `--query-title`, `--query-author`,
  `--query-bibliographic`, or generically `--query-field name=term`. A misspelled
  one is refused with the whole list rather than sent.
* **A query term is a bag of words, not a phrase.** `--query-title "machine
  learning"` matches records with either word, ranked; there is no way to demand
  the phrase. Narrow with filters, not with quoting.
* **Filters are a separate vocabulary from fields** — about 90 of them on
  `/works`, as `name` for the ones asking whether a record has something
  (`has-abstract`, `has-orcid`, `has-references`) and `name:value` for the ones
  asking what it is (`type:journal-article`, `issn:0308-597X`, `member:78`,
  `funder:10.13039/501100005416`, `from-pub-date:2023-01-01`). A misspelled name
  or an unreadable value is caught before anything is sent, and lists what does
  exist.
* **A date filter needs a whole date.** `from-pub-date:2020` is refused; write
  `2020-01-01`.
* **A filter value cannot contain a `,`.** Crossref splits the `filter`
  parameter on it after decoding, so `container-title:Ecology, Evolution`
  arrives as two filters and there is no form that survives. Such a query is
  refused rather than misread.
* **`type:` values come from `crossref types`** — `journal-article`,
  `book-chapter`, `posted-content` (which is where preprints land), `dataset`,
  `peer-review`, `grant`, and about two dozen more.
* **`--sort`, `--order`, `--sample` and the `--query-*` flags are `/works`
  only.** The other routes answer them with a `400`, so they do not exist as
  flags there. `--filter` exists on `works`, `funders` and `members`, each with
  its own vocabulary — `location` belongs to `/funders` alone. `journals` and
  `licenses` narrow by nothing.
* **`--sample N` makes crossref ignore every other option**, filters included.

## Reading the output

`works` searching answers a page object: `total-results` (how many matched in
all), `items-per-page`, `query.start-index`, and `items[]`. `works --id` answers
the single `Work` unwrapped. `journals`, `members`, `funders`, `licenses` and
`styles` answer the same page shape around their own item type.

A `Work` carries `DOI`, `title[]`, `author[]` (`.family`, `.given`, `.ORCID`,
`.affiliation`), `container-title[]`, `publisher`, `type`, `issued`
(`date-parts`, the earliest known publication date), `published-print`,
`published-online`, `volume`, `issue`, `page`, `ISSN[]`, `abstract`,
`reference[]`, `license[]`, `funder[]`, `link[]`, `is-referenced-by-count`,
`references-count`, `score`, `URL`.

`abstract` is **absent far more often than not**, and when present it is JATS
XML (`<jats:p>…</jats:p>`) that needs the tags stripped before it is quoted.
A date is `{"date-parts": [[2022, 1, 15]]}`, with month and day sometimes
missing.

```shell
# with jq
crossref works --query-title salmon --limit 5 \
  | jq -r '.items[] | "\(.DOI)  \(.issued["date-parts"][0][0])  \(.title[0])"'
crossref cite @LindforsJakobsen2022 | jq -r '.[] | select(.verdict == "matched") | .doi'
```

```powershell
# PowerShell, no jq needed
$r = crossref works --query-title salmon --limit 5 | ConvertFrom-Json
$r.items | ForEach-Object { "{0}  {1}" -f $_.DOI, $_.title[0] }
```

## Citations — which direction this reaches

**Backward (what a work cites) is here.** A single-work fetch carries
`reference[]` when the publisher deposited it, most entries with their own
`DOI`; `references-count` says how many, and `--filter has-references` narrows
to the works that have any.

**Forward (who cites a work) is a count and nothing more.**
`is-referenced-by-count` is crossref's own citation count, and there is no
filter or query on this api that turns it into a list — the authoritative
filter list `/works` returns has no `reference.*` in it. So "which works cite
this DOI" cannot be asked here, in any spelling.

It *can* be asked of crossref, just not through this api.
[Cited-by](https://www.crossref.org/documentation/cited-by/retrieve-citations/)
is a separate service that answers exactly that, returning the citing works as
XML from a different host:

```text
https://doi.crossref.org/servlet/getForwardLinks?usr=<user>&pwd=<password>&doi=<doi>
```

It needs Crossref **member** credentials — an account a publisher who deposits
metadata has and a reader does not — so this CLI does not speak it. If the user
has such an account, that URL with `curl` is the whole of it.

**The open route is [OpenCitations](https://opencitations.net)**, which inverts
the same open references crossref publishes into a citation index — CC0, no
account, over two billion citations. If the `opencitations` CLI is on `PATH`,
that is the tool for this direction, and it pairs with this one:

```shell
# the works citing a paper, then each one's full crossref record
opencitations citations doi:10.1016/j.marpol.2021.104855 --since 2024 -f dois \
  | while read doi; do crossref works --polite you@example.com --id "$doi"; done
```

Its coverage is a floor rather than a total — the index only holds what
publishers opened — so read its skill before quoting a count. OpenAlex, Semantic
Scholar and NIH-OCC answer the same question from overlapping data. Reach for
one of these rather than trying to reconstruct a citation graph by crawling
`reference[]` across crossref.

**`relation.type` is not the citation graph.** It carries the relationships a
publisher deposited — `is-review-of` (1.3M works), `is-preprint-of` (830k),
`has-preprint` (564k), `is-supplement-to` (229k) — and while `references` is in
the vocabulary, only 37k works in all of crossref carry it, against 123M
registered journal articles. Useful for finding a preprint's published version
or a paper's open reviews; useless as a citation index.

```shell
crossref works --filter relation.type:is-preprint-of --filter has-abstract --limit 5
```

## Resolving citation keys

`cite` takes the keys a bibliography is written in and finds the works they
stand for. A key carries only surnames and a year, and crossref answers anything
— so each candidate is checked against the key before it is reported: the
surnames have to be credited in the order the key gives them, from the first
author, and the year has to fall inside `--year-window` (one either way by
default, since a work published online in one year and in an issue the next is
cited by both).

| verdict | what it means |
|---|---|
| `matched` | one work, and the key vouches for it. `doi` and the whole `work` are reported |
| `ambiguous` | several works the key vouches for, nothing in the key to choose between them. All listed under `candidates` |
| `unmatched` | nothing the key vouches for. The nearest misses are listed with `mismatches` saying why each was turned down |

`matched-as` appears when the key's diacritics had to be guessed back —
`Floysand` reaches `Fløysand` no other way, since keys are ascii and crossref
folds nothing. Each guess costs a request; `--spellings` caps how many (12 by
default, `0` for none). `--candidates` sets how many works are weighed per
attempt.

`cite` exits `0` only when **every** key resolved to exactly one work, so a
bibliography can be checked in a script; the report is written either way.

```shell
grep -o '@[a-z]*{[^,]*' refs.bib | cut -d'{' -f2 | xargs crossref cite \
  || echo "some keys did not resolve"
```

## Exit status and errors

A search that matched nothing still exits `0` — read `total-results` and
`items`, and remember it is almost never zero.

Everything that failed exits `1` with `Error: <the error>` on stderr:

| Error | What it means |
|---|---|
| `ResourceNotFound { resource: ... }` | no such DOI / ISSN / member / funder id |
| `ValidationFailure { failures: ... }` | crossref refused the request and said why |
| `RateLimited { attempts, limit }` | the retry budget ran out; something else is querying alongside |
| `UnsendableFilterValue { .. }` | a filter value carrying a `,` (see above) |
| `UnexpectedItem { expected, got }` | crossref answered with a different message type than the route promises |
| `ReqWest { reqwest: ... Status(400 ...) }` | crossref refused without saying why — an offset past 10 000, most often |

A bad flag is caught by the parser instead, before any request, and prints the
vocabulary it expected.

## Rate, paging and scale

* The limiter lives inside one process. **Do not run several `crossref` calls in
  parallel** — separate processes do not share it, and the pool is one or three
  requests a second in total.
* A page is 20 works by default; `--limit` asks for more, `--offset` skips
  ahead. Crossref caps `rows` at 1000 (`--limit 1001` comes back
  `integer-not-valid`) and refuses an offset past 10 000 with a bare `400`,
  which surfaces as a `ReqWest ... Status(400)` rather than a message — beyond
  that a cursor is the only way through.
* `--deep-page` crawls the whole result set with a cursor, one request per page.
  **Setting `--limit` alongside it turns the crawl off** and returns a single
  page — the limit wins, as the help says. So a crawl is `--deep-page` with no
  `--limit`, and it is worth narrowing hard with filters first: 100 000 works is
  5 000 requests, which is half an hour in the polite pool.
* `--deep-page` prints a flat JSON array of `Work` rather than the page object,
  since it has joined the pages already.
* Prefer `-o out.json` over shell redirection for a long crawl.

## Verifying claims with this

* **Crossref is a DOI registry, not an index of the literature.** A record means
  a member deposited metadata when they minted the DOI. It is not evidence of
  peer review, and it says nothing about whether the work was later retracted —
  that lives in `update-to` / `relation`, and only when the publisher deposited
  it.
* **Absence proves less than it looks.** Not every DOI is a crossref DOI
  (datasets are often DataCite), preprints only appear once someone mints one,
  and older or small-publisher material may never have been registered. Say
  which queries you ran before concluding a work does not exist.
* **The metadata is the publisher's, not crossref's.** `type`,
  `container-title`, dates and author lists are as deposited: `issued` can be a
  year alone, an author list can be truncated, `abstract` is usually missing.
  Quote what the record says, not what you expect it to say.
* **`is-referenced-by-count` undercounts.** It counts citations from other
  crossref works whose publishers deposited their reference lists, so it sits
  below Scopus or Google Scholar and lags behind them. Never report it as *the*
  citation count, and see *Citations — which direction this reaches* before
  promising anyone a list of citing works.
* **Cite what you checked**: the DOI, and `https://doi.org/<DOI>` as the link.
  Say whether you resolved the DOI or matched a search result, and if the latter,
  what you checked it against.
