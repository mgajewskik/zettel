# Vault folder conventions (check-links)

Public, generic layout used by `zettel check-links --rule zettel`. Adapt folder
names to your vault; this document describes the **rule semantics**, not a
personal vault path.

## Reserved folders (non-permanent)

Notes whose vault-relative path lies under any of these directories are **not**
permanent notes:

| Folder | Role |
|--------|------|
| `inbox/` | Captures / unprocessed |
| `source/` | Literature / source notes |
| `project/` | Project-scoped notes |
| `archive/` | Archived notes |
| `media/` | Attachments (also skipped by vault walks) |
| `canvas/` | Canvas files (also skipped by vault walks) |
| `moc/` | Maps of content |

Everything else — including notes at the vault root and under other folders
such as `notes/` — is treated as a **permanent** note.

## Rule `zettel`

Permanent notes must **not** link **to** `source/**`.

- Outbound wiki links (`[[…]]`), embeds (`![[…]]`), and local markdown links
  (`.md` URLs) are checked.
- A link is a violation when its **resolved** path is under `source/`, or when
  an unresolved target string indicates `source/` (for example `[[source/foo]]`).
- Source notes **may** link to permanents; those edges are never flagged.
- Permanent → permanent links are fine.

Resolution reuses the same graph helpers as `links` / `unresolved` /
`backlinks`: path-style targets, same-folder basename, then unique vault-wide
basename.

## CLI

```bash
zettel check-links --rule zettel [PATH…] [--include-archive]
zettel --format json check-links --rule zettel
```

- Optional `PATH` filters limit which **permanent sources** are reported
  (same semantics as `unresolved`).
- `archive/**` sources are excluded by default (`--include-archive` to opt in).
  Notes under `archive/` are also non-permanent, so they are outside the rule’s
  candidate set regardless.
- Exit **0** if clean, **1** if any violations, **2** on config/usage errors.
