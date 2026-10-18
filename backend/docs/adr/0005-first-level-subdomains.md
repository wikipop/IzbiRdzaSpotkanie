# 0005 — Subdomeny pierwszego poziomu za Cloudflare

Status: przyjęta, 2026-10-18. Część o certyfikatach zastępuje [0006](0006-cloudflare-origin-certificate.md).

## Kontekst

DNS domeny `izbi.beer` jest w Cloudflare. Darmowy Universal SSL obejmuje
tylko jeden poziom subdomen, czyli `*.izbi.beer`. Let's Encrypt limituje
liczbę nowych certyfikatów w tygodniu. 169 osobnych certyfikatów odpada.

## Decyzja

Każdy host jest subdomeną pierwszego poziomu. Gateway działa pod
`api-meet.izbi.beer`, a slot pod `{slot}-meet.izbi.beer`, np.
`pn-00-meet.izbi.beer`. Frontend działa pod `meet.izbi.beer`.

Traefik dostaje jeden certyfikat wildcard `*.izbi.beer` przez wyzwanie DNS-01.
Wnioskuje o niego tylko router gatewaya.

## Konsekwencje

Rekordy mogą być proxowane przez Cloudflare w trybie SSL Full (strict).

Traefik potrzebuje tokenu API Cloudflare z uprawnieniem `Zone:DNS:Edit`
na `izbi.beer`.
