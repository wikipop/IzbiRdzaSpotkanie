# IzbiRdzaSpotkanie

Aplikacja pomaga grupie znaleźć wspólny termin spotkania. Działa w stylu When2meet.
To zadanie rekrutacyjne BIT Web w ścieżce programistycznej.

| Część | Gdzie | Odpowiedzialny |
|---|---|---|
| Frontend w Nuxt 4 | [`frontend/`](frontend/README.md) | [wojpo](https://github.com/wojpo) |
| Backend: gateway i 168 usług nano w Ruście | [`backend/`](backend/README.md) | [wikipop](https://github.com/wikipop) |
| Decyzje architektoniczne | [`backend/docs/adr`](backend/docs/adr/README.md) | [wikipop](https://github.com/wikipop) |

## Szybki start

```bash
cd frontend && npm ci && npm run dev
```

```bash
cd backend && cargo run -p monolith
```

Frontend działa pod <http://localhost:3000>, a dokumentacja API jest pod
<http://localhost:8080/docs>.

Frontend domyślnie łączy się z monolitem pod `http://localhost:8080`, więc oba procesy
muszą działać jednocześnie. Adresy API zmienia się w `frontend/.env`
(wzór w [`frontend/.env.example`](frontend/.env.example)).

## Jak działa aplikacja

1. Na stronie głównej tworzysz spotkanie: zakres godzin w wybrane dni albo konkretne godziny.
2. Udostępniasz link `/e/{id}`. Każdy dołącza pseudonimem i opcjonalnym hasłem.
3. Zaznaczasz godziny kliknięciem albo przeciąganiem: „tak”, „może” albo brak wpisu.
4. Mapa ciepła pokazuje, ilu osobom pasuje każda godzina. Filtr uczestników porównuje wybrane osoby.
5. Gdy nie ma żadnej wspólnej godziny dla całej grupy, Garnek raz prosi o poprawienie planu.

Każda godzina to osobna usługa, więc frontend czyta je niezależnie. Niedostępna godzina jest
oznaczona jako nieznana i można ją ponowić, zamiast traktować ją jak pustą.

## Testy

```bash
cd frontend && npm run typecheck && npm run lint && npm test
```

```bash
cd backend && cargo test
```

Testy end-to-end frontendu (`npm run test:e2e`) uruchamiają własny mock API, opis jest w
[`frontend/README.md`](frontend/README.md).

## Użycie AI

Kod i dokumentacja powstały z pomocą Claude Code i Codexa.

Pomysł, architektura i infrastruktura powstały bez AI. Architektura opisana
w ADR została zaprojektowana przed implementacją.

Autor przejrzał i przepisał znaczną część backendu.
