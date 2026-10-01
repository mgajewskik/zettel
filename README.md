# zettel

Fast, Obsidian-compatible CLI for wikilink graph queries. Single static binary named `zettel`.

Built on [`obsidian-rs-core`](https://crates.io/crates/obsidian-rs-core) (Apache-2.0) for note
resolution, outbound links, backlinks, vault health checks, and search — this crate does **not**
reimplement the link graph.

## Install

```bash
# from this repo
cargo install --path .

# or build a release binary
cargo build --release
# → target/release/zettel
```

Prebuilt binaries for Linux (x86_64), macOS (aarch64 + x86_64), and Windows (x86_64) are attached to
[GitHub Releases](https://github.com/mgajewskik/zettel/releases) when a version tag is pushed (see
[Releases](#releases)).

**Rust:** edition 2024 / MSRV **≥ 1.88** (see `rust-toolchain.toml`). Use [rustup](https://rustup.rs/).

## Releases

Cut a release by tagging `main` and pushing the tag (do **not** commit binaries into the repo):

```bash
git tag v0.1.0
git push origin v0.1.0
```

That triggers `.github/workflows/release.yml`: `cargo test`, then cross-platform release builds, then
a GitHub Release with assets named `zettel-<os>-<arch>` (Windows: `.exe` suffix).

## Vault configuration

| Priority | Source |
|----------|--------|
| 1 | `--vault <path>` |
| 2 | env `ZETTEL_VAULT` |

If neither is set, the CLI exits with code **2** and an error message. No personal vault path is
hardcoded.

```bash
export ZETTEL_VAULT=/path/to/your/vault
# or
zettel --vault /path/to/your/vault links alpha
```

## Commands (P0)

Global flags: `--vault <path>`, `--format text|json` (default: `text`).

| Command | Description |
|---------|-------------|
| `zettel links <note>` | Outbound wikilinks / markdown links / embeds from a note |
| `zettel backlinks <note>` | Notes that link to `<note>` |
| `zettel unresolved [path…]` | Broken links (optional path filters on source notes) |
| `zettel exists <query>` | Resolve note by id / alias / title / stem / path |
| `zettel find <query>` | Search by id, title, alias, or content |

`<note>` / `<query>` use core’s stem + kebab-id + alias matching.

### Examples

```bash
zettel --vault ./tests/fixtures/vault links alpha
zettel --vault ./tests/fixtures/vault --format json backlinks beta
zettel --vault ./tests/fixtures/vault unresolved
zettel --vault ./tests/fixtures/vault exists "Alpha Alias"
zettel --vault ./tests/fixtures/vault find gamma
```

### Exit codes

| Code | Meaning |
|------|--------|
| 0 | Success |
| 1 | Issues found (`unresolved` nonempty, `exists`/`find` miss) |
| 2 | Usage / config error (missing vault, bad args) |

## Output

- **text** (default): human-readable lines
- **json**: pretty-printed JSON for bots / scripts (`--format json`)

## Vault walk filters

Core’s `ignore` crate already skips `.obsidian/` and `.git/`. This CLI additionally filters out
notes under `media/` and `canvas/` directory components when scanning (`unresolved`, `find`).

## Known gaps (upstream / by design)

- **Path-style wikilinks** like `[[folder/Note]]` are not indexed as first-class targets in
  `obsidian-rs-core` 0.6 (stem / id / alias matching only). Prefer ids or note stems.
- No LSP server in this binary (by design).

## Development

```bash
cargo test
cargo build --release
```

CI runs `cargo test` on pushes and PRs to `main` (`.github/workflows/ci.yml`).

Synthetic fixtures live under `tests/fixtures/` (fake notes only — safe to publish).

## License

Apache-2.0 — see [LICENSE](LICENSE).

`obsidian-rs-core` is also Apache-2.0; credit to [epwalsh/obsidian.rs](https://github.com/epwalsh/obsidian.rs).
