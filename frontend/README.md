# IzbiRdzaSpotkanie frontend

Nuxt 4 SSR client for event creation, joining and LettuceMeet-style availability comparison across
168 independent hourly services. It uses strict TypeScript and Tailwind CSS 4.

Live at <https://meet.izbi.beer>, against the production API at <https://api-meet.izbi.beer/docs>.

## Requirements and setup

- Node.js 22 or newer;
- npm 10 (the exact version is recorded in `package.json`);
- the development monolith on `http://localhost:8080` for manual use.

```bash
cd frontend
npm ci
npm run dev
```

The application listens on <http://localhost:3000>. On NixOS, a Node installation is not required:

```bash
nix-shell -p nodejs_22 --run 'cd frontend && npm run dev'
```

## Architecture

- `app/services/api/` contains validated gateway/slot transport and shared timeouts;
- `app/domain/` contains URL-independent aggregation, validation, slot and Garnek rules;
- `app/composables/` owns bounded reads, cancellation and serialized optimistic writes;
- `app/components/` and `app/pages/` render accessible UI and route/session flows;
- `sessionStorage` keeps only an event-scoped join session and dismissed dialogue signatures. JWT
  expiry decoding is a UX hint; protected API responses remain authoritative.

Every read cell represents a separate service. Reads are public, while PUT/DELETE use the current
event JWT. Failed reads remain unknown and are never interpreted as an empty response.

## Commands

```bash
npm run typecheck
npm test
npm run lint
npm run build
npm run test:e2e
```

Playwright starts the frontend and `tests/e2e/mock-api.mjs` itself. Install its Chromium once with
`npx playwright install chromium`. On NixOS use the browser package and point Playwright at its
store output, for example:

```bash
BROWSERS_PATH=$(nix-build '<nixpkgs>' -A playwright-driver.browsers --no-out-link)
PLAYWRIGHT_BROWSERS_PATH="$BROWSERS_PATH" nix-shell -p nodejs_22 --run 'cd frontend && npm run test:e2e'
```

If the local Nixpkgs attribute differs, obtain the browser package output with
`nix-store -q --outputs` and set `PLAYWRIGHT_BROWSERS_PATH` to that directory.

## API configuration

Configuration is centralized in Nuxt runtime config and the API URL builder. Components never
construct service URLs. Development defaults to the monolith:

```dotenv
NUXT_PUBLIC_API_MODE=development
NUXT_PUBLIC_GATEWAY_BASE=http://localhost:8080
```

Production mode uses `https://api-meet.izbi.beer` for the gateway and
`https://{slot}-meet.izbi.beer` for slots:

```dotenv
NUXT_PUBLIC_API_MODE=production
```

Staging or self-hosted deployments move slot hosts with a template; `{slot}` becomes the slot ID
(for example `pn-09`). The gateway override alone never changes production slot hosts:

```dotenv
NUXT_PUBLIC_API_MODE=production
NUXT_PUBLIC_GATEWAY_BASE=https://api-staging.example
NUXT_PUBLIC_SLOT_BASE_TEMPLATE=https://{slot}-staging.example
```

Never put JWTs or other secrets in `.env`; the browser receives an event-scoped JWT from join.

## Production deployment

Production runs on Dokploy as the `frontend` service of `backend/docker-compose.dokploy.yml`, from
the `ghcr.io/wikipop/meet-frontend` image that CI publishes on every push to `master`. The steps are
in [`backend/README.md`](../backend/README.md#dokploy). To build the image locally:

```bash
docker build -t meet-frontend:local frontend
```

Without Docker, build and run the Nuxt/Nitro Node server:

```bash
npm ci
npm run build
node .output/server/index.mjs
```

Configure the exact public frontend origin as `ALLOWED_ORIGIN` for the gateway and every slot
service. Do not ship development URL overrides. `NUXT_PUBLIC_*` settings are public by design and
must contain origins only, never credentials.

## Product constraints

The API has no dates, time zones or live updates, and cannot reveal participants with no slot
entry. The grid therefore refreshes explicitly, preserves partial failures, and enables Garnek's
no-overlap conclusion only after complete reads, settled writes, at least two visible selected
participants. The check starts automatically after a short pause following the first edit and the
dialog stays dismissed for the rest of that event session.
