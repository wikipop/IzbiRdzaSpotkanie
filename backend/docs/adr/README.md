# Decyzje architektoniczne

Zapisujemy je w formacie Michaela Nygarda, [*Documenting Architecture Decisions*](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions).
Przyjętej decyzji nie edytujemy. Zmiana kursu oznacza nowy ADR, który zastępuje poprzedni.

| # | Decyzja | Status |
|---|---|---|
| [0001](0001-nano-service-per-hour.md) | Jedna usługa na godzinę tygodnia, każda z własnym SQLite | przyjęta |
| [0002](0002-frontend-aggregation-public-reads.md) | Agregacja po stronie frontendu, publiczne odczyty | przyjęta |
| [0003](0003-jwt-eddsa-local-verification.md) | JWT EdDSA weryfikowany lokalnie | przyjęta |
| [0004](0004-monolith-dev-mode.md) | Monolit na potrzeby developmentu | przyjęta |
| [0005](0005-first-level-subdomains.md) | Subdomeny pierwszego poziomu za Cloudflare | przyjęta, certyfikaty w 0006 |
| [0006](0006-cloudflare-origin-certificate.md) | Certyfikat Origin CA zamiast Let's Encrypt | przyjęta |

Nowy ADR dostaje kolejny numer i trzy sekcje: kontekst, decyzja, konsekwencje.
