# 0004 — Monolit na potrzeby developmentu

Status: przyjęta, 2026-10-18.

## Kontekst

169 kontenerów to za dużo na lokalny development i testy.

## Decyzja

`cargo run -p monolith` montuje te same routery w jednym procesie. Gateway
jest pod `/`, a slot `X` pod `/slots/X/`. Testy kontraktowe działają na
monolicie.

## Konsekwencje

Nie ma osobnej ścieżki kodu do utrzymania.

W przeglądarce jest jeden origin. CORS zachowuje się więc inaczej niż na
produkcji.
