//! Russian spelling → segments (`phonetics::russian`), through speech and through sung lyrics.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::planner::{PlanOptions, plan_utterance};
use chimera::phonetics::russian;
use chimera::phonetics::songs::DEMO_SONGS;
use chimera::phonetics::syllables::lyrics_to_syllables;

fn ipa(segs: &[chimera::phonetics::Segment]) -> String {
    // a soft consonant is written with its ʲ
    let soft = |s: &chimera::phonetics::Segment| if s.palatalized == Some(true) && !s.ipa.ends_with('ʲ') { "ʲ" } else { "" };
    segs.iter().map(|s| format!("{}{}", s.ipa, soft(s))).collect::<Vec<_>>().join(" ")
}
/// as sung: no vowel reduction
fn sung(w: &str) -> String {
    ipa(&russian::word(w, false))
}
/// as spoken: unstressed vowels reduce
fn said(w: &str) -> String {
    ipa(&russian::word(w, true))
}
/// a spoken phrase of Russian words, each read with the word after it (Speak has no Russian yet:
/// it comes back through the shared speech path, so the speech rules are tested on the reader)
fn spoken(text: &str) -> Vec<chimera::phonetics::Segment> {
    let words: Vec<&str> = text.split_whitespace().collect();
    words.iter().enumerate().flat_map(|(i, w)| russian::word_before(w, words.get(i + 1).copied(), true)).collect()
}
fn phrase(text: &str) -> String {
    ipa(&spoken(text))
}

#[test]
fn vowels_and_consonants() {
    assert_eq!(sung("аоуыэи"), "a o u ɨ e i");
    assert_eq!(sung("шар"), "ʂ a ɾˠ");
    assert_eq!(sung("хор"), "x o ɾˠ");
    assert_eq!(sung("цирк"), "ts ɨ ɾˠ k");
    assert_eq!(sung("жир"), "ʐ ɨ ɾˠ");
    // р is a tap: the trill closed the voice off for ~100 ms and "рыба" was heard as "дыба"
    assert_eq!(sung("рыба"), "ɾˠ ɨ b a");
}

#[test]
fn iotation_and_softness() {
    assert_eq!(sung("ёж"), "j o ʂ");
    assert_eq!(sung("маяк"), "m a j a k");
    assert_eq!(sung("съезд"), "s j e s t");
    // a soft consonant is the palatalised one, with no /j/ of its own (it was "l j u k", heard
    // as a foreign "льюк")
    assert_eq!(sung("лук"), "ɫ u k");
    assert_eq!(sung("люк"), "lʲ u k");
    assert_eq!(sung("мять"), "mʲ a tʲ");
    // soft к г х are marked too (the planner fronts a velar, it cannot raise it)
    assert_eq!(sung("кит"), "kʲ i t");
    // ч and щ are always soft, ж ш ц never
    assert_eq!(sung("чай"), "tʃʲ a j");
    assert_eq!(sung("щи"), "ʃʲ i");
    assert_eq!(sung("жить"), "ʐ ɨ tʲ");
    // ь before и and о: the vowel gets its own /j/ (it was dropped)
    assert_eq!(sung("чьи"), "tʃʲ j i");
    assert_eq!(sung("бульо́н"), "b u lʲ j o n");
}

#[test]
fn stress_marks_and_the_fallbacks() {
    let s = russian::word("молоко́", true);
    assert_eq!(ipa(&s), "m ɐ ɫ ɐ k o");
    assert_eq!(s.iter().filter_map(|s| s.stress).collect::<Vec<_>>(), [0, 0, 1]);
    // ё is stressed
    assert_eq!(said("поёт"), "p ɐ j o t");
    // a frequent word from the table, then the first vowel
    assert_eq!(said("тебя"), "tʲ ɪ bʲ a");
    assert_eq!(said("мама"), "m a m ɐ");
    // singing keeps full vowels
    assert_eq!(sung("молоко́"), "m o ɫ o k o");
}

#[test]
fn unstressed_ya_and_a_after_ch() {
    // я was /ɐ/ everywhere; before the stress it is /ɪ/
    assert_eq!(said("язы́к"), "j ɪ z ɨ k");
    assert_eq!(said("пяти́"), "pʲ ɪ tʲ i");
    assert_eq!(said("часы́"), "tʃʲ ɪ s ɨ");
    // but word-final я and а stay open
    assert_eq!(said("неде́ля"), "nʲ ɪ dʲ e lʲ ɐ");
    assert_eq!(said("ту́ча"), "t u tʃʲ ɐ");
}

#[test]
fn devoicing_and_assimilation_inside_a_word() {
    assert_eq!(sung("дуб"), "d u p");
    assert_eq!(sung("лодка"), "ɫ o t k a");
    assert_eq!(sung("сдать"), "z d a tʲ");
    assert_eq!(sung("вторник"), "f t o ɾˠ nʲ i k");
    // в doesn't voice what's before it
    assert_eq!(sung("твой"), "t v o j");
    // a soft consonant devoices to its soft partner, and soft ones assimilate too
    assert_eq!(sung("ло́шадь"), "ɫ o ʂ a tʲ");
    assert_eq!(sung("про́сьба"), "p ɾˠ o zʲ b a");
}

#[test]
fn a_preposition_is_part_of_the_next_word() {
    // each was devoiced as if it ended a phrase: "нат лесом", "ф лесу", "ис дома"
    assert_eq!(phrase("над ле́сом"), "n ɐ d lʲ e s ɐ m");
    assert_eq!(phrase("в лесу́"), "v lʲ ɪ s u");
    assert_eq!(phrase("из до́ма"), "i z d o m ɐ");
    // and it assimilates to an obstruent
    assert_eq!(phrase("в ти́хом"), "f tʲ i x ɐ m");
    assert_eq!(phrase("с горы́"), "z g ɐ ɾˠ ɨ");
    assert_eq!(phrase("из-под стола́"), "i s p ɐ t s t ɐ ɫ a");
    // sung words come one at a time: the preposition keeps its spelling's voicing
    assert_eq!(sung("над"), "n a d");
    // a word that isn't a preposition still devoices
    assert_eq!(phrase("са́д мо́й"), "s a t m o j");
}

#[test]
fn g_in_the_genitive_ending_is_v() {
    assert_eq!(said("его́"), "j ɪ v o");
    assert_eq!(said("сего́дня"), "sʲ ɪ v o d nʲ ɐ");
    assert_eq!(said("кра́сного"), "k ɾˠ a s n ɐ v ɐ");
    assert_eq!(said("ничего́"), "nʲ i tʃʲ ɪ v o");
    assert_eq!(said("мно́го"), "m n o g ɐ");
}

#[test]
fn spelling_conventions() {
    assert_eq!(sung("что"), "ʂ t o");
    assert_eq!(sung("коне́чно"), "k o nʲ e ʂ n o");
    assert_eq!(sung("сча́стье"), "ʃʲ a s tʲ j e");
    assert_eq!(sung("мужчи́на"), "m u ʃʲ i n a");
    assert_eq!(sung("по́здно"), "p o z n o");
    assert_eq!(sung("со́лнце"), "s o n ts e");
    assert_eq!(sung("чу́вство"), "tʃʲ u s t v o");
    assert_eq!(said("смеётся"), "s mʲ ɪ j o ts ɐ");
    assert_eq!(sung("купа́ться"), "k u p a ts a");
}

#[test]
fn the_review_s_small_gaps() {
    // silent д in рдц, т in стл (they were said)
    assert_eq!(sung("се́рдце"), "sʲ e ɾˠ ts e");
    assert_eq!(sung("счастли́вый"), "ʃʲ a s lʲ i v ɨ j");
    // unstressed э reduces like е (it stayed /e/)
    assert_eq!(said("этажа́"), "ɪ t ɐ ʐ a");
    assert_eq!(said("э́то"), "e t ɐ");
    // the vowel of -тся, -ться gets a Russian length, not English's 40-50 ms
    let s = spoken("смеётся");
    assert_eq!(s.last().unwrap().dur, Some(0.12));
    let s = spoken("смеётся он");
    let k = s.iter().position(|s| s.ipa == "ts").unwrap() + 1;
    assert_eq!((s[k].ipa.as_str(), s[k].dur), ("ɐ", Some(0.09)));
}

#[test]
fn a_doubled_letter_is_one_consonant() {
    // long between vowels, plain before a consonant (it was two separate closures)
    let vanna = russian::word("ва́нна", false);
    assert_eq!(ipa(&vanna), "v a n a");
    assert!(vanna[2].dur.is_some_and(|d| d > 0.1), "{:?}", vanna[2].dur);
    let russkiy = russian::word("ру́сский", false);
    assert_eq!(ipa(&russkiy), "ɾˠ u s kʲ i j");
    assert_eq!(russkiy[2].dur, None);
}

#[test]
fn the_reader_s_speech_plans() {
    for text in ["Приве́т", "Ты меня́ слы́шишь"] {
        let plan = plan_utterance(&spoken(text), &PlanOptions { body: BodyMix::MALE, ..Default::default() });
        assert!(plan.end.is_finite() && plan.end > 0.0);
        assert!(!plan.gestures.is_empty());
    }
}

#[test]
fn english_speech_is_untouched() {
    // Speak's tokenizer kept U+0301 in every word for Russian, so "cafe" + U+0301 (as pasted
    // text sometimes arrives) read as one word, "k eɪ f ɛ", where main read "cafe"
    let g = EnglishG2P::new();
    assert_eq!(g.to_phrases("cafe\u{301}")[0].segments, g.to_phrases("cafe")[0].segments);
}

#[test]
fn lyrics_split_into_syllables() {
    let g = EnglishG2P::new();
    let s = lyrics_to_syllables("мо-ло-ко́ ~", &g);
    assert_eq!(s.len(), 4);
    assert_eq!(s[3].melisma, Some(true));
    let texts = |t: &str| lyrics_to_syllables(t, &g).iter().map(|s| s.text.clone()).collect::<Vec<_>>();
    assert_eq!(texts("мая́к"), ["ма", "я\u{301}к"]);
    assert_eq!(russian::split_spelling("поёт", 2), ["по", "ёт"]);
}

#[test]
fn sung_syllables_are_open() {
    // Russian singing gives the next syllable the whole cluster, so the vowel holds the note: only
    // the onsets пр, стр were allowed, and я́блоко sang as я́б-ло-ко, Москва́ put /sk/ on the first
    // note, ло́дка sang as ло́д-ка
    let g = EnglishG2P::new();
    let onsets = |t: &str| lyrics_to_syllables(t, &g).iter().map(|s| ipa(&s.onset)).collect::<Vec<_>>();
    assert_eq!(onsets("ло́дка"), ["ɫ", "t k"]);
    assert_eq!(onsets("я́блоко"), ["j", "b ɫ", "k"]);
    assert_eq!(onsets("Москва́"), ["m", "s k v"]);
    assert_eq!(onsets("сестра́"), ["sʲ", "s t ɾˠ"]);
    // a leading sonorant (р л н м й) stays with the vowel before it
    assert_eq!(onsets("се́рдце"), ["sʲ", "ts"]);
    assert_eq!(onsets("войска́"), ["v", "s k"]);
    assert_eq!(onsets("темно́"), ["tʲ", "n"]);
    // the /j/ after ь goes with the consonant before it
    assert_eq!(onsets("семья́"), ["sʲ", "mʲ j"]);
    // the spelling on the notes splits the same way
    let texts = lyrics_to_syllables("я́блоко", &g).iter().map(|s| s.text.clone()).collect::<Vec<_>>();
    assert_eq!(texts, ["я\u{301}", "бло", "ко"]);
    // a sonorant left on the note before is written there too
    let texts = |t: &str| lyrics_to_syllables(t, &g).iter().map(|s| s.text.clone()).collect::<Vec<_>>();
    assert_eq!(texts("войска́"), ["вой", "ска\u{301}"]);
    assert_eq!(texts("се́рдце"), ["се\u{301}р", "дце"]);
    assert_eq!(texts("бо́льно"), ["бо\u{301}ль", "но"]);
    assert_eq!(texts("Москва́"), ["Мо", "сква\u{301}"]);
}

#[test]
fn the_demo_song_has_a_note_per_syllable() {
    let g = EnglishG2P::new();
    let song = DEMO_SONGS.iter().find(|s| russian::contains_russian(s.lyrics)).expect("a Russian demo song");
    assert_eq!(lyrics_to_syllables(song.lyrics, &g).len(), song.notes.len());
}

#[test]
fn a_sung_preposition_without_a_vowel_joins_the_next_word() {
    // в, с, к have no vowel, so no note: they were silently dropped from the song
    let g = EnglishG2P::new();
    let s = lyrics_to_syllables("в лесу́", &g);
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].text, "в ле");
    assert_eq!(ipa(&s[0].onset), "v lʲ");
    // and it takes the next word's voicing
    assert_eq!(ipa(&lyrics_to_syllables("к тебе́", &g)[0].onset), "k tʲ");
    assert_eq!(ipa(&lyrics_to_syllables("с горы́", &g)[0].onset), "z g");
    assert_eq!(ipa(&lyrics_to_syllables("в ти́хом", &g)[0].onset), "f tʲ");
}

#[test]
fn voiceless_stops_are_unaspirated() {
    // the planner's default is English's rule: п т к before a vowel came out aspirated
    let s = russian::word("папа", false);
    assert_eq!(s[0].aspirated, Some(false));
    assert_eq!(s[2].aspirated, Some(false));
    assert_eq!(russian::word("тётя", false)[2].aspirated, Some(false));
    assert_eq!(russian::word("чай", false)[0].aspirated, Some(false));
    // not said for anything else, and never for English
    assert_eq!(russian::word("мама", false)[0].aspirated, None);
    assert!(EnglishG2P::new().word("paper", false).iter().all(|s| s.aspirated.is_none()));
    // the plan has the short, unaspirated devoicing: it ends 5 ms after the stop, not 20
    let end_of_devoicing = |aspirated| {
        let segs = [chimera::phonetics::Segment { aspirated, ..chimera::phonetics::Segment::new("t") }, chimera::phonetics::Segment { stress: Some(1), ..chimera::phonetics::Segment::new("a") }];
        let plan = plan_utterance(&segs, &PlanOptions { body: BodyMix::MALE, ..Default::default() });
        let t1 = plan.segments[0].t1;
        plan.gestures.iter().find(|g| g.label.as_ref().is_some_and(|l| l.as_str() == "aspiration")).map(|g| g.t1 - t1).expect("a devoicing gesture")
    };
    assert!(end_of_devoicing(None) > 0.015, "English default");
    assert!(end_of_devoicing(Some(false)) < 0.01);
    assert_eq!(end_of_devoicing(Some(true)), end_of_devoicing(None));
}

#[test]
fn a_soft_consonant_is_the_plain_one_palatalized_by_the_planner() {
    // no palatalised inventory entries and no /j/ glide: the segment carries the mark, and the
    // planner gives the consonant a tongue-body gesture ("palatalized")
    let segs = russian::word("тётя", false);
    assert_eq!(segs.iter().map(|s| s.ipa.as_str()).collect::<Vec<_>>(), ["t", "o", "t", "a"]);
    assert_eq!(segs.iter().map(|s| s.palatalized).collect::<Vec<_>>(), [Some(true), None, Some(true), None]);
    let palatal = |w: &str| {
        let plan = plan_utterance(&russian::word(w, false), &PlanOptions { body: BodyMix::MALE, ..Default::default() });
        plan.gestures.iter().filter(|g| g.label.as_ref().is_some_and(|l| l.as_str() == "palatalized")).count()
    };
    assert_eq!(palatal("тётя"), 2);
    assert_eq!(palatal("тот"), 0);
    // every soft consonant gets one, the soft р (its own tap) and the always-soft ч щ included
    for w in ["пять", "бить", "мя́со", "фен", "ве́ра", "день", "нет", "се́но", "зя́блик", "лю́к", "ря́д", "чай", "щи", "ки́т", "ги́д", "хи́трый"] {
        assert!(palatal(w) >= 1, "{w}");
    }
}

#[test]
fn r_is_a_tap_that_does_not_seal() {
    use chimera::phonetics::inventory::phoneme;
    use chimera::anatomy::human::ArticKey;
    // a sealed tap (the stops' tip height, 25 ms of closure) was heard as /d/: Whisper wrote
    // "дада" for правда and "дактор" for трактор, and heard 7 of 37 р
    let stop = phoneme("d", BodyMix::MALE).unwrap().targets.get(ArticKey::TtY).unwrap();
    for ipa in ["ɾˠ", "ɾʲ"] {
        for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
            let p = phoneme(ipa, body).unwrap_or_else(|| panic!("{ipa} is not in the inventory"));
            assert!(p.targets.get(ArticKey::TtY).unwrap() < stop - 0.3, "{ipa} seals");
        }
    }
    // the hard one draws the tongue body back, where a plain alveolar fronts it
    let hard = phoneme("ɾˠ", BodyMix::MALE).unwrap();
    assert!(hard.targets.get(ArticKey::TbX).unwrap() < 3.0);
    assert_eq!(sung("рис"), "ɾʲ i s");
    assert_eq!(ipa(&chimera::phonetics::ipa::tokenize_ipa("ɾˠaɾʲi")), "ɾˠ a ɾʲ i");
}

#[test]
fn unstressed_vowels_keep_a_syllable_s_length() {
    // the duration rules are English's and have no /ɐ/: unstressed vowels came out 40-50 ms,
    // "Доброе утро" took 0.75 s for five syllables, and Whisper lost whole words
    let p = spoken("До́брое у́тро");
    let weak: Vec<f64> = p.iter().filter(|s| s.stress == Some(0)).filter_map(|s| s.dur).collect();
    assert_eq!(weak.len(), 3);
    assert!(weak.iter().all(|d| *d >= 0.07), "{weak:?}");
    // the last one, before a pause, is the longest
    assert!(weak[2] > weak[0]);
}

#[test]
fn a_stressed_y_is_as_long_as_the_other_stressed_vowels() {
    // the duration table's /ɨ/ is English's reduced vowel (110 ms inherent): "дым" came out with
    // half the vowel of "дум" and Whisper wrote "дум" for it
    let y = |text: &str| spoken(text).iter().find(|s| s.ipa == "ɨ").unwrap().dur;
    assert_eq!(y("ды́м"), Some(0.19));
    assert_eq!(y("ды́м и до́м"), Some(0.12));
    // sung, the note sets the length
    assert!(russian::word("дым", false).iter().all(|s| s.ipa != "ɨ" || s.dur.is_none()));
}

#[test]
fn russian_is_a_front_end_on_the_lyrics_router() {
    let g = EnglishG2P::new();
    // one line, three front ends: each word goes to its own
    let s = lyrics_to_syllables("twinkle ми́р 별", &g);
    assert_eq!(s.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["twin", "kle", "ми́р", "별"]);
    assert!(s.iter().filter(|s| s.word_end).count() == 3);
    // a preposition with no vowel leans only on a Russian word right after it in the phrase
    assert_eq!(lyrics_to_syllables("в лесу́", &g)[0].text, "в ле");
    assert_eq!(lyrics_to_syllables("в, лесу́", &g)[0].text, "ле");
    let s = lyrics_to_syllables("в star ми́р", &g);
    assert_eq!(s.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(), ["star", "ми́р"]);
}

#[test]
fn a_lone_stress_mark_is_not_a_russian_word() {
    // Russian claimed a word that starts with U+0301, so ni + U+0301 (pinyin with a separate
    // tone mark) would lose its tone to Russian once Mandarin is on the router
    use chimera::phonetics::syllables::FrontEnd;
    assert!(!russian::Russian.starts('\u{301}'));
    assert!(russian::Russian.continues('\u{301}'));
    let g = EnglishG2P::new();
    assert_eq!(lyrics_to_syllables("молоко́", &g).len(), 3);
}
