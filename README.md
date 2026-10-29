# IzbiRdzaSpotkanie

Aplikacja pomaga grupie znaleźć wspólny termin spotkania. Działa w stylu When2meet.
To zadanie rekrutacyjne BIT Web w ścieżce programistycznej.

| Część | Gdzie |
|---|---|
| Frontend w Nuxt 4 | [`frontend/`](frontend/README.md) |
| Backend: gateway i 168 usług nano w Ruście | [`backend/`](backend/README.md) |
| Decyzje architektoniczne | [`backend/docs/adr`](backend/docs/adr/README.md) |

## Szybki start

```bash
cd frontend && npm ci && npm run dev
```

```bash
cd backend && cargo run -p monolith
```

Frontend działa pod <http://localhost:3000>, a dokumentacja API jest pod
<http://localhost:8080/docs>.

## Użycie AI

Kod i dokumentacja powstały z pomocą Claude Code i Codexa.

Pomysł, architektura i infrastruktura powstały bez AI. Architektura opisana
w ADR została zaprojektowana przed implementacją.

Autor przejrzał i przepisał znaczną część backendu.
