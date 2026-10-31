# syntax=docker/dockerfile:1
# Build context: the frontend/ directory.

FROM node:22-alpine AS builder
WORKDIR /src
COPY package.json package-lock.json .nuxtrc ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY . .
# Type checking runs in CI; the image build only bundles.
RUN NUXT_TYPECHECK=false NODE_ENV=production npm run build

FROM node:22-alpine
WORKDIR /app
COPY --from=builder --chown=node:node /src/.output ./.output
ENV NODE_ENV=production HOST=0.0.0.0 PORT=3000 NUXT_PUBLIC_API_MODE=production
USER node
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["node", "-e", "fetch('http://127.0.0.1:3000/robots.txt').then(r => process.exit(r.ok ? 0 : 1), () => process.exit(1))"]
CMD ["node", ".output/server/index.mjs"]
