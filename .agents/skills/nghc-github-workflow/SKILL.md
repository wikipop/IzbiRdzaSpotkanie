---
name: nghc-github-workflow
description: Apply Norbiros' GitHub Conventions (NGHC) consistently across issues, branch names, commit boundaries and messages, and pull requests. Use whenever a repository or user mentions NGHC or Norbiros' GitHub Conventions, or when contributing to a project that explicitly adopts NGHC, especially before creating a branch, splitting or writing commits, opening a PR, selecting issue metadata, or reviewing whether Git history and PR metadata follow the convention.
---

# NGHC GitHub Workflow

Treat an NGHC change as one semantic story expressed consistently across the issue, branch, commits, and pull request. Prefer repository evidence over generic Git advice.

## Establish the repository contract

1. Read repository instructions such as `AGENTS.md`, `CONTRIBUTING.md`, `.github/`, issue forms, PR templates, and release or commit policies.
2. Confirm that the repository actually adopts NGHC. Do not activate NGHC merely because the project uses Conventional Commits.
3. Identify the default/base branch and any repository-defined scopes. Never invent a scope just to fill the format.
4. Inspect the relevant issue, current branch, staged and unstaged changes, and recent commit/PR history before proposing metadata.
5. Let explicit user instructions and repository-local rules override this baseline when they intentionally differ from NGHC.

Read [nghc-reference.md](references/nghc-reference.md) for the baseline convention and [edge-cases.md](references/edge-cases.md) when the change does not map cleanly to one type, scope, issue, or PR.

## Build one semantic spine

Determine these facts once, then reuse them everywhere:

- **type:** `feat`, `fix`, `docs`, `perf`, `refactor`, `test`, `revert`, or `chore`;
- **scope:** a repository-defined area when useful, otherwise omit it;
- **subject:** short imperative description of the outcome;
- **issue:** the real related issue number when one exists;
- **breaking:** whether the change intentionally breaks a public contract.

Keep the artifacts aligned:

| Artifact | NGHC expression |
| --- | --- |
| Issue | matching issue type, plus repository labels when applicable |
| Branch | `type/short-description` or `type/scope/short-description` |
| Commit | `type(scope): imperative description (#issue)` |
| PR title | prefer the same commit-style semantic title unless the repository prefers a sentence title |
| PR body | start with the correct closing keyword when merge should close the issue |

Do not force unrelated work under one semantic spine. Split the work instead.

## Shape reviewable commits

1. Review the complete diff before deciding commit boundaries.
2. Group changes by one functional reason, not by file extension or the order in which edits happened.
3. Split commits when the work changes scope, has independently reviewable steps, or contains a pure move/rename plus later behavioral edits.
4. Keep a functional change and the tests required to prove that same change together unless the test work is independently meaningful.
5. Do not leave `wip`, `fix stuff`, `typo`, `will squash`, or similar process-noise commits in PR history.
6. Do not compress a large change into one commit merely to make history shorter. Prefer a readable sequence of atomic commits.
7. If a pre-existing bug must be fixed to enable a larger feature and can stand alone, prefer a separate commit or PR so it can be reviewed and merged independently.

Before any real commit, stage only files that belong to that commit. Preserve unrelated user changes.

## Write NGHC commit messages

Use an NGHC-compatible Conventional Commit header:

```text
<type>(<optional-scope>)<optional-!>: <imperative description> (#issue)
```

Omit the parentheses when no scope is used. Use the Conventional Commits breaking-change position, for example `feat(api)!: ...`, unless the target repository documents another exact syntax.

- Use imperative mood: `add`, `fix`, `update`, `remove`, not `added` or `fixed`.
- Keep the subject concise, preferably under 72 characters before the reference.
- Use lowercase singular scopes defined by the project.
- Treat CI, dependencies, style, and release automation as scopes where appropriate, not new top-level types. Examples include `fix(ci): preserve test artifacts on failure`, `chore(deps): update Nuxt dependencies`, and `chore(release): prepare the next release`.
- Add a body when the reason, tradeoff, migration, or non-obvious behavior is useful to future readers. Explain why and consequences instead of narrating the diff.
- Add `BREAKING CHANGE: ...` when a breaking change needs an explicit footer.

Never invent an issue or PR number. See [edge-cases.md](references/edge-cases.md) when NGHC's reference rule cannot yet be satisfied.

## Name the branch

Use:

```text
<type>/<short-description>
<type>/<scope>/<short-description>
```

- lowercase only;
- hyphen-separated description;
- scope optional, but use it when the project has a stable scope vocabulary and it improves navigation;
- prefer a descriptive branch over `issue/123` or another identifier-only name;
- branch type should match the dominant change represented by the branch and intended PR.

Do not work directly on the production branch when the repository expects PR-based changes.

## Prepare the pull request

1. Re-read the branch diff against the intended base branch and verify the PR has one reviewable purpose.
2. Prefer an NGHC commit-style title derived from the same type, scope, and subject. Use a clear sentence title only when repository convention calls for it.
3. If merging the PR should close an issue, place the closing line at the top, for example:

```text
Fixes #123
```

4. Follow the repository's PR template when one exists. Otherwise use a minimal body that explains what changed, why, and how it was verified.
5. Assign the PR only when responsibility currently belongs to someone other than the author, or when the repository has an explicit workflow requiring assignment.
6. If an independent fix or prerequisite can merge separately from a large feature, split it. Use stacked PRs only when dependency between reviewable changes makes that useful.
7. Do not claim tests, linters, builds, or manual verification that were not actually run.

## Validate consistency before handoff

Check all applicable items:

- issue type maps to the chosen commit type;
- branch type and scope agree with the intended PR;
- each commit is atomic and has an allowed type;
- commit scopes come from repository evidence or are omitted;
- commit subjects are imperative and meaningful;
- real issue references are used consistently;
- PR title expresses the same dominant change as the branch;
- PR body uses a closing keyword only when merge should close the issue;
- no unrelated changes, WIP history, fabricated metadata, or unverified test claims remain.

If reviewing rather than creating a change, report concrete mismatches and the smallest correction that restores consistency. Do not rewrite history or mutate GitHub unless the user has authorized that action.
