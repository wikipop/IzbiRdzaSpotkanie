# Frontend implementation plan

This is the authoritative living roadmap for the frontend. Update the checkboxes and the
decision/risk log as implementation progresses. The Rust backend, its ADRs, and its API
contracts remain authoritative and out of scope for modification.

## Status

- [x] Read the project instructions and repository overview.
- [x] Inspect the checked-in API documentation, relevant ADRs, protocol DTOs, and ambiguous
      gateway/nano implementation details (2026-10-27).
- [x] Define the initial frontend architecture, API boundaries, state model, visual direction,
      accessibility expectations, and implementation phases.
- [x] Re-check runtime gateway and representative slot OpenAPI documents before implementing
      transport types (2026-10-27); their route/schema surface matched the checked-in contract.
- [x] Complete the first vertical slice: Nuxt/Tailwind scaffold, typed gateway/slot clients,
      create-event flow, event metadata states, join/session flow, and visual foundation.
- [x] Implement the frontend in the phases below.

## Product scope and principles

Build a polished, accessible, responsive When2meet-like client under `frontend/` with Nuxt 4,
strict TypeScript, and Tailwind CSS. The browser owns slot aggregation and the heatmap. It must keep
three facts distinct at all times:

1. no availability entry means “unavailable”;
2. a failed or timed-out slot request means “unknown”;
3. a pending local write means “not confirmed yet”.

The main experience is an original pastel literature-club visual novel aesthetic inspired by
the mood requested for the project: stationery, notebook panels, soft pinks, expressive dialogue
cards, and playful transitions. Do not copy Doki Doki Literature Club logos, characters, artwork,
dialogue, fonts, or other protected assets. Visual styling must not reduce legibility, obscure
network state, or rely on color alone.

## Verified backend contract

### Environments and URL ownership

| Concern | Development monolith | Production |
|---|---|---|
| Gateway | `http://localhost:8080` | `https://api-meet.izbi.beer` |
| Slot `sr-09` | `http://localhost:8080/slots/sr-09` | `https://sr-09-meet.izbi.beer` |

Only one environment-aware URL builder may know these rules. Components and domain code must
never concatenate API origins or slot subdomains. Public slot GETs must not include the JWT.

### Gateway endpoints

| Method and path | Request | Success |
|---|---|---|
| `POST /events` | `CreateEventRequest` below | `201 { event_id }` |
| `GET /events/{eid}` | none | `200 { event_id, name, slots }` |
| `POST /events/{eid}/join` | `{ nick, password? }` | `200 { user_id, token }` |
| `GET /health` | none | `200 { status: "ok" }` |

`CreateEventRequest` is exactly one of:

```ts
type CreateEventRequest =
  | { name: string; slots: string[] }
  | { name: string; days: Weekday[]; hours: { from: number; to: number } }
```

- Weekdays are the case-sensitive values `pn`, `wt`, `sr`, `cz`, `pt`, `sb`, `nd`.
- Slots use `{weekday}-{two digit hour}`, for example `sr-09`; valid hours are `00`–`23`.
- `hours.to` is exclusive and the range must satisfy `0 <= from < to <= 24`.
- Event names are trimmed and contain 1–100 Unicode characters.
- An event needs at least one slot. Duplicate days/slots collapse.
- Returned slots are normalized Monday 00 through Sunday 23, not request order.
- Events are recurring weekday/hour models: the API has no calendar date, timezone, duration
  beyond one hour, or edit-event endpoint.

Joining accepts a trimmed 1–32 character Unicode nick with no control characters. Nick matching
is case-insensitive per event, while the first spelling is retained. Passwords are optional,
untrimmed, and at most 128 characters. Rejoining an unprotected nick returns that participant
even if a password is supplied; a protected nick requires the correct password.

### Per-slot endpoints

For every slot returned by the gateway event response:

| Method and path | Auth/body | Success |
|---|---|---|
| `GET /events/{eid}` | no auth | `200 { slot, entries }` |
| `PUT /events/{eid}/me` | Bearer JWT; `{ state: "yes" | "maybe" }` | `200 AvailabilityEntry` |
| `DELETE /events/{eid}/me` | Bearer JWT | `204`, idempotent |

```ts
type AvailabilityState = 'yes' | 'maybe'
type AvailabilityEntry = {
  user_id: string
  nick: string
  state: AvailabilityState
  updated_at: number // Unix seconds
}
```

There is no stored `absent` entry: absence is represented by DELETE/no entry. Each write is an
independent upsert/delete with no batch endpoint, transaction, rollback, revision, conflict
response, or live-update channel. `updated_at` has one-second precision on independent services
and must not be treated as a global revision.

### Authentication and errors

- The join response carries an EdDSA JWT in JSON; the API does not set a cookie.
- Default token lifetime is 14 days but is server-configurable. There is no refresh or revocation
  endpoint. Rejoining issues a new token, and key rotation/restart can invalidate one earlier.
- A write must treat `invalid_token` as authoritative even if the decoded `exp` looks valid.
- Never log, render, send to analytics, or attach the token to public reads.
- API errors are `{ error: { code, message } }`; control flow must use `error.code`, never match
  human-readable messages.
- Known codes are `invalid_body`, `invalid_event_name`, `invalid_slot`, `invalid_nick`,
  `invalid_password`, `invalid_event_id`, `missing_token`, `invalid_token`,
  `invalid_credentials`, `event_mismatch`, `slot_out_of_range`, `event_not_found`, and `internal`.

### Architectural limits the UI must expose honestly

- There are 168 independent hourly services. One outage must not blank the whole grid.
- A nano service cannot verify event existence; a valid unknown event ID can return an empty list.
- There is no participant-list endpoint. Participants are the union of entries returned by
  successfully loaded slots; a joined participant with no selections is invisible to viewers.
- There is no server-side aggregate view, polling contract, push channel, or cross-slot
  consistency guarantee. Refresh is explicit or client-managed.

## Frontend architecture

### Runtime and dependency baseline

- Nuxt 4 using its standard `app/` directory, Vue 3 Composition API, and strict TypeScript.
- Tailwind CSS 4 through the official Vite plugin, with semantic CSS design tokens in the main
  stylesheet. Confirm compatibility against the actual Nuxt release during initialization.
- Keep runtime dependencies minimal. Prefer native Nuxt/Vue statepierwszy and `$fetch`/`ofetch`; add a
  state library only if implementation demonstrates a real cross-route need.
- Vitest and Nuxt test utilities for unit/component tests; Playwright for a small set of keyboard,
  touch, persistence, and failure-state journeys.
- Pin and document the Node version and package manager when the scaffold is created.

### Proposed directory boundaries

```text
frontend/
├── app/
│   ├── assets/css/             # Tailwind import, semantic tokens, textures, motion rules
│   ├── components/
│   │   ├── base/               # buttons, fields, dialog, status badges
│   │   ├── event/              # creation and join UI
│   │   ├── schedule/           # grid, cells, legend, service state, participant details
│   │   └── garnek/             # original mascot presentation and dialogue panel
│   ├── composables/            # page orchestration and browser session state
│   ├── domain/                 # pure slot parsing, aggregation, color depth, overlap rules
│   ├── pages/                  # landing/create and event routes
│   ├── services/api/           # URL builder, gateway client, slot client, error mapping
│   ├── stores/                 # only if a store is justified during implementation
│   ├── types/api/              # DTOs verified against OpenAPI/Rust proto
│   ├── utils/                  # framework-neutral helpers
│   ├── app.vue
│   └── error.vue
├── public/                     # original static assets only
├── tests/                      # fixtures, integration, accessibility, E2E
├── nuxt.config.ts
└── package.json
```

Dependency direction is one-way: pages/components → composables → domain and API services.
Transport DTOs remain separate from view models. Domain functions must be deterministic and
framework-free so aggregation, overlap, and heatmap projection can be exhaustively unit tested.

### Routes and page responsibilities

- `/`: product introduction plus create-event form. Creation supports a compact day/hour range
  first, with an explicit-slot editor as an advanced mode. On `201`, navigate to the event route.
- `/e/[eventId]`: load gateway metadata, offer join/rejoin, aggregate slots, edit the current
  participant’s availability, show participant-aware results, refresh, and share the canonical URL.
- Unknown/malformed IDs get a dedicated recoverable state based on gateway `event_not_found`;
  do not infer validity from slot responses.

### API and state design

1. `urlBuilder` receives environment config and produces gateway/slot URLs. In development it
   mounts slot paths on the monolith; in production it emits slot subdomains.
2. `gatewayClient` owns create/get/join calls; `slotClient` owns public reads and authenticated
   PUT/DELETE. Both normalize network/timeout/HTTP/API-code failures into typed results.
3. `useEventSchedule` runs bounded-concurrency slot reads, records every slot result separately,
   and exposes progress. Start with a measured concurrency limit rather than firing all 168 at
   once; tune it from browser/network testing.
4. The aggregator builds participant and cell view models only from successful responses while
   preserving an `unknown` mask for failed slots. The heatmap never counts unknown as absent.
5. Local editing uses a three-state cycle (`yes`, `maybe`, `absent`) and keeps confirmed state,
   desired state, and per-slot write state separate. Coalesce rapid edits per cell, serialize a
   cell’s writes, and retry only failed desired writes. Never claim a multi-cell save succeeded
   atomically.
6. Manual refresh is always available. Optional low-frequency refetch may be considered only
   after the explicit refresh flow works and preserves unsaved local edits.

Recommended state machines:

- Event load: `idle → loading → ready | not-found | failed`.
- Slot read: `queued → loading → success | failed`; retry transitions only failed slots.
- Cell write: `confirmed → dirty → saving → confirmed | failed`; another edit updates `desired`
  without losing the last confirmed server value.
- Session: `anonymous → joining → authenticated | join-failed`; any authoritative token rejection
  transitions back to `reauth-required` without deleting unsaved desired selections.

### Token persistence decision

Use a per-event browser session adapter rather than reading storage directly throughout the app.
Initial implementation should default to `sessionStorage` to limit persistence and SSR exposure,
namespaced by event ID, storing `{ userId, nick, token }`. Keep the JWT in memory after hydration.
Decode `exp` only to warn before a likely-expired write; signature/validity remains server-owned.
Provide “forget this session” and a rejoin flow. Do not put tokens in cookies, route params, logs,
Nuxt payloads, error telemetry, or SSR requests. Reassess `localStorage` only with an explicit UX
decision and documented XSS trade-off.

## Scheduling and heatmap behavior

- Desktop: days as columns and hours as rows, limited to the event’s configured slots.
- Mobile: maintain large touch targets and sticky labels; permit intentional horizontal/vertical
  scrolling without drag-paint hijacking page scroll.
- Keyboard: one grid tab stop with arrow-key navigation; Space/Enter cycles the focused cell;
  row/column labels are announced; Home/End behavior is documented and tested.
- Mouse/pen painting applies one chosen state across a gesture and supports reversing that state.
  Touch keeps native panning and uses tap; pointer painting never replaces click/keyboard operation.
- Encode `yes`, `maybe`, `absent`, `unknown`, `saving`, and `failed` using text/icon/pattern plus
  color. Unknown/failure cells are never editable aggregate facts.
- Do not render a ranked list of hours. Like LettuceMeet, the heatmap itself is the result: darker
  cells mean more visible participants are available, and selecting a cell shows participant names.
- Participant chips/filtering are derived from the union of successful slot entries and must carry
  a note that people with no selected availability cannot be discovered by this API.

## Garnek no-overlap feature

Garnek is an original, friendly pot-shaped club mascot presented in a visual-novel dialogue panel.
Its original neutral and sad/pleading artwork lives with a provenance note under
`frontend/public/assets/garnek/`; no DDLC imagery is reused.

Define the trigger precisely to keep it useful and truthful:

1. the current user is authenticated and at least two participants are visible in successful slot
   data (the display filter does not narrow Garnek's group-wide check);
2. every selected participant has at least one confirmed `yes` or `maybe` selection;
3. all event slot reads have settled successfully and no current write is pending;
4. there is no slot where every selected participant has `yes` or `maybe`;
5. the current user has made their first availability edit and the short automatic-check delay has
   elapsed.

If any slot read failed, Garnek must not claim there is no overlap. Show an inline “cannot determine
yet” status with retry instead. Do not show the popup merely because the current user has selected
nothing, while loading, immediately on join, or for participants invisible to the API.

Interaction details:

- Arm on the first availability edit, wait briefly, and evaluate only after settled save feedback.
  Once the dialogue is dismissed, never show it again in the stored event session—even if the
  result or participant scope changes.
- The original Polish copy has Garnek gently plead for another attempt because it is sad not
  everyone can attend together. Keep copy in a small content map so tone can be refined without
  changing logic.
- Offer `Wróć do planu` as the primary action, an optional “show closest slots” action, and a clear
  dismiss control. Restore focus to the cell or control that led to the result.
- Use an accessible dialog only when it truly interrupts after a user action; otherwise use a
  polite status/dialogue card. Never trap the user in repeated mascot prompts.
- Respect `prefers-reduced-motion`; animation is decorative, short, cancellable, and never carries
  the only explanation.
- Unit test the overlap predicate separately from popup frequency/session behavior.

Garnek evaluates all visible participants independently of the display filter. This keeps the
default self-focused editing view separate from the group-wide meaning of “everyone”.

## Visual system and accessibility

### Direction

- Palette: warm paper/cream canvas, blush and sakura pink surfaces, berry/magenta primary actions,
  lavender accents, ink-like near-black text, and distinct semantic amber/red/green tokens.
- Components: notebook cards, subtle ruled-paper or dot textures, sticker-like badges, ribbon tabs,
  expressive but restrained dialogue boxes, and softly layered shadows.
- Typography: highly readable open-licensed body and display families with Polish glyph coverage;
  avoid novelty/script fonts for forms, grids, error copy, and dense data.
- Illustration: original Garnek artwork and small original motifs. Maintain an asset provenance file.
- Tone: warm and playful in headings/empty states, direct and calm in auth, save, and failure copy.

### Non-negotiable accessibility checks

- WCAG 2.2 AA contrast for text, controls, focus rings, and every availability state.
- Full keyboard operation, visible focus, logical focus return, skip link, semantic headings, and
  no keyboard traps.
- Minimum 44×44 CSS-pixel touch targets where practical; zoom and 320px-wide layouts remain usable.
- Grid semantics tested with screen readers; use a simpler semantic table/list fallback if an ARIA
  grid becomes less understandable.
- Status updates use restrained live regions; a bulk load/save must not announce 168 messages.
- Errors appear next to affected controls/cells and in a concise summary; color is supplementary.
- Reduced-motion and high-contrast/forced-colors behavior are tested.

## Implementation roadmap

### Phase 0 — Contract snapshot and project scaffold

- [x] Start the backend and compare gateway plus representative slot OpenAPI documents.
- [x] Resolve any mismatch against Rust DTOs before frontend types are committed (none found).
- [x] Initialize `frontend/` with Nuxt 4, strict TypeScript, Tailwind CSS 4, lint/typecheck, and
      unit/component-test foundations. Playwright E2E setup remains in Phase 6.
- [x] Add runtime config and the centralized environment URL builder with tests.
- [x] Add verified DTOs, runtime response guards, and a discriminated typed API error/result model.
- [x] Establish semantic design tokens, global focus/motion rules, and an asset provenance file.

Exit: a minimal route builds and the URL/DTO/error tests pass; no product flow is required yet.

### Phase 1 — Event creation and metadata

- [x] Implement event-name, weekday, and exclusive hour-range domain validation matching backend
      behavior; server errors remain authoritative.
- [x] Build an accessible create form with pending, validation, server-error, retry, and duplicate
      submission protection.
- [x] Add advanced explicit-slot selection without allowing an empty event.
- [x] Navigate to `/e/[eventId]`, load gateway metadata first, and render not-found/network states.
- [x] Add canonical share/copy interaction with assistive success and fallback feedback.

Exit: an event can be created and reopened by URL against the development backend.

### Phase 2 — Join, session, and recovery

- [x] Build join/rejoin form with nick/password rules and code-based API error messages.
- [x] Implement the per-event session adapter, in-memory hydration, expiry warning, forget-session,
      and `reauth-required` recovery while preserving unsaved desired edits.
- [x] Verify tokens never reach logs, public GET headers, URLs, SSR payloads, or test snapshots.
- [x] Document password semantics so users do not assume adding a password protects an existing
      unprotected nick.

Exit: a participant can join/rejoin safely and recover from invalid/expired credentials.

### Phase 3 — Public aggregation and heatmap results

- [x] Fetch event slots with bounded concurrency, progress, per-slot timeout, cancellation on route
      change, and retry of failed slots only.
- [x] Implement pure slot normalization, participant union, grid projection, coverage, and color
      depth functions. The no-overlap predicate remains deliberately scoped to the Garnek phase.
- [x] Render partial data while explicitly marking unknown slots and showing a concise failure
      summary plus retry controls.
- [x] Add manual refresh that does not erase the authenticated session.
- [x] Document that no-selection participants are undiscoverable and data is not live.

Exit: one or many slot outages degrade locally, never masquerade as absence, and the heatmap states
its data coverage.

### Phase 4 — Editable accessible schedule

- [x] Build responsive schedule navigation for keyboard, touch, pointer, and screen reader use.
- [x] Implement `yes → maybe → absent` editing with confirmed/desired/write states per slot.
- [x] Add optimistic visual intent plus explicit saving/failed/retry indicators; serialize or
      coalesce rapid writes per cell and ignore stale responses.
- [x] Handle partial multi-cell failures without rollback claims or loss of successful writes.
- [x] Preserve edits across reauthentication and reconcile them after a successful refetch.

Exit: all states can be edited accessibly, failures can be retried per slot, and server-confirmed
state remains distinguishable from local intent.

### Phase 5 — Garnek and literature-club polish

- [x] Add original neutral and sad/pleading Garnek assets, responsive presentation, decorative
      alt-text policy, generated source files, and a provenance record.
- [x] Implement and test the selected-group no-overlap eligibility predicate exactly as specified
      above.
- [x] Add the once-per-event-session dialogue experience, focus management, dismissal, and
      reduced-motion behavior.
- [x] Apply the visual system consistently to all loading, auth, error, partial-data, empty, and
      success states—not only the happy path.

Exit: Garnek appears only on trustworthy data, is original and accessible, and never blocks work.

### Phase 6 — Hardening and release readiness

- [x] Unit-test validators, slot ordering, URL generation, typed error mapping, aggregation,
      heatmap depth, unknown coverage, token-expiry hints, write coalescing, and Garnek predicates.
- [x] Integration-test create/join/load/edit/delete, malformed API payloads, token rejection recovery,
      timeouts, mixed slot failures, stale writes, and refresh during unsaved changes.
- [x] E2E-test narrow mobile/touch, keyboard-only grid use, reload/session recovery, reduced motion,
      and a full no-overlap-to-Garnek journey.
- [x] Run automated axe checks plus real-browser keyboard, touch, focus, live-region, and semantic
      screen-reader-oriented smoke tests.
- [x] Check production URL construction/CORS requirements, performance with all 168 slots, bundle output, no-secret
      scans, typecheck, lint, unit/component/E2E tests, and production build.
- [x] Perform a read-only code/API integration review after the meaningful feature set is complete.
- [x] Update this document with completed work, deviations, verification output, and open risks.

Exit: relevant checks pass, remaining failures/risks are recorded accurately, and production config
contains no development URL assumptions or credentials.

## Test matrix minimums

| Area | Required cases |
|---|---|
| URL builder | dev gateway, dev slot path, prod gateway, all valid prod slot hosts, reject invalid slot |
| Creation | both request variants, exclusive `to`, duplicates, empty selection, Unicode length |
| Join/auth | protected/unprotected nick, invalid credentials, expired/invalid token, event/slot mismatch |
| Reads | all success, some failed, all failed, retry subset, cancellation, malformed body |
| Aggregation | absent vs unknown, invisible no-entry participant limitation, deterministic ordering |
| Writes | PUT yes/maybe, DELETE absent, rapid edits, out-of-order response, partial failure, retry |
| Grid access | arrows, Space/Enter, focus visibility/return, touch target, scroll, screen-reader labels |
| Garnek | overlap, no overlap, no own entries, no other visible participant, partial read failure, pending write, delayed automatic check, once-per-session dismissal |

## Decision log

- **2026-10-27 — Framework:** use Nuxt 4 + strict TypeScript + Tailwind CSS as requested; there is
  no existing frontend whose conventions need migration.
- **2026-10-27 — Transport boundary:** use separate gateway and slot clients behind one URL builder;
  domain functions never know origins or auth headers.
- **2026-10-27 — Source of truth:** use checked-in Rust/API contract for planning, then compare runtime
  OpenAPI before implementation because the local backend was unavailable during this review.
- **2026-10-27 — Session:** default to event-scoped `sessionStorage` plus in-memory state; server
  write responses are authoritative for token validity.
- **2026-10-27 — Styling:** use an original literature-club/visual-novel system rather than copied
  DDLC assets or exact trade dress; accessibility tokens override decorative preferences.
- **2026-10-27 — No-overlap:** Garnek compares every visible participant only when
  every slot is known and writes are settled.
- **2026-10-27 — First increment:** Nuxt 4.6, Tailwind CSS 4, typed transport guards, event
  creation/loading, join, event-scoped session storage with memory fallback, and the initial
  literature-club UI are implemented. A read-only review found no blocking/high-severity issues;
  its valid storage, route isolation, Unicode, language, accessibility, and test findings were fixed.
- **2026-10-27 — Public schedule:** slot reads use an eight-request concurrency limit, ten-second
  per-slot timeout, route-change cancellation, explicit unknown states, and failed-only retry.
- **2026-10-27 — Editable schedule:** the grid fits its card from the `md` breakpoint while retaining
  deliberate horizontal scrolling on narrow phones. Aggregate availability uses four pink-depth
  levels plus non-color dot markers and exact screen-reader labels. Authenticated cells use a
  roving tab stop, arrow/Home/End navigation, per-slot serialized writes, retryable local intent,
  reauthentication recovery, and protection against stale reads overwriting confirmed writes.
- **2026-10-27 — LettuceMeet-style result view:** remove ranked-hour output entirely; the heatmap is
  the result. Give the grid almost the full viewport width, move join/rejoin into a compact modal,
  show participant details for a selected cell, support mouse/pen painting with touch panning and
  tapping, and expose the last read status of every independent hourly service.
- **2026-10-27 — Participant comparison and Garnek:** participant chips now filter both heatmap and
  cell details, support multi-person comparison, and keep editing available only when the current
  participant is included. Garnek evaluates whether every visible person shares a
  confirmed hour, waits for complete reads and settled writes, and shows an original visual-novel
  dialogue at most once per stored event session.

## Verification log

### 2026-10-27 — First frontend increment

- Runtime gateway and `pn-00` slot OpenAPI documents fetched successfully and matched the planned
  path/schema surface.
- Live monolith smoke test passed for event creation, normalized metadata retrieval, and joining.
- Nuxt SSR smoke tests returned `200` and rendered both the landing page and live event metadata.
- `npm run typecheck`: passed.
- `npm test`: 6 files, 28 tests passed.
- `npm run lint`: passed with no warnings after review fixes.
- `npm run build`: production client, server, and Nitro build passed.
- Repository scan found no committed JWT, smoke-test password, or bearer credential.

### 2026-10-27 — Public aggregation increment

- Landing copy shortened and the example changed to “Wspólne izbienie”.
- Live monolith smoke test passed with seeded `yes` and `maybe` entries on separate slot services.
- Pure aggregation, the bounded-concurrency utility, slot client authorization, grid,
  retry, and partial-data presentation are covered by unit/component tests. Full composable
  lifecycle/route-race coverage remains part of integration hardening.
- `npm run typecheck`, `npm test` (9 files, 36 tests), `npm run lint`, and `npm run build`:
  passed after review fixes.
- Final read-only re-review confirmed the join-navigation race, duplicate hydration load,
  misleading progress count, and excessive live-region findings are resolved with no issues left.

### 2026-10-27 — Responsive and editable schedule increment

- Removed laptop/tablet horizontal overflow by allowing the fixed-layout grid to fit its container
  from `md`; the mobile minimum width remains intentional for usable touch targets.
- Replaced visible per-cell counts with weighted four-level color depth. Four dot markers, exact
  accessible names, and participant details retain the information without relying on color alone.
- Added `yes → maybe → absent` writes, per-cell saving/failure/retry state, rapid-edit coalescing,
  stale route/write protection, reauthentication retry, and refresh/write reconciliation.
- Live monolith smoke test passed for authenticated `PUT`, public `GET`, and idempotent `DELETE`.
- `npm run typecheck`, `npm test` (10 files, 44 tests), `npm run lint`, and `npm run build`: passed.
- Read-only review found a rapid-edit failure race, stale roving tab stop, missing editable depth
  markers, and incomplete Home/End coverage. All four were fixed with regression tests; re-review
  found no remaining issues in those areas.

### 2026-10-27 — Full-width heatmap interaction

- Removed the ranked-hours component and ranking computation; the aggregate heatmap and selected
  cell participant breakdown are now the only result presentation.
- Replaced the permanent join sidebar with a compact modal and expanded the schedule container to
  `96rem`, keeping only small session/action controls above the grid.
- Added per-cell green/red/loading service indicators with honest last-read wording, mouse/pen
  drag painting, native touch panning plus tap editing, and explicit dialog focus restoration.
- `npm run typecheck`, `npm test` (10 files, 43 tests), `npm run lint`, and `npm run build`: passed.
- Read-only review caught touch-scroll capture, unreliable dialog focus return, over-announced hover
  details, and overly narrow failure wording. All were fixed; focused re-review found no remaining
  issues in those areas.

### 2026-10-27 — Participant comparison and Garnek dialogue

- Added single- and multi-participant filters; the heatmap, color depth, accessible counts, and the
  right-side selected-cell inspector all use the same explicit scope.
- Added a truthful selected-group overlap predicate that refuses to decide with failed slot reads,
  pending writes, fewer than two selected people, a missing current participant, or a selected
  participant with no known availability.
- Added an accessible native visual-novel dialogue with original neutral and sad/pleading Garnek
  artwork, once-per-session dismissal, and grid focus return.
- The first read-only review found that refresh could clear the selected comparison and the asset
  manifest was stale. Selection now survives transient loads and both asset manifests agree.
- Final `npm run typecheck`, `npm test` (12 files, 54 tests), `npm run lint`, and `npm run build`:
  passed.
- Follow-up UX correction removed the duplicate header availability action. Garnek no longer treats
  a still-pending write as a finished plan.

### 2026-10-29 — Release-ready frontend

- Added advanced creation with an exact, non-contiguous 168-hour editor, bulk-day controls and a
  single roving keyboard tab stop. Both creation request variants are verified against typed
  contracts; the E2E mock rejects any unexpected explicit-slot payload.
- Hardened event-scoped sessions with safe JWT-expiry hints, different-identity write isolation,
  same-identity reauthentication, persisted Garnek dismissals and storage-denial fallback. Closing
  join invalidates late responses, while SSR payload reuse avoids a duplicate hydration request.
- Applied a shared transport timeout to gateway reads/writes and slot calls, strict API-mode
  validation, a production-safe Nuxt configuration and a global Polish error page.
- Completed grid accessibility hardening: failed cells participate in arrow navigation, retry keeps
  keyboard focus, hover does not spam the live region, and the mobile inspector stays visible.
- Automated checks cover the full 168-service load at an eight-request ceiling, timeout handling,
  exact creation payloads, session boundaries and focus regressions. Playwright passed all six
  desktop/mobile scenarios, including axe, touch, keyboard, reload, reduced motion and Garnek.
- Updated root/frontend runbooks with architecture, local setup, API configuration, verification,
  SSR deployment, CORS and backend limitations.
- Replaced the manual “sprawdź plan” action with a delayed automatic check armed by the first
  availability edit. The check waits for complete reads and settled writes; once dismissed, Garnek
  remains silent for the rest of the stored event session, even if the result later changes.
- After joining or restoring a session, participant filtering starts on the current user so their
  own availability is visually isolated; they can then add specific people or switch to everyone.
- Final review found two medium issues: aggregate retry focus loss and acceptance of a valid slot
  response for the wrong requested hour. Focus now moves to the schedule heading before the retry
  control disappears, and the slot client rejects mismatched response identifiers; no blocker or
  high finding remained.
- Final verification: `npm run typecheck`, `npm run lint` and `npm run build` passed; Vitest passed
  14 files / 67 tests; Playwright previously passed 6/6 desktop and mobile scenarios. The latest
  local rerun was interrupted before the application scenario by the helper mock API process
  exiting; unit/component checks and the production build remained green. The production Nitro
  node-server output is 2.17 MB (549 kB gzip), and an application/build scan found no embedded test
  token or bearer credential.

## Open risks and questions

- The API cannot reveal joined users with no entries, dates/timezones, or a globally consistent
  participant snapshot. UI copy must not imply otherwise.
- The eight-request read ceiling and transport timeouts are regression-tested, but should still be
  observed under real production latency before tuning.
- Session storage intentionally loses identity when the tab/session ends; longer-lived credentials
  would increase exposure and require an explicit product/security decision.
- Deployment is defined as Nuxt SSR, but the exact public frontend origin remains an external
  deployment choice and must be configured consistently in backend CORS.
- Original Garnek artwork and Polish dialogue now have implementation provenance but still need
  product/design approval before release; font licenses also need final verification.
- Automated axe, semantic labels and real-browser keyboard/touch checks pass. A manual NVDA or
  VoiceOver pass is still recommended before a public release.
- `npm audit --omit=dev` currently reports seven low-severity transitive findings in the Nuxt/Nitro
  dependency tree; the offered automatic fix is a forced Nuxt downgrade, so it was not applied.

## Research sources

- `AGENTS.md`, `README.md`, and `backend/README.md`
- `backend/docs/adr/0001` through `0005` (0006 is infrastructure-only for this frontend plan)
- `backend/crates/proto/src/{gateway,nano,error,json}.rs`
- Relevant gateway, nano, auth, domain, and CORS implementation used only to resolve documented
  ambiguities
- Nuxt 4 official documentation for the standard `app/` structure and auto-import boundaries
- Tailwind CSS official documentation for the current Vite plugin integration
