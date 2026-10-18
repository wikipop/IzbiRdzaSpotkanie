---
name: "zerya-brand"
description: "Identyfikacja wizualna organizacji Zerya. Używaj przy tworzeniu i ocenie każdego materiału Zerya: prezentacji, pitch decków, dokumentów, stron www."
---

# Zerya: identyfikacja wizualna (v0.4)

## 1. Zasada nadrzędna

Treść jest ważniejsza od kompozycji. Mówimy z radykalną szczerością. Projektujemy minimalistycznie.

- Jeżeli coś nie jest potrzebne do przekazania sensu, nie trafia do materiału.
- Każdy element musi odpowiedzieć na pytanie: co straci odbiorca, jeśli go usuniemy? Jeśli nie ma odpowiedzi, element znika.
- Nie upiększamy, nie zawyżamy i nie dopowiadamy. Piszemy wprost, co jest, a czego nie ma.
- Rdzeń systemu jest wspólny dla wszystkich nośników. Zmienia się tylko skala i wykończenie.

## 2. Pisanie

### Pełne zdania
Piszemy pełnymi, krótkimi zdaniami. Jedno zdanie niesie jedną myśl.

### Myślniki
- W tekście nie używamy myślników ani półpauz jako zamiennika przecinka, nawiasu lub kropki.
- Jeśli zdanie wymaga myślnika, dzielimy je na dwa zdania albo przebudowujemy.
  - Źle: Konkurenci robią jedno albo drugie — nauczyciel żongluje narzędziami.
  - Dobrze: Konkurenci robią jedno albo drugie. Nauczyciel żongluje narzędziami.
- Myślnik może wystąpić wyłącznie jako graficzny separator w elementach systemowych, nigdy w zdaniu:
  - numer i nazwa sekcji, np. „04 — Konkurencja”,
  - zakres liczbowy w cenach i danych, np. „0–19 zł”, „2019–2026”.
- Łącznik w wyrazach złożonych, np. „biało-czarny”, zostaje, ponieważ wynika z ortografii.

### Wersaliki
- Unikamy tekstu pisanego wersalikami. Domyślnie stosujemy zapis zdaniowy: wielka litera tylko na początku zdania i w nazwach własnych.
- Wyróżnienie budujemy wariantem tekstu, czyli wielkością, wagą lub kolorem. Jeśli tekst potrzebuje wersalików, żeby się wyróżnić, jest w złym wariancie.
- Wyjątkiem są skrótowce i nazwy, które zapisuje się wersalikami, np. UE, API.

## 3. Typografia

| Rola | Font | Uwagi |
|---|---|---|
| Nagłówki | Outfit | wagi 500 i 600 |
| Tekst | Geist | wagi 400 i 500 |
| Mono | Geist Mono | tylko gdy treść tego wymaga, np. kod, dane, identyfikatory. Domyślnie nie używamy. |

### Skala

Hierarchię budujemy wyłącznie trzema zmiennymi: wielkością, wagą i kolorem (od ink do dimmed). Nie stosujemy podkreśleń, ramek ani kolorowych teł pod tekstem. Interlinie leżą na siatce 4 px.

| Token | Font | Rozmiar / interlinia | Waga | Kolor | Zastosowanie |
|---|---|---|---|---|---|
| display | Outfit | 56 / 64 px | 600 | ink | jedna główna teza: tytuł slajdu, hero |
| h1 | Outfit | 36 / 44 px | 600 | ink | tytuł sekcji lub dokumentu |
| h2 | Outfit | 22 / 28 px | 500 | ink | podsekcja |
| body | Geist | 16 / 24 px | 400 | ink | treść główna |
| small | Geist | 14 / 20 px | 400 | dimmed | opisy, kontekst, przypisy |
| label | Geist | 12 / 16 px | 500 | dimmed | etykiety, metadane, numery sekcji |

Zasady:
- W jednym widoku (slajd, ekran, strona) używamy najlepiej 4 wariantów, maksymalnie 6.
- Na widok przypada jeden display. Na gęstych widokach tezę niesie h1.
- Kolor dimmed oznacza informację drugorzędną i nigdy nie służy dekoracji.
- W prezentacjach całą skalę mnożymy przez 2, co zachowuje siatkę 8 px.
- Na mobile display ma 40 / 48 px, a h1 ma 28 / 36 px.

## 4. Spacing

### Siatka
- Wszystkie marginesy, odstępy, wymiary elementów i kolumny leżą na siatce 8 px.
- Wartość 4 px stosujemy tylko wtedy, gdy 8 px się nie da: odstęp ikony od tekstu, elementy inline, interlinie małych stopni, korekty optyczne. Każde użycie 4 px ma uzasadnienie.
- Wartości spoza siatki, np. 10, 15 lub 20 px, są niedozwolone.

### Skala odstępów

| Token | Wartość | Zastosowanie |
|---|---|---|
| space-0.5 | 4 px | wyjątek: ikona i tekst, inline, korekty optyczne |
| space-1 | 8 px | etykieta nad nagłówkiem, ciasna grupa |
| space-2 | 16 px | nagłówek nad treścią, odstęp między akapitami |
| space-3 | 24 px | padding kart, elementy listy z opisem |
| space-4 | 32 px | bloki w obrębie sekcji |
| space-6 | 48 px | grupy treści |
| space-8 | 64 px | sekcje |
| space-12 | 96 px | oddech wokół tezy display |
| space-16 | 128 px | otwarcie strony, slajd tytułowy |

### Zasady
1. Bliskość oznacza relację. Odstęp wewnątrz grupy jest zawsze mniejszy niż odstęp między grupami, co najmniej o jeden stopień skali, najlepiej o dwa.
2. Odstęp rośnie z hierarchią. Najważniejszy element ma wokół siebie najwięcej przestrzeni.
3. Pusta przestrzeń zastępuje linie. Najpierw zwiększamy odstęp. Linię dodajemy dopiero wtedy, gdy to nie wystarcza.
4. W jednym widoku używamy najwyżej 4 lub 5 tokenów odstępu.
5. Odstępy mierzymy od krawędzi elementu lub linii bazowej tekstu, nigdy na oko.

### Ustawienia dla nośników

| Nośnik | Marginesy | Kolumny | Rynna |
|---|---|---|---|
| Slajd 1920 × 1080 | 128 px w poziomie, 96 px w pionie | 12 | 32 px |
| Dokument A4 | 64 px | 6 | 24 px |
| Web desktop | treść do 1200 px, boki co najmniej 64 px | 12 | 24 px |
| Web mobile | 24 px | 4 | 16 px |

## 5. Kolor

### Kolory bazowe (około 95% powierzchni)

| Token | Hex | Rola |
|---|---|---|
| paper | #F5F3EC | naturalna biel, domyślne tło |
| paper-raised | #FBFAF6 | karty i wyróżnione pola |
| ink | #0F110F | tekst główny, tło trybu ciemnego |
| graphite | #24272A | ciemny szary przechodzący w czerń, powierzchnie w trybie ciemnym |
| dimmed | #6E726E | tekst drugorzędny |
| line | #DAD7CE | linie podziału, używane oszczędnie |
| forest | #0E3B34 | leśna zieleń, kolor marki i tło znaku: logo, kluczowe CTA, jeden element na widok |

Gradację od graphite do ink stosujemy w tłach trybu ciemnego i w sekcjach otwierających.

Na tle forest tekst i znak są w kolorze paper. Forest może też być tłem całego widoku, np. slajdu tytułowego, bo tak wygląda znak marki.

### Akcent zorzy (najwyżej 5% powierzchni)

| Token | Hex | Rola |
|---|---|---|
| aurora | #5CF0A0 | jasna zieleń, główny akcent |
| aurora-light | #D9FFEB | wygaszenie zorzy do światła |
| aurora-gradient | od #5CF0A0 do #D9FFEB | jedyny dozwolony gradient akcentowy |
| celadon | #B5E3B0 | seledyn, kolor pomocniczy |
| ember | #F1E15B | żółty, kolor pomocniczy, pojedynczy punkt uwagi |

Zasady:
- Domyślnie materiał jest czarno-biało-szary z kolorem forest. Akcent zorzy jest wyjątkiem.
- Kolory pomocnicze nigdy nie występują samodzielnie. Pojawiają się tylko w obrębie akcentu i tylko wtedy, gdy niosą znaczenie.
- Akcentów nie stosujemy do tekstu body i small.
- Akcent aurora najlepiej działa na tle ink, graphite lub forest, tak jak zorza na nocnym niebie.

## 6. Akcenty i motywy

Domyślnie akcentów nie używamy. Akcent można dodać tylko wtedy, gdy spełnia wszystkie trzy warunki:

1. Bez niego materiał traci sens lub czytelność, a nie tylko wygląda gorzej.
2. Ma zależność, czyli da się jednym zdaniem powiedzieć, co oznacza i z czym się wiąże.
3. Jest jeden: jeden typ akcentu na materiał i jedno jego wystąpienie na widok.

### Źródła motywów

| Źródło | Przykład zastosowania z zależnością |
|---|---|
| Zorza polarna | gradient oznacza zmianę, postęp lub przebudzenie, np. w tle sekcji otwierającej lub we wskaźniku postępu |
| Mitologia słowiańska i romantyzm (Mickiewicz, Słowacki), runy nordyckie | runa jako znak systemowy, np. numeracja rozdziałów lub ikona kategorii; cytat jako teza sekcji |
| Technologia (hologram, riso) | ziarno riso lub przesunięcie kolorów oznacza materiał roboczy albo prototyp |

### Kolejność eskalacji
1. Brak akcentu.
2. Minimalistyczny wektor, np. linia, punkt, prosta forma geometryczna. Od tego zawsze zaczynamy.
3. Motyw z listy powyżej, wyłącznie po wskazaniu przez osobę z ownership marki.

W jednym materiale nie łączymy różnych motywów.

## 7. Kompozycja

- Pusta przestrzeń jest elementem hierarchii, a nie brakiem treści.
- Wyrównujemy do lewej. Centrujemy tylko pojedynczą tezę display.
- Nie stosujemy cieni ani ozdobnych ramek. Zaokrąglenia mają 0, 4 lub 8 px.
- Ikony są liniowe, mają grubość 1,5 px i rozmiar 16, 24 lub 32 px, w kolorze ink lub dimmed. Używamy ich tylko wtedy, gdy zastępują tekst, a nie gdy go powtarzają.
- Zdjęcia muszą dokumentować rzeczywistość bez idealizacji. Nie używamy stockowych ilustracji.

## 8. Tabele

- Komórki tabeli są o stopień niżej niż tekst narracyjny widoku. Dane nie konkurują z tezą.
- Rynna między kolumnami ma co najmniej 32 px w prezentacji i 16 px w webie oraz dokumentach. Najdłuższa wartość nigdy nie wypełnia kolumny do krawędzi.
- Liczby wyrównujemy do prawej, stany do środka, tekst do lewej.
- Powtarzalne wartości stanowe, np. tak, częściowo, nie, zamieniamy na minimalistyczne wektory z legendą.
- Kolumnę, która opisuje zamiast porównywać, przenosimy do podpisu pod nazwą wiersza.
- Na slajdzie mieści się najwyżej 6 kolumn. Większa tabela trafia na dwa slajdy albo do aneksu.

## 9. Zastosowania

| Nośnik | Wspólny rdzeń | Wykończenie |
|---|---|---|
| Prezentacje i pitch decki | fonty, skala, siatka, kolory, zasada akcentu, zasady pisania | skala razy 2, jedna teza na slajd, tło forest lub ink dla slajdu tytułowego i sekcji otwierających |
| Dokumenty promocyjne | jak wyżej | tło paper, dłuższe bloki body, label jako metadane stron |
| Strony www | jak wyżej | tryb jasny i ciemny, skala responsywna, akcent może reagować na interakcję, jeśli ta interakcja coś oznacza |

## 10. Tokeny CSS

```css
:root {
  --font-heading: "Outfit", sans-serif;
  --font-body: "Geist", sans-serif;
  --font-mono: "Geist Mono", monospace;

  --paper: #F5F3EC;
  --paper-raised: #FBFAF6;
  --ink: #0F110F;
  --graphite: #24272A;
  --dimmed: #6E726E;
  --line: #DAD7CE;
  --forest: #0E3B34;

  --aurora: #5CF0A0;
  --aurora-light: #D9FFEB;
  --aurora-gradient: linear-gradient(135deg, #5CF0A0, #D9FFEB);
  --celadon: #B5E3B0;
  --ember: #F1E15B;

  --space-0-5: 4px;
  --space-1: 8px;
  --space-2: 16px;
  --space-3: 24px;
  --space-4: 32px;
  --space-6: 48px;
  --space-8: 64px;
  --space-12: 96px;
  --space-16: 128px;
}
```

## 11. Checklist przed oddaniem materiału

- [ ] Czy każdy element jest potrzebny? Co jeszcze można usunąć?
- [ ] Czy komunikat jest szczery, bez przesady i obietnic bez pokrycia?
- [ ] Czy tekst składa się z pełnych zdań bez myślników?
- [ ] Czy wersaliki występują tylko w skrótowcach?
- [ ] Czy hierarchia działa na najwyżej 6 wariantach tekstu, najlepiej na 4?
- [ ] Czy wszystko leży na siatce 8 px, a każde 4 px ma uzasadnienie?
- [ ] Czy odstępy wewnątrz grup są mniejsze niż między grupami?
- [ ] Czy kolor forest występuje najwyżej raz na widok (poza tłem całego slajdu tytułowego)?
- [ ] Jeśli jest akcent: jaką ma zależność i czy ownership zatwierdził motyw?
- [ ] Czy Geist Mono pojawia się tylko tam, gdzie treść tego wymaga?
