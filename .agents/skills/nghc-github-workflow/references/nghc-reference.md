# NGHC Baseline Reference

Use this file for the convention details behind the workflow. Repository-local NGHC adoption may deliberately narrow or override these defaults.

## Commit types

NGHC maps code changes to a small set of top-level types:

| Type | Meaning |
| --- | --- |
| `feat` | new user/developer-facing capability |
| `fix` | user/developer-facing bug fix |
| `docs` | documentation changes |
| `perf` | performance improvement |
| `refactor` | maintainability/readability change without intended behavior change |
| `test` | adding or refactoring tests when test work is the primary change |
| `revert` | reverting a previous change |
| `chore` | internal work not fitting the other types |

NGHC intentionally treats many commonly used Conventional Commit types as scopes instead:

- CI work: `feat(ci)`, `fix(ci)`, `chore(ci)` depending on the actual semantic change;
- dependency maintenance: commonly `chore(deps)`;
- formatting/style automation: commonly `chore(style)`;
- release maintenance: commonly `chore(release)`.

Do not add `ci`, `build`, or `enhancement` as top-level types unless the target repository explicitly overrides NGHC.

## Commit header

Baseline shape:

```text
<type>(<optional-scope>)<optional-!>: <short description> (#issue)
```

Examples:

```text
feat(auth): add password reset flow (#42)
fix(ui): align submit button with form actions (#101)
docs: document local setup
chore(deps): update Nuxt dependencies
fix(ci): preserve test artifacts on failure
```

Rules:

- imperative subject;
- concise header, preferably under 72 characters before a reference;
- scope is optional but recommended when a stable repository vocabulary exists;
- body is useful for non-obvious reasoning, tradeoffs, or consequences;
- breaking changes use `!` and may use a `BREAKING CHANGE:` footer.

NGHC documentation says commits should reference the issue they solve, or the PR when no issue exists. Never fabricate a number to satisfy this rule. Use the practical handling in `edge-cases.md`.

## Commit boundaries

NGHC treats commit structure as part of reviewability and project history:

- split distinct implementation steps when they remain understandable and reviewable independently;
- split when the scope of the work changes;
- keep file moves/renames separate from later behavioral edits when practical;
- keep commits functionally atomic;
- avoid WIP/process-noise commits in the final PR history;
- do not assume a large PR should have only one commit;
- if a prerequisite bug fix is independently useful, prefer a separate commit or PR.

## Branches

The production branch is normally `master` in NGHC, with `main` accepted for compatibility. Direct production-branch commits are discouraged; changes should generally pass through PRs and CI.

Normal branch shape:

```text
<type>/<short-description>
<type>/<scope>/<short-description>
```

Examples:

```text
fix/resolve-login-issue
chore/deps/update-nuxt
feat/backend/user-authentication
```

Use lowercase letters and hyphens. Prefer descriptive branches over identifier-only names such as `issue/123`.

## Pull requests

NGHC permits two title styles:

- clear sentence, for example `Fixes the bug in the authentication flow`;
- commit-style title, for example `fix(auth): correct authentication flow`.

For agent-generated changes, prefer commit style because it preserves the semantic mapping from branch and commits unless the repository chooses sentence titles.

When a PR closes an issue, put a GitHub closing keyword at the top of the body, for example:

```text
Fixes #123
```

Use the assignee field only when the person currently responsible for the PR differs from the author, or when multiple contributors are explicitly sharing responsibility.

Keep PRs reviewable. Independent prerequisite fixes should be separated from a larger feature when practical. Stacked PRs are appropriate when reviewable changes depend on each other.

## Issues and labels

NGHC aligns issue types with commit types:

| Issue type | Commit type |
| --- | --- |
| Feature | `feat` |
| Bug | `fix` |
| Documentation | `docs` |
| Performance | `perf` |
| Refactor | `refactor` |
| Test | `test` |
| Chore | `chore` |

NGHC label families include:

- `good first issue` for well-scoped first-contributor work;
- `scope: <area>` for project areas;
- `type: <type>` for clarifiers such as regression or upstream;
- `status: pending triage` for newly filed work awaiting maintainer review.

Do not add labels that do not exist in the target repository.
