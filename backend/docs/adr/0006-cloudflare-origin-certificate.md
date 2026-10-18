# 0006 — Certyfikat Origin CA zamiast Let's Encrypt

Status: przyjęta, 2026-10-18. Zastępuje część [0005](0005-first-level-subdomains.md) o certyfikatach.

## Kontekst

ADR 0005 zakładał wildcard z Let's Encrypt przez wyzwanie DNS-01. To wymaga
tokenu API Cloudflare w Traefiku i osobnego resolvera w Dokploy. Każdy
dodatkowy sekret na serwerze to dodatkowa powierzchnia ataku.

Rekord `*.izbi.beer` jest proxowany przez Cloudflare. Przeglądarka dostaje
certyfikat Universal SSL od Cloudflare, a nie od serwera.

Certyfikat HTTP-01 dla każdego hosta też odpada. 169 hostów przekracza limit
Let's Encrypt wynoszący 50 nowych certyfikatów na domenę w tygodniu.

## Decyzja

Hosty zostają bez zmian. Gateway działa pod `api-meet.izbi.beer`, a slot pod
`{slot}-meet.izbi.beer`.

Połączenie od Cloudflare do serwera szyfruje certyfikat Cloudflare Origin CA
dla `izbi.beer` i `*.izbi.beer`. Dodajemy go raz w Dokploy, w zakładce
certyfikatów. Routery Traefika mają tylko `tls=true`, bez resolvera ACME.

Tryb SSL w Cloudflare to Full (strict).

## Konsekwencje

Na serwerze nie ma tokenu API Cloudflare. Nie ma też limitów wydawania
certyfikatów.

Certyfikat jest ważny do 15 lat, a jego wymiana jest ręczna.

Ufa mu tylko Cloudflare. Wyłączenie proxy dla rekordu sprawi, że przeglądarki
odrzucą połączenie.
