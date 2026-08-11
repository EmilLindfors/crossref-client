# TODO

Follow-up work after the Rust 2024 / dependency modernization (`1129c29`).

Status at that commit: builds clean on Rust 1.96, clippy and rustdoc are
warning-free, 62 tests pass (26 unit, 30 doc, 6 integration).

Items are ordered by impact. The first section contains correctness bugs that
can silently produce wrong results today.

---

## 1. Correctness

### 1.1 Percent-encode query parameters — **highest priority**

`format_query` / `format_queries` (`src/query/mod.rs:553`, `:563`) only replace
whitespace with `+`. Nothing in the crate percent-encodes anything.

```rust
WorksQuery::new("R&D")     // -> /works?query=R&D
```

The `&` terminates the parameter, so crossref sees a stray `D` parameter and
silently returns results for `R`. The same applies to `#`, `=`, `%`, `?` and any
non-ASCII term. Filter values (`WorksFilter::ContainerTitle`, `Assertion`, …)
and field query values go out unencoded too.

This is both a correctness bug and a query-injection vector for callers who pass
user input straight through.

**Fix.** Build routes with a real encoder rather than string concatenation. The
`url` dependency was declared but never used and has been removed — reintroduce
`form_urlencoded`, or `url::Url` with `query_pairs_mut()`.

Note this interacts with item 1.2: the current `CrossrefQueryParam` shape makes
correct encoding awkward, so the two are best done together.

### 1.2 `param_key` returns more than a key

`CrossrefQueryParam::param()` joins `param_key()` and `param_value()` with `=`.
Two variants need to emit *two* parameters, and work around it by stuffing both
into `param_key()` and returning `None` from `param_value()`:

- `ResultControl::RowsOffset` — `src/query/mod.rs:348`
- `WorkResultControl::Cursor` with `rows` — `src/query/works.rs:552`

The second one was a live bug until recently (it rendered `cursor=*=rows=20`);
the shape invites that class of mistake and defeats any attempt to encode keys
and values separately.

**Fix.** Change the trait to yield pairs, e.g.
`fn params(&self) -> Vec<(Cow<'_, str>, Cow<'_, str>)>`, and let the route
builder do the joining and encoding. This is the enabling refactor for 1.1.

### 1.3 Deep paging silently swallows every error

`WorkListIterator::next` (`src/lib.rs:856`, `:860`) returns `None` on *any*
failure — network error, 429, deserialization failure. A caller crawling 100k
works who hits one transient rate-limit gets a truncated result set that is
indistinguishable from a clean finish.

**Fix.** Make the iterator yield `Result<WorkList>` (and `WorkIterator` yield
`Result<Work>`). This is a breaking change to `AsyncIterator::Item`; do it
before 1.0.

### 1.4 Response parsing panics instead of erroring

69 `.unwrap()` calls inside `TryFrom<serde_json::Value>` impls that already
return `Result`:

- `src/response/work.rs` — 54
- `src/response/mod.rs` — 15

For example `WorkList::try_from` does `Work::try_from(v.clone()).unwrap()` per
item, so one unexpected work in a 20-item page panics the caller's task rather
than returning an error.

**Fix.** Propagate with `?`. Where crossref genuinely returns heterogeneous
shapes, prefer `Option`/`serde_json::Value` in the type over an unwrap.

Consider whether these hand-written `TryFrom` impls should exist at all — the
`Journal` ones were deleted in favour of plain serde derives because the two
paths had drifted apart and disagreed. `Work` has the same duplication.

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
crossref now models affiliations.

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
again.

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

### 3.4 `journals()` is inconsistent with every other route

```rust
client.journals(query: String, result_control: Option<JournalResultControl>)
```

Every other list route takes a query struct (`WorksQuery`, `MembersQuery`,
`FundersQuery`). `JournalResultControl` is also a parallel, stringly-typed
result-control type with `sort: Option<String>` instead of `Sort`.

**Fix.** Introduce `JournalsQuery` following `impl_common_query!` and fold the
result control into the standard `ResultControl`.

---

## 4. API surface and docs

### 4.1 `Component` doc comments are shuffled

`src/query/mod.rs` — `Prefixes` is documented as "a list of all Crossref
members", `Members` as "a list of valid work types", and `Types` as "a list of
licenses". Straight copy-paste drift; the same wrong text is duplicated on
`ResourceComponent`.

### 4.2 Document the public API

~118 public items have no doc comment (59 enum variants, 41 struct fields, 8
methods, 6 associated functions, 4 structs). `#![warn(missing_docs)]` is
commented out in `src/lib.rs` with a TODO — re-enable it once they are filled
in, and keep it on.

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

---

## 5. Project hygiene

### 5.1 Add CI

`.travis.yml` has been deleted (Travis CI for open source is gone) and nothing
replaced it. Add a GitHub Actions workflow running `cargo test`,
`cargo clippy --all-targets --all-features -- -D warnings` and `cargo doc`.

The integration tests hit the live API and serialize themselves through a mutex
to stay under the rate limit — either gate them behind a feature or accept the
network dependency in CI.

### 5.2 Include the repository URL in the polite User-Agent

`Cargo.toml` now sets `repository`, so `polite()` can send crossref's preferred
form, `Project/version (https://url; mailto:email)`, instead of the current
`crossref-client/0.2.0 (mailto:email)`.

### 5.3 Publish under the fork name

The crate is now `crossref-client` (see the fork notice in the README). It has
not been published yet — `cargo publish --dry-run` first, and confirm the name
is free on crates.io.

### 5.4 Offline route tests

Every test that exercises routing beyond the unit level needs the network. Once
3.2 lands, add tests that point `base_url` at a local mock and assert the exact
URL produced for representative queries — that is where bugs like the missing
`/works?` prefix on `sample` would have been caught.
