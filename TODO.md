# TODO

Follow-up work after the Rust 2024 / dependency modernization (`1129c29`) and
the API-surface pass that followed it.

Status: builds clean on Rust 1.96, `cargo fmt`, clippy and rustdoc are
warning-free with `#![warn(missing_docs)]` on, and 128 tests pass (61 unit, 39
doc, 15 live-api, 7 response-shape, 6 offline route).

Everything in sections 1–4 of the previous list is done. What is left is
publishing, and a handful of gaps found along the way.

---

## 1. Publishing

### 1.1 Publish under the fork name

The crate is now `crossref-client` (see the fork notice in the README). It has
not been published yet — `cargo publish --dry-run` first, and confirm the name
is free on crates.io.

### 1.2 Build CLI binaries on release

`cargo install crossref-client --features cli` works, but a release should also
attach prebuilt `crossref` binaries so the CLI can be used without a toolchain.
Add a `release.yml` workflow triggered on a tag that cross-builds
`--features cli` for linux-gnu, macos (x86_64 + aarch64) and windows-msvc and
uploads them to the GitHub release.

---

## 2. Known gaps

### 2.1 `/types` accepts a query, and this crate cannot send one

Probing the routes turned up that `/types` takes `query`, `rows` and `offset`,
but `Types` only models `All`, `Identifier` and `Works`. There are 28 work
types in total, so querying them is close to pointless — noted so the gap is a
decision rather than an oversight.

`/prefixes` has no list route at all (`/prefixes` alone is a `404`), which is
why `Prefixes` has no `Query` variant.

### 2.2 `Work::doi` is required, so a `select` that omits it fails

`WorksQuery::elements` narrows the response and every field left out is `None`,
which `Work` now models — except `DOI`, which is still required. Modelling that
properly means a separate partial-work type rather than making the primary key
optional. Worth doing if anyone needs it.

### 2.3 A `,` inside a filter value cannot be escaped

Crossref splits a `filter` value on `,` *after* percent-decoding it, so neither
the encoded nor the literal form survives. That is an api limitation, not a
client one, but it means `WorksFilter::ContainerTitle("A, B")` cannot match.

### 2.4 Untyped corners

* `Journal::coverage` and `Journal::coverage_type` are `serde_json::Value`.
  Crossref keeps adding fields to them, and a struct listing the ones it sends
  today would reject every journal deposited after the next field lands. The
  `Coverage` struct on `Member` *is* typed and has exactly that problem.
* `#[allow(missing_docs)]` survives on `Type`, `MessageType`, `Funder`,
  `Member` and `Coverage` — enums whose variants are self-describing crossref
  identifiers, and structs that mirror a crossref record field for field. Each
  carries a comment saying why.

---

## 3. Maintenance

### 3.1 A periodic shape check against the live API

`tests/work_shapes.rs` pins the shapes known today, from a sample of 38 800
works. Crossref keeps adding fields and members keep depositing new gaps, so
`.github/workflows/ci.yml` runs the live suite weekly. That only covers the
records those tests happen to touch; re-running a large `sample=100` sweep
through `Work` occasionally is still the way to catch the next new shape before
a user does.

### 3.2 The API coverage tests pin a snapshot, not the API

`every_works_filter_is_accepted_by_the_api` and its four siblings compare this
crate's vocabulary against a list copied out of crossref's own `400` body. They
catch a name this crate would send and crossref would reject, and one crossref
accepts that this crate cannot express — but only against the snapshot. The
weekly live job is what surfaces a list crossref has since changed, and only if
someone re-copies it. A test that fetches the `400` body and diffs it would
close that loop at the cost of a network dependency in the coverage tests.
