# 0002 — Agregacja po stronie frontendu

Status: przyjęta, 2026-10-18.

## Kontekst

Mapa dostępności potrzebuje wszystkich slotów wydarzenia. Usługi nie
rozmawiają ze sobą (ADR 0001). Odczyt z wielu originów z nagłówkiem
`Authorization` wymusza preflight CORS przy każdym żądaniu.

## Decyzja

Przeglądarka pobiera sloty równolegle. Sama liczy mapę dostępności i ranking
terminów.

Żądania `GET` nie wymagają uwierzytelnienia. Identyfikator wydarzenia ma 128
losowych bitów, więc link jest uprawnieniem do odczytu. Tak samo działa
When2meet.

Token jest wymagany tylko przy zapisie, czyli przy `PUT` i `DELETE`.

## Konsekwencje

Odczyt to jedno żądanie na slot, bez preflightu.

Backend nie ma warstwy agregacji, którą trzeba by skalować.

Każdy, kto ma link, widzi nicki i dostępność uczestników. Przy ustalaniu
terminu spotkania koła to akceptowalne.
