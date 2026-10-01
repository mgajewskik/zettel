# Vault folder conventions (check-links)

Public, generic layout used by `zettel check-links --rule zettel`. Adapt folder
names to your vault; this document describes the **rule semantics**, not a
personal vault path.

## Permanent notes (`--scope root`)

For `--rule zettel`, **permanent** notes are only `*.md` files **directly under
the vault root** (no subdirectory). This matches the CONTEXT convention that
atomic permanents live at the vault root after review.

`--scope root` is the explicit default (and currently the only scope).

Examples that **are** permanents:

- `idea.md` at vault root
- `moc-overview.md` at vault root (root `moc-*.md` files are permanents)

Examples that are **not** permanents (any subdirectory):

| Folder | Typical role |
|--------|----------------|
| `inbox/` | Captures / unprocessed |
| `source/` | Literature / source notes |
| `project/` | Project-scoped notes |
| `archive/` | Archived notes |
| `area/` | Area / domain notes |
| `docs/`, `grok/`, `templates/` | Supporting material |
| `media/`, `canvas/` | Attachments / canvases (also skipped by vault walks) |
| `moc/` | Maps of content **inside** a folder (unlike root `moc-*.md`) |

## Rule `zettel`

Permanent notes must **not** link **to** `source/**`.

- Outbound wiki links (`[[…]]`), embeds (`![[…]]`), and local markdown links
  (`.md` URLs) are checked.
- A link is a violation when its **resolved** path is under `source/`, or when
  an unresolved target string indicates `source/` (for example `[[source/foo]]`).
- Source notes **may** link to permanents; those edges are never flagged.
- Permanent → permanent links are fine.
- Notes under subdirectories (including `area/` linking `source/**`) are outside
  the candidate set and are never flagged by this rule.

Resolution reuses the same graph helpers as `links` / `unresolved` /
`backlinks`: path-style targets, same-folder basename, then unique vault-wide
basename.

## CLI

```bash
zettel check-links --rule zettel [--scope root] [PATH…] [--include-archive]
zettel --format json check-links --rule zettel
```

- `--scope root` (default): only vault-root `*.md` count as permanents.
- Optional `PATH` filters limit which **permanent sources** are reported
  (same semantics as `unresolved`).
- `archive/**` sources are excluded by default (`--include-archive` to opt in).
  With `--scope root`, archive notes are already non-permanents; the flag is
  kept for filter parity with `unresolved`.
- Exit **0** if clean, **1** if any violations, **2** on config/usage errors.
