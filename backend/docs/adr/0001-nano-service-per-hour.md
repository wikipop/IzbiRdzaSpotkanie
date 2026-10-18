# 0001 — Jedna usługa na godzinę tygodnia

Status: przyjęta, 2026-10-18.

## Kontekst

Wydarzenie to zbiór godzin w ogólnym tygodniu. Tydzień ma 7 × 24 = 168 slotów.
Klucz partycjonowania jest znany z góry, skończony i nigdy się nie zmienia.

Obciążenie rozkłada się nierówno. Wtorek o 18:00 dostaje więcej zapisów niż
niedziela o 4:00. Wspólna baza byłaby wspólnym punktem awarii i wspólną
kolejką zapisów.

## Decyzja

System składa się z jednego gatewaya i 168 usług nano. Gateway obsługuje
wydarzenia, uczestników i tokeny. Każda usługa nano obsługuje jeden slot,
od `pn-00` do `nd-23`. Wszystkie usługi nano startują z tego samego obrazu
i różnią się tylko zmienną `SLOT_ID`.

Usługi nie komunikują się ze sobą. Każda ma własny plik SQLite.

Każdy proces jest strojony pod niskie zużycie pamięci. Używa runtime'u Tokio
`current_thread` i jednego synchronicznego połączenia z SQLite. Obrazy są
budowane `FROM scratch`.

## Konsekwencje

Awaria jednej usługi ukrywa jedną godzinę. Reszta grafiku działa.

Topologia jest stała, więc nie ma rebalansowania ani migracji shardów.

Jedno połączenie z bazą wystarcza. Zapytanie trwa mikrosekundy, a o zapis
rywalizują tylko uczestnicy jednej godziny.

Pliku compose dla 169 usług nie da się sensownie pisać ręcznie. Generuje go
`cargo xtask gen-compose`.
