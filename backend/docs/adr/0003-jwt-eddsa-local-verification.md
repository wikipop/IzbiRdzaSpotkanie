# 0003 — JWT EdDSA weryfikowany lokalnie

Status: przyjęta, 2026-10-18.

## Kontekst

Usługa nano musi autoryzować zapis bez pytania gatewaya (ADR 0001).

## Decyzja

Przy dołączeniu do wydarzenia gateway wystawia JWT podpisany kluczem Ed25519.
Token zawiera id użytkownika, nick, id wydarzenia i bitmapę jego slotów.
Klucz prywatny ma tylko gateway.

Usługi nano weryfikują token kluczami publicznymi. Klucze są podane jako
lista `kid=PEM`, co pozwala je rotować. Usługa sprawdza też, czy wydarzenie
się zgadza i czy jej slot jest w bitmapie.

Zakres slotów wydarzenia jest niezmienny. Wystawione tokeny pozostają więc
poprawne.

Opcjonalne hasło do nicka jest hashowane algorytmem Argon2id.

## Konsekwencje

Zapis nie wymaga dodatkowego żądania sieciowego.

Tokenów nie da się unieważnić. Wygasają po 14 dniach albo przy rotacji klucza.
