# TODO

Follow-up work after the Rust 2024 / dependency modernization (`1129c29`).

Status: builds clean on Rust 1.96, clippy and rustdoc are warning-free, 84 tests
pass (39 unit, 32 doc, 7 integration, 6 response-shape).

Section 1 (correctness) is done, along with 3.4. Items are otherwise ordered by
impact.

---

## 1. Correctness — done

### 1.1 Percent-encode query parameters — **done**

`src/query/encode.rs` now percent-encodes every key and value, and
`CrossrefRoute` implementations build routes from key/value pairs rather than by
concatenation. `WorksQuery::new("R&D")` produces `/works?query=R%26D`; before it
produced `/works?query=R&D`, which crossref read as `query=R` plus a stray `D`
parameter and answered with the results for `R`.

The encode set keeps `:` `,` `/` `*` literal, because crossref's own filter and
cursor syntax is built from them and the api percent-decodes before it splits.
Both forms were checked against the live api and return identical results.

Note this means a `,` inside a filter *value* still can't be escaped — crossref
splits after decoding, so neither form survives. That is an api limitation, not
a client one.

### 1.2 `param_key` returns more than a key — **done**

`CrossrefQueryParam` now has a single method, `params(&self) -> Vec<(Cow<str>,
Cow<str>)>`. `ResultControl::RowsOffset` and `WorkResultControl::Cursor { rows }`
return two pairs instead of smuggling a second parameter through `param_key`.

### 1.3 Deep paging silently swallows every error — **done**

`WorkListIterator::Item` is `Result<WorkList>` and `WorkIterator::Item` is
`Result<Work>`. A failed page yields the error and then ends the iteration, so a
truncated crawl can no longer be mistaken for a clean finish.

### 1.4 Response parsing panics instead of erroring — **done**

The 18 hand-written `TryFrom<serde_json::Value>` impls in `src/response/work.rs`
(1256 lines, 54 `unwrap`s) were deleted in favour of the serde derives those
types already carried, following what was done for `Journal`. `Work` and
`WorkList` keep a `TryFrom` that delegates to `serde_json::from_value`.
`Message::try_from` no longer discards the underlying serde error with
`map_err(|_e| …)`, so a parse failure now names the field that failed.

Replacing the parsers surfaced six shapes the response types rejected outright.
Each was found by parsing 37 600 works sampled from the live api — a mix of
uniform `sample=100` draws and per-type deep pages — and each is now covered by
a fixture in `tests/work_shapes.rs`:

| shape | frequency |
| --- | --- |
| work with no `title` | ~5% (most `component` records) |
| `content-domain` with no `crossmark-restriction` | ~6% |
| work with no `publisher` | ~1 in 5 000 |
| `funder` entry with no `name` | ~1 in 2 000 |
| work with no `member` | ~1 in 6 000 |
| work with no `type` | ~1 in 20 000 |
| `license` with no `start` | ~1 in 3 000 |
| bare `{}` `affiliation` | ~1 in 1 000 |
| `assertion.explanation` as `{"URL": …}` rather than a string | rare |

Every one of these used to panic the caller's task (`WorkList::try_from` did
`Work::try_from(v.clone()).unwrap()` per item), and after 1.3 would have cost a
whole page of a crawl.

The conclusion is that crossref validates member deposits loosely enough that no
field is guaranteed, so the member-deposited fields on `Work` are now `Option`.
`created` and `indexed` are `Option` as well — not for data quality, but because
`select` returns only the fields asked for (see 1.5). `explanation` became an
untagged `Explanation` enum covering both shapes.

### 1.5 `WorksQuery::elements` could never have worked — **done**

`WorkElement` was not re-exported from the crate root, so `elements()` was
unreachable from outside the crate; and `select` responses omit every field that
was not selected, which `Work` required. Both are fixed and
`selected_elements_narrow_the_response` covers it against the live api.

`Work::doi` is still required. A `select` that omits `DOI` therefore fails to
parse. Modelling that properly means a separate partial-work type rather than
making the primary key optional — worth doing if anyone needs it.

### 1.6 Confirm the `Explanation::Text` variant

`Assertion::explanation` was typed `Option<String>` but the only occurrence in
37 600 sampled works was a `{"URL": …}` object, so it is now an untagged enum
carrying both. The `Text` variant is inferred from the original type, not
observed — either find a record that uses it or drop the variant and make
`Explanation` a plain struct.

---

## 2. Crossref API coverage

Coverage was measured against the live API (it enumerates valid filters in the
`400` body for an unknown filter, and valid sort fields for an unknown sort).

### 2.1 Missing `/works` filters — 26 of 90

`WorksFilter` implements 66. Missing:

```
clinical-trial-number   from-approved-date      from-awarded-date
from-event-end-date     from-event-start-date   from-issued-date
funder-doi-asserted-by  group-title             gte-award-amount
has-affiliation-ror-id  has-alias               has-award
has-event               has-funder-doi          has-funder-ror-id
has-prime-doi           has-ror-id              has-update
lte-award-amount        ror-id                  until-approved-date
until-awarded-date      until-event-end-date    until-event-start-date
until-issued-date       update-type
```

The ROR and award-amount families are the notable ones — ROR IDs are how
crossref now models affiliations. Relatedly, `Affiliation` only models `name`;
crossref now also returns an `id` array carrying ROR identifiers.

### 2.2 Filters that are not valid on `/works`

`WorksFilter::Location` and `WorksFilter::ReferenceVisibility` are rejected by
`/works`. `location` belongs to `/funders`. Either move them to the right
filter type or document the restriction — currently they produce a 400 at
runtime with no compile-time signal.

### 2.3 Missing field queries — 12 of 21

`FieldQuery` has 9 constructors (`src/query/works.rs`). Missing:

```
query.degree                  query.description
query.event-acronym           query.event-location
query.event-name              query.event-sponsor
query.event-theme             query.funder-name
query.publisher-location      query.publisher-name
query.standards-body-acronym  query.standards-body-name
```

### 2.4 `/licenses` route not implemented

The only documented route with no support. `Component`'s doc comments already
mention licenses (see 4.1).

### 2.5 Add a coverage test

`src/query/mod.rs` now has `every_sort_key_is_accepted_by_the_api`, which pins
`Sort` against the list crossref reports. Do the same for `WorksFilter` and
`FieldQuery` so drift is caught by `cargo test` rather than by a 400 in
production.

---

## 3. Client behaviour

### 3.1 Honour rate limits

Crossref returns `x-rate-limit-limit` and `x-rate-limit-interval` on every
response (currently 1/s anonymous, higher in the polite pool) and expects
clients to respect them. The client ignores both and has no retry, so bursts
return 429 — this is why the integration tests have to take turns through a
mutex.

**Fix.** Parse the headers and add a shared limiter, plus bounded retry with
backoff on 429. This would also let the integration tests run concurrently
again. Now that deep paging yields `Result` (1.3), a 429 mid-crawl is visible
rather than silent, but it still ends the crawl.

### 3.2 Expose `base_url` on the builder

`CrossrefBuilder` has a `base_url` field but no setter, so the only way to point
the client at a mock server is to construct `Crossref { base_url, client }`
literally. Add `CrossrefBuilder::base_url` — it is also what makes offline
tests of the routing layer possible.

### 3.3 Reconsider the public fields on `Crossref`

`base_url: String` and `client: reqwest::Client` are `pub`. That leaks reqwest
into the public API and lets callers mutate the base url mid-flight. Make them
private once 3.2 lands (`examples/check_pool.rs` uses `client` and would need
adjusting).

### 3.4 `journals()` is inconsistent with every other route — **done**

`JournalsQuery` replaces the `(String, Option<JournalResultControl>)` pair, and
`JournalResultControl` — with its stringly-typed `sort: Option<String>` — is
gone. `Crossref::journals` now takes a query struct like every other list route.

`JournalsQuery` deliberately carries only free form terms and a `ResultControl`
rather than going through `impl_common_query!`: `/journals` was probed against
the live api and rejects `filter`, `sort`, `order`, `facet`, `select` and
`sample`, accepting only `query`, `rows` and `offset`. Note `ResultControl` can
still express `Sample`, which this route rejects — the same class of gap as 2.2.

### 3.5 The CLI silently drops flags a route cannot honour

`crossref journals --sort score` accepts `--sort` and ignores it, because
`/journals` has no `sort` parameter (3.4). `--sample` is dropped there too. The
flags are shared across every subcommand through one `Opts`, so the CLI should
either reject a flag the target route does not support or warn about it, rather
than quietly returning differently ordered results than asked for.

---

## 4. API surface and docs

### 4.1 `Component` doc comments are shuffled

`src/query/mod.rs` — `Prefixes` is documented as "a list of all Crossref
members", `Members` as "a list of valid work types", and `Types` as "a list of
licenses". Straight copy-paste drift; the same wrong text is duplicated on
`ResourceComponent`.

### 4.2 Document the public API

`#![warn(missing_docs)]` is commented out in `src/lib.rs` with a TODO — re-enable
it once the remaining items are documented, and keep it on.

### 4.3 Finish or remove `cn` and `tdm`

`src/cn.rs` defines `CnFormat` with mime types and headers, but nothing
references it — content negotiation is not actually implemented. `src/tdm/mod.rs`
is an empty file. Either implement content negotiation (a `Crossref::transform`
that sends the `Accept` header and returns the raw body) or delete both modules;
right now they are public API that does nothing.

### 4.4 Builder setters that accept `Option`

The CLI has to fall back to struct-literal construction because the builder
setters take bare values while its own options are `Option<T>`. Adding
`maybe_sort(Option<Sort>)`-style setters, or making the existing ones take
`impl Into<Option<T>>`, would remove that.

### 4.5 Two deserialization paths for `Response`

`Response` has both a `TryFrom<serde_json::Value>` (used by the client) and a
hand-written `Deserialize` (used by the fixture tests). They are independent
implementations of the same mapping and have drifted before. Collapse them —
`TryFrom` should call `serde_json::from_value::<Response>`.

`JournalList::try_from` is part of the same knot: it deserializes into a private
`Raw` struct and then synthesizes `facets: HashMap::new()`, because `JournalList`
carries a `facets` field that `/journals` never returns. The field probably does
not belong on the type.

### 4.6 Response types are not re-exported at the crate root

`WorkElement` not being re-exported made `WorksQuery::elements` unreachable from
outside the crate (1.5). The same is true of most of `response::work` —
`Contributor`, `License`, `Date`, `PartialDate`, `Reference`, `Explanation` and
the rest are only reachable through the full `crossref_client::response::work::`
path, even though they appear in the public fields of `Work`. Re-export the
types that a caller of `works()` has to name.

### 4.7 `Relation` and `Review` are dead types

`Work::relation` and `Work::review` are both `Option<Relations>`, i.e.
`HashMap<String, serde_json::Value>`, so the `Relation` and `Review` structs
next to them are never constructed. The comment on `Relations` says the value
can also be an array, which is why it was widened. Either type those two fields
properly (an untagged enum over the two shapes, as done for `Explanation`) or
delete the unused structs.

---

## 5. Project hygiene

### 5.1 Add CI

`.travis.yml` has been deleted (Travis CI for open source is gone) and nothing
replaced it. Add a GitHub Actions workflow running `cargo test`,
`cargo clippy --all-targets --all-features -- -D warnings` and `cargo doc`.

`tests/integration.rs` hits the live API and serializes itself through a mutex
to stay under the rate limit — either gate it behind a feature or accept the
network dependency in CI. `tests/work_shapes.rs` is offline and can always run.

### 5.2 Include the repository URL in the polite User-Agent

`Cargo.toml` now sets `repository`, so `polite()` can send crossref's preferred
form, `Project/version (https://url; mailto:email)`, instead of the current
`crossref-client/0.2.0 (mailto:email)`.

### 5.3 Publish under the fork name

The crate is now `crossref-client` (see the fork notice in the README). It has
not been published yet — `cargo publish --dry-run` first, and confirm the name
is free on crates.io.

### 5.4 Offline route tests

Route construction is now covered offline for `/works` and `/journals`
(encoding, `select`, cursors, rows+offset, empty queries). Once 3.2 lands, add
tests that point `base_url` at a local mock and assert the full request url end
to end.

### 5.5 A periodic shape check against the live API

`tests/work_shapes.rs` pins the shapes known today. Crossref keeps adding fields
(ROR ids on affiliations, for one) and members keep depositing new gaps, so it
is worth re-running a large `sample=100` sweep through `Work` occasionally to
catch the next one before a user does.
