# Russian

Sing accepts Cyrillic lyrics. A word with a Cyrillic letter is read by
`crates/chimera/src/phonetics/russian.rs`, a front end on the lyrics router
([language-front-ends.md](language-front-ends.md)), so a line can mix it with English and Korean.
Russian is mapped onto the existing inventory, with the engine's palatalization for the soft
consonants; the only new entries are two taps for р.

Speak doesn't read Russian yet. The reader already has the speech rules (`word_before` with
`reduce`: vowel reduction, Russian vowel lengths, prepositions voiced by the next word), and they
are tested on it directly; Russian Speak comes back through the shared speech path, with the same
front-end list as Sing.

## Writing it

- **Mark the stress** with a combining acute after the vowel: `молоко́`. ё is always stressed.
  An unmarked word is looked up in a short table of frequent words (`STRESS`); failing that, its
  **first vowel** is stressed. That guess is wrong for most longer words, so mark them.
- In Sing, one vowel is one note. `мо-ло-ко́` forces the breaks, `~` holds the previous vowel.
- Sung syllables are open: the next note takes the whole consonant cluster (я-бло-ко, мо-сква́),
  except a leading sonorant р л н м й, which stays on the note before (се́р-дце, вой-ска́).
- Write numbers as words: digits still go through the English number expander.

## What it does

| | |
|---|---|
| Vowels | а о у э ы и; я е ё ю add /j/ at the start of a word, after a vowel and after ь ъ |
| Reduction (speech only) | unstressed о а → /ɐ/; е, э, and я before the end of the word → /ɪ/; а after ч щ → /ɪ/ |
| Hard-only consonants | ж ш ц: и and unstressed е after them are /ɨ/ |
| Voicing | a final obstruent devoices (дуб → /p/); voicing spreads backwards through a cluster (ло́дка → /tk/, сдать → /zd/); в doesn't start it (твой) |
| Prepositions | в, с, к, из, над, под… are unstressed and take their voicing from the next word (в лесу́ /v/, в ти́хом /f/, с горы́ /z/) |
| Spelling conventions | -ого, -его and его, сего́дня → /v/; что, коне́чно → /ʂ/; сч, зч, жч → щ; silent т д л в in стн, стл, здн, рдц, лнц, вств; -тся, -ться → /tsa/ |
| Doubled letters | one consonant, half again as long between vowels (ва́нна), plain before a consonant (ру́сский) |

## Soft consonants

A soft consonant is the plain one with `Segment.palatalized` set (IPA ʲ): the planner raises and
fronts the tongue body under it, towards /j/, holds it until the voice returns and lets it go
into the vowel. There are no palatalised inventory entries and no /j/ glide segment.

- Soft л is the clear /l/ (the hard one is the dark /ɫ/, which holds the tongue body itself).
- Soft к г х are marked too; the planner only fronts a velar, whose height is its closure.
- ч is /tʃ/ and щ a long /ʃ/, always soft. They keep /ʃ/'s lip rounding: the inventory has no
  alveolo-palatals. ж ш ц are always hard.
- Voiceless stops and affricates are unaspirated (`Segment.aspirated = false`). Unsaid, the
  planner keeps English's rule.

## What is approximate

- **Soft consonants**: by ear the softness is weak, and after a soft consonant е can be heard as
  "иа". An earlier version with its own palatalised entries and a stronger, longer body gesture
  scored a little better on a Whisper judge (0.37 against 0.41 character error rate on 30
  phrases, within the judge's noise).
- **р** is one tap that stops short of a seal, /ɾˠ/ hard (tongue body drawn back) and /ɾʲ/ soft
  (no body target of its own, palatalized by the planner). It is not a trill: the tongue tip in
  the model cannot open and close at 25 Hz, and the inventory's /r/ comes out as one 85 ms
  closure. A sealed tap was heard as /d/ (Whisper wrote дада for правда), an open one drifts
  towards /l/. Whisper still hears only about a third of the р; a real trill needs the tip's
  opening modulated in the engine.
- **Hard л** at the start of an utterance can be heard as р: with the tip raised F2 stays near
  1.5 kHz whatever the tongue body does (a real one is near 1 kHz).
- **Sung prepositions**: в, с, к have no vowel and so no note; they start the next word's first
  syllable (and take its voicing). A preposition with a vowel is sung as its own word and keeps
  its spelling's voicing (над ти́хим keeps /d/).
- Not done: stress for arbitrary unmarked words, voicing across other word boundaries, loanwords
  with hard consonants before е (те́ннис), numbers.
- Ukrainian і ї є ґ are not Russian letters, so they split a word (привіт sings as прив and т).

## Tests

`crates/chimera/tests/russian.rs` has one case per rule above.
