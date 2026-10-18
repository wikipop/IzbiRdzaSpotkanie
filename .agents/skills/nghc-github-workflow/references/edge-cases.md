# NGHC Agent Edge Cases

Use these rules when the baseline convention is ambiguous or cannot be followed literally without inventing metadata or damaging history.

## No issue exists and the PR number does not exist yet

NGHC says a commit title should reference the issue it solves or, when there is no issue, the PR. A normal workflow creates commits before the PR has a number.

Use this order of preference:

1. If a real related issue exists, reference it.
2. If no issue exists and the repository does not enforce a reference in every commit, create the commit without a fabricated reference and put the real PR relationship in the PR metadata later.
3. If the repository strictly enforces a PR reference in commit headers, surface the constraint before rewriting history. Amend already-published commits only when the user explicitly authorizes the history rewrite and the collaboration state makes it safe.
4. Never guess or reserve a fake `#123` style value.

## The change appears to need two types

Choose the type from the primary semantic outcome, not from the number of touched files.

Examples:

- feature plus tests proving the feature: usually `feat`, not separate `test`, unless the test work is independently meaningful;
- bug fix plus regression test: usually `fix`;
- refactor that also fixes observable behavior: `fix` if the behavior correction is the reason for the change;
- dependency update with no intended behavior change: `chore(deps)`;
- CI bug: `fix(ci)`;
- new CI capability: `feat(ci)`;
- pure test-suite restructuring: `test`;
- documentation changed only because a feature changed: keep docs in the feature commit when inseparable, or use a separate `docs` commit when independently reviewable.

If two outcomes are independently valuable, split them into separate commits or PRs.

## Scope is unclear

Do not infer a scope from a random directory name.

Prefer, in order:

1. explicit repository documentation;
2. existing `scope: <area>` labels;
3. stable scopes in recent accepted commits/PRs;
4. stable package/module names clearly used as scopes;
5. no scope.

A missing optional scope is better than a novel one that fragments history.

## Branch contains multiple commits with different types

The branch and PR should use the dominant user/reviewer-facing purpose. Individual commits may use different valid types for genuinely separate steps.

Example:

```text
branch: feat/auth/password-reset
commits:
  feat(auth): add reset-token flow (#42)
  docs(auth): document reset-token configuration (#42)
```

If the second commit has an independent lifecycle or can merge without the feature, consider a separate PR instead.

## File move plus behavior change

Prefer:

1. move/rename-only commit;
2. behavior change commit.

Combine them only when separating would create a broken or misleading intermediate state.

## Closing versus referencing an issue

Use a closing keyword such as `Fixes #123` only when merging this PR should close that issue.

Use a non-closing reference such as `Refs #123` when the PR contributes to a larger issue but does not complete it.

Never close an umbrella/tracking issue accidentally.

## Repository rules disagree with baseline NGHC

Treat the repository's explicit adopted variant as authoritative for that repository. Mention the divergence if it affects generated metadata, but do not "correct" a deliberate local convention back to generic NGHC.

## Remote mutation and history rewriting

Generating names/messages/descriptions is not permission to mutate GitHub or rewrite published Git history.

Before destructive or collaboration-affecting actions such as force-push, rebase of shared commits, commit amendment after publication, assignment, issue closure, or PR creation, follow the agent/platform's authorization rules and the user's explicit request.
