# Backend

Backend składa się z gatewaya i 168 usług nano w Ruście. Każda usługa nano
obsługuje jedną godzinę tygodnia. Uzasadnienie jest w [`docs/adr`](docs/adr/README.md).

```
frontend (meet.izbi.beer)           scatter-gather po stronie przeglądarki
   │
   ├──► gateway  api-meet.izbi.beer      wydarzenia, uczestnicy, JWT
   │
   ├──► nano     pn-00-meet.izbi.beer    jedna godzina tygodnia
   │    …        (168 instancji)
   └──► nano     nd-23-meet.izbi.beer
```

Gateway i usługi nano nie komunikują się ze sobą. Każda usługa ma własny plik
SQLite. Wszystko o danej godzinie, czyli dostępność i nicki, żyje w usłudze
tej godziny. Mapę dostępności i ranking terminów liczy frontend. System nie
wysyła aktualizacji na żywo.

## Uruchomienie

Potrzebny jest Rust w wersji 1.90 lub nowszej. Polecenia uruchamiamy
z katalogu `backend/`.

```bash
cargo run -p monolith
```

Monolit uruchamia cały system na jednym porcie. Gateway jest pod
`http://localhost:8080/`, a slot `X` pod `http://localhost:8080/slots/X/`.
Każdy slot ma własną bazę w `./data/`.

Jeśli klucze nie są skonfigurowane, monolit generuje jednorazową parę.
Po restarcie tokeny tracą ważność i trzeba dołączyć ponownie. Na macOS
najpierw podnieś limit deskryptorów poleceniem `ulimit -n 4096`.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

### Docker

```bash
cargo xtask gen-keys          # zapisuje .env ze świeżą parą Ed25519, nie commituj go
docker compose up --build     # gateway i 168 usług nano
```

Na hoście publikowany jest tylko gateway pod `localhost:8080`. Wybrane sloty
można wystawić poleceniem `cargo xtask gen-compose --expose-slots pn-09,sr-10`.
Slot `X` dostaje wtedy port `9000 + index(X)`.

Pliki compose są generowane. Zmiany wprowadzaj w [`xtask/src/compose.rs`](xtask/src/compose.rs), a nie w YAML.

### Dokploy

1. CI publikuje obrazy `ghcr.io/wikipop/meet-gateway`, `ghcr.io/wikipop/meet-nano` i `ghcr.io/wikipop/meet-frontend` przy każdym pushu na `master`.
2. W Cloudflare rekord `*.izbi.beer` wskazuje na serwer Dokploy i jest proxowany. Tryb SSL to Full (strict).
3. Wygeneruj w Cloudflare certyfikat Origin CA dla `izbi.beer` i `*.izbi.beer`. Znajdziesz to w SSL/TLS, w zakładce Origin Server.
   Dodaj certyfikat i klucz w Dokploy, w zakładce Certificates. Robisz to raz. Uzasadnienie jest w [ADR 0006](docs/adr/0006-cloudflare-origin-certificate.md).
4. Utwórz aplikację Docker Compose z pliku `backend/docker-compose.dokploy.yml` i ustaw zmienne:

   | Zmienna | Wartość |
   |---|---|
   | `FRONTEND_IMAGE` | `ghcr.io/wikipop/meet-frontend:latest` |
   | `GATEWAY_IMAGE` | `ghcr.io/wikipop/meet-gateway:latest` |
   | `NANO_IMAGE` | `ghcr.io/wikipop/meet-nano:latest` |
   | `JWT_KEY_ID`, `JWT_PRIVATE_KEY`, `JWT_PUBLIC_KEYS` | z `cargo xtask gen-keys` |
   | `TRAEFIK_ENTRYPOINT` | `websecure` |
   | `ALLOWED_ORIGIN` | `https://meet.izbi.beer` |

Domen nie dodajemy w zakładce Domains. Routing definiują etykiety w pliku compose:
frontend jest pod `meet.izbi.beer`, gateway pod `api-meet.izbi.beer`, a sloty pod
`{slot}-meet.izbi.beer`. Obrazy z ghcr.io muszą być publiczne albo Dokploy musi mieć
do nich dostęp przez Registry.

## API

| | Gateway | Slot `{slot}` |
|---|---|---|
| Produkcja | `https://api-meet.izbi.beer` | `https://{slot}-meet.izbi.beer` |
| Lokalnie | `http://localhost:8080` | `http://localhost:8080/slots/{slot}` |

Slot ma postać `{dzień}-{godzina}`. Dni to `pn wt sr cz pt sb nd`, a godziny
to `00`–`23`. Na przykład `sr-09` oznacza środę od 9:00 do 10:00. Indeks
slotu to `dzień * 24 + godzina`.

1. Utworzenie wydarzenia: `POST {gateway}/events` z ciałem
   `{ "name": "Spotkanie koła", "days": ["pn", "sr"], "hours": { "from": 9, "to": 17 } }`
   albo `{ "name": "…", "slots": ["pn-09", "sr-18"] }`. Odpowiedź to `201 { "event_id": "e_…" }`.
   Przedział `hours` jest prawostronnie otwarty. Przy `to: 17` ostatnim slotem jest `16`.
2. Odczyt wydarzenia: `GET {gateway}/events/{eid}` zwraca `{ event_id, name, slots: [...] }`.
3. Dołączenie: `POST {gateway}/events/{eid}/join` z ciałem `{ "nick": "Wiktor", "password": "opcjonalne" }`
   zwraca `{ user_id, token }`. Znany nick bez hasła loguje jako ten sam uczestnik
   na innym urządzeniu. Nick chroniony hasłem wymaga hasła, w przeciwnym razie odpowiedź to `401`.
4. Zapis: dla każdego zmienionego slotu wyślij `PUT {slot}/events/{eid}/me`
   z nagłówkiem `Authorization: Bearer {token}` i ciałem `{ "state": "yes" | "maybe" }`.
   Usunięcie to `DELETE {slot}/events/{eid}/me`, które zwraca `204` i jest idempotentne.
   Brak wpisu oznacza brak dostępności.
5. Odczyt dostępności: wyślij `GET {slot}/events/{eid}` do wszystkich slotów równolegle,
   bez nagłówka `Authorization`. Każdy slot zwraca `{ slot, entries: [{ user_id, nick, state, updated_at }] }`.
   Sloty, których nie udało się pobrać, oznacz w interfejsie osobno.

Błąd ma zawsze postać `{ "error": { "code": "slot_out_of_range", "message": "…" } }`.
Obsługę błędów opieraj na polu `code`. Pełna lista kodów jest w schemacie
OpenAPI pod `{gateway}/docs` i `{slot}/docs`. Surowy schemat jest pod `/openapi.json`.

## Struktura

| Ścieżka | Zawartość |
|---|---|
| `crates/domain` | `SlotId`, `SlotBitmap` (168 bitów), `State`, walidacja nicków, identyfikatory |
| `crates/auth` | claims JWT, wystawianie tokenów w gatewayu, weryfikacja, extractor `AuthUser` |
| `crates/proto` | DTO żądań i odpowiedzi ze schematami OpenAPI, wspólny format błędów |
| `crates/server` | konfiguracja z env, CORS, tracing, graceful shutdown, healthcheck |
| `crates/gateway` | biblioteka z routerem i binarka gatewaya |
| `crates/nano` | biblioteka z routerem i binarka usługi nano, `SLOT_ID` z env |
| `crates/monolith` | gateway i 168 slotów w jednym procesie |
| `xtask` | `gen-compose`, `gen-keys` |
| `tests` | testy kontraktowe na monolicie |
| `docker` | wieloetapowe Dockerfile, builder musl i obraz `FROM scratch` |
| `docs/adr` | decyzje architektoniczne |

Testy jednostkowe i testy API są w katalogu `tests/` każdego crate'a.

## Konfiguracja

| Zmienna | Używa | Domyślnie |
|---|---|---|
| `PORT` | wszystkie | `8080` |
| `DATA_DIR` | wszystkie | `/data`, w monolicie `./data` |
| `ALLOWED_ORIGIN` | wszystkie | `http://localhost:*`, lista po przecinku, `:*` oznacza dowolny port |
| `JWT_ISSUER` | wszystkie | `api-meet.izbi.beer` |
| `JWT_PRIVATE_KEY` | gateway | brak |
| `JWT_KEY_ID` | gateway | `k1` |
| `TOKEN_TTL_DAYS` | gateway | `14` |
| `JWT_PUBLIC_KEYS` | nano | brak, format `kid=PEM,kid=PEM` |
| `SLOT_ID` | nano | brak |
| `LOG_FORMAT` | wszystkie | `json` w obrazach |
| `RUST_LOG` | wszystkie | `info` |

## Zużycie pamięci

Pomiar wykonujemy poleceniem `docker stats --no-stream` na pełnym stacku lokalnym.
Wyników jeszcze nie ma.

| Usługa | Instancje | RAM na instancję | Razem |
|---|---:|---:|---:|
| gateway | 1 | do pomiaru | do pomiaru |
| nano | 168 | do pomiaru | do pomiaru |
| razem | 169 | | do pomiaru |
