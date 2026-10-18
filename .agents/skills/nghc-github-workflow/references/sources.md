# Sources and Provenance

This skill separates source convention from agent-specific workflow. Re-check upstream when materially changing the convention because NGHC describes itself as a living beta document.

## Primary NGHC sources

Snapshot inspected from `Norbiros/nghc` on 2026-08-21, default branch `master`; most recent repository commit observed: `fe6cd9c4b8c6249105305bf86e36fbe4c4754036`.

- NGHC README: https://github.com/Norbiros/nghc/blob/master/README.md
- Commit messages: https://github.com/Norbiros/nghc/blob/master/conventions/commit-messages.md
- Branches: https://github.com/Norbiros/nghc/blob/master/conventions/branches.md
- Pull requests: https://github.com/Norbiros/nghc/blob/master/conventions/prs.md
- Issues: https://github.com/Norbiros/nghc/blob/master/conventions/issue.md
- Contributor committing guide: https://github.com/Norbiros/nghc/blob/master/guide/contributors/1-committing.md
- Contributor PR guide: https://github.com/Norbiros/nghc/blob/master/guide/contributors/2-opening-prs.md
- Issue templates: https://github.com/Norbiros/nghc/tree/master/resources/ISSUE_TEMPLATE

## Skill-authoring sources

The structure follows the conventions already used by `Zerya-Dev/skills`:

- repository skill rules: https://github.com/Zerya-Dev/skills/blob/master/AGENTS.md
- example skill: https://github.com/Zerya-Dev/skills/blob/master/skills/dotnet/zerya-dotnet-ddd-architecture/SKILL.md

The agent-specific design also follows current skill-authoring guidance:

- OpenAI skill creator: https://github.com/openai/skills/blob/main/skills/.system/skill-creator/SKILL.md
- OpenAI Academy, Using skills: https://openai.com/academy/skills/
- Anthropic skill creator: https://github.com/anthropics/skills/blob/main/skills/skill-creator/SKILL.md
- Anthropic public skills repository / Agent Skills examples: https://github.com/anthropics/skills
- Conventional Commits 1.0.0: https://www.conventionalcommits.org/en/v1.0.0/

## Design choices derived from the research

- Keep one coherent capability: applying NGHC to a contribution lifecycle.
- Put activation conditions in frontmatter `description`, because discovery depends on metadata.
- Keep `SKILL.md` concise and procedural.
- Move detailed convention facts and ambiguities to `references/` for progressive disclosure.
- Prefer repository evidence over generic defaults.
- Add explicit negative boundaries so the agent does not invent scopes, issue numbers, test results, reviewers, or authorization.
- Preserve one semantic classification across issue, branch, commits, and PR while still allowing independent commits to use different valid types.
