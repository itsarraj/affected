# affected

Given a git diff, which projects in this monorepo actually changed — so
CI only builds/tests what changed, instead of every project on every PR.
Nx and Turborepo do this, but only if you buy into their whole build
system. This is a standalone binary that answers just this one question.

## Usage

```bash
affected --range main...HEAD          # merge-base diff, typical PR/CI use
affected --range main..HEAD           # literal two-dot diff
affected --range HEAD                 # working-tree changes vs last commit
affected --json
```

```bash
# in CI:
for project in $(affected --range "$BASE_SHA...$HEAD_SHA"); do
  (cd "$project" && cargo test)
done
```

Changed files that don't fall under any recognized project (a top-level
`README.md`, a `notes/` file) are reported separately on stderr, not
silently dropped — "nothing matched" and "everything matched cleanly"
are different facts worth knowing apart.

## How it finds project boundaries

A directory is a project root if it directly contains `Cargo.toml`,
`package.json`, `pyproject.toml`, or `go.mod` — discovered from `git
ls-files`, not a filesystem walk, so it only ever sees what's actually
tracked. A changed file maps to the **most specific** (longest-path)
matching root, so a change inside `apps/web/src/` resolves to `apps/web`
even if some broader ancestor also happens to be a project root.

## Status: built and verified against this actual monorepo's real history, not a toy fixture

- **12 unit tests** (`cargo test --lib`): project-root discovery
  (including a marker file at the repo's own top level becoming the `"."`
  root, and duplicate marker files in one directory correctly collapsing
  to a single root instead of two); the most-specific-root-wins matching
  rule; unmatched files reported rather than dropped; and — a real
  correctness edge case, not a hypothetical — **`tools/pgqueue-extra/x.rs`
  confirmed to *not* falsely match root `tools/pgqueue`**, the exact bug
  a naive `starts_with` prefix check without a trailing-slash boundary
  check would produce; plus `git.rs`'s `ls-files`/`diff --name-only`
  against real temporary git repositories, not mocked output.
- **Dogfooded against this actual monorepo's real commit history**: run
  with `--range dbe543c..HEAD` (an 8-commit real range spanning a
  workspace-wide "add unit test suite" pass across dozens of apps) —
  correctly identified all **135 real affected projects**
  (`apps/airwave/app`, `apps/airwave/bff`, `apps/airwave/ui`, ... down
  the actual `apps/*` tree, each `app`/`bff`/`ui` split resolved as its
  own distinct project since each has its own marker file, exactly
  matching this repo's real structure). Then run with `--range HEAD`
  against this repo's actual uncommitted working-tree state at the time
  (two real modified files, `tools/README.md` and
  `tools/create-and-push-repos.sh`, both legitimately outside any
  project) — correctly produced **zero** affected projects on stdout and
  both files listed under "changed file(s) outside any known project" on
  stderr, not silently treated as if nothing happened.

**Not done / deliberately deferred**: dependency-graph awareness (Nx's
other headline feature — if `libs/shared` changes, everything that
*imports* it is also "affected," not just `libs/shared` itself; this only
answers "which project's own files changed," not "and what depends on
it," a real and meaningfully harder problem); and marker files beyond the
four listed (no `Gemfile`, `composer.json`, `*.csproj`, ... — easy to
extend, not attempted speculatively here).
