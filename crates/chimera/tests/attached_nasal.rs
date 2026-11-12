//! Japanese note allocation is explicit: small N is a coda, ordinary N a nucleus.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::sing::{self, LegatoFrom, NoteOnOptions, SongOptions, SungNote};
use chimera::phonetics::text::Language;
use chimera::phonetics::{text, Segment, Syllable};

fn lyrics(s: &str) -> Vec<Syllable> {
    text::lyrics_to_syllables(s, None, &|_: &str, _: bool| Vec::<Segment>::new()).unwrap()
}

#[test]
fn attached_nasal_takes_no_note_and_assimilates() {
    assert_eq!(lyrics("あん").len(), 2);
    let a = lyrics("あ𛅧");
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].text, "あん");
    assert_eq!(a[0].nucleus.ipa, "a");
    assert_eq!(a[0].coda[0].ipa, "ɰ̃");
    for (s, ipa) in [
        ("あ𛅧た", "n̪"),
        ("せ𛅧ば", "m"),
        ("り𛅧ご", "ŋ"),
        ("ま𛅧じ", "ɲ"),
        ("あ𛅧あ", "ɰ̃"),
        ("ア𛅧タ", "n̪"),
        ("きゃ𛅧た", "n̪"),
    ] {
        let x = lyrics(s);
        assert_eq!(x.len(), 2, "{s}");
        assert_eq!(x[0].coda[0].ipa, ipa, "{s}");
    }
}

#[test]
fn invalid_attachment_is_reported_instead_of_losing_notes() {
    let word = |_: &str, _: bool| Vec::<Segment>::new();
    for s in [
        "𛅧",
        "𛅧あ",
        "あ𛅧𛅧",
        "ん𛅧",
        "あっ𛅧",
        "あ𛅧っか",
        "あ 𛅧",
        "あ、𛅧",
        "んー𛅧",
        "ー𛅧",
    ] {
        assert!(text::lyrics_to_syllables(s, None, &word).is_err(), "{s}");
    }
    assert!(text::lyrics_to_syllables("あ𛅧", Some(Language::English), &word).is_err());
    assert!(text::lyrics_to_syllables("あー𛅧", None, &word).is_ok());
}

fn song(s: &str) -> chimera::phonetics::Plan {
    let notes = lyrics(s)
        .into_iter()
        .map(|syl| SungNote {
            midi: 62.0,
            beats: 1.0,
            syl,
        })
        .collect::<Vec<_>>();
    sing::plan_song(
        &notes,
        &SongOptions {
            body: BodyMix::MALE,
            bpm: 120.0,
            t0: 0.3,
            vibrato: 0.0,
        },
    )
}

#[test]
fn melisma_and_live_release_sing_the_coda_once() {
    let p = song("あ𛅧 - た");
    let n = p
        .segments
        .iter()
        .filter(|s| s.ipa == "ɰ̃")
        .collect::<Vec<_>>();
    assert_eq!(n.len(), 1);
    assert!(n[0].t0 > 0.8); // after the second note's beat, never between the two vowels
    assert_eq!(p.segments.iter().filter(|s| s.ipa == "a").count(), 3);
    let x = lyrics("あ𛅧た");
    let opt = NoteOnOptions {
        body: BodyMix::MALE,
        vibrato: 0.0,
        tag: 10.0,
        legato_from: None,
    };
    assert!(!sing::plan_note_on(&x[0], 62.0, &opt)
        .iter()
        .any(|g| g.label_str() == "n̪"));
    let off = sing::plan_note_off(&x[0], 62.0, BodyMix::MALE);
    assert!(off.iter().any(|g| g.label_str() == "n̪"));
    let on = sing::plan_note_on(
        &x[1],
        64.0,
        &NoteOnOptions {
            legato_from: Some(LegatoFrom {
                syl: x[0].clone(),
                midi: 62.0,
            }),
            ..opt
        },
    );
    assert!(on.iter().any(|g| g.label_str() == "n̪"));
    assert!(on.iter().any(|g| g.label_str() == "t̪"));
}

#[test]
fn antagata_has_a_note_for_every_syllable_including_the_two_kinds_of_n() {
    let demo = chimera::phonetics::songs::DEMO_SONGS
        .iter()
        .find(|s| s.name == "あんたがたどこさ")
        .unwrap();
    let x = lyrics(demo.lyrics);
    assert_eq!(x.len(), demo.notes.len());
    assert_eq!(x[0].text, "あん");
    assert_eq!(x[0].coda[0].ipa, "n̪");
    assert_eq!(x.iter().filter(|s| s.text == "せん").count(), 1);
    assert_eq!(x.iter().filter(|s| s.text == "ん").count(), 1);
    let k = x
        .windows(3)
        .find(|w| w[0].text == "く" && w[1].text == "て" && w[2].text == "さ")
        .unwrap();
    assert_eq!(k[0].coda[0].ipa, "t̪");
    assert_eq!(k[1].onset[0].ipa, "t̪");
    let p = song("くってさ");
    assert_eq!(
        p.segments
            .iter()
            .map(|s| s.ipa.as_str())
            .collect::<Vec<_>>(),
        ["k", "ɯ", "t̪", "t̪", "e", "s̪", "a"]
    );
    // Two overlapping closure gestures keep contact across the coda/onset boundary.
    let stops = p
        .segments
        .iter()
        .filter(|s| s.ipa == "t̪")
        .collect::<Vec<_>>();
    let closures = p
        .gestures
        .iter()
        .filter(|g| g.label_str() == "t̪" && g.strength == 40.0)
        .collect::<Vec<_>>();
    assert_eq!(closures.len(), 2);
    assert!(closures[1].t0 - closures[1].ramp_in <= closures[0].t1);
    assert_eq!(stops[0].t1, stops[1].t0);
}

#[test]
fn kuttesa_keeps_one_physical_closure_across_the_two_t_segments() {
    use chimera::voice::human_voice::{HumanVoice, SourcePatch, VoiceTelemetry};
    for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
        let p = song("くってさ");
        let stops = p
            .segments
            .iter()
            .filter(|s| s.ipa == "t̪")
            .collect::<Vec<_>>();
        let boundary = stops[0].t1;
        let mut v = HumanVoice::new(48000.0, body.body(), 2);
        v.set_source(&SourcePatch::f0(150.0));
        v.schedule(&p.gestures);
        let mut t = VoiceTelemetry::default();
        let mut frame = [0.0; 48];
        while v.clock < boundary + 0.025 {
            v.render(&mut frame);
            v.telemetry(&mut t);
            if t.clock > boundary - 0.025 {
                let min = t.or_areas.iter().copied().fold(f32::INFINITY, f32::min);
                assert!(
                    min < 0.01,
                    "{body:?}: closure reopened at {}: {min}",
                    t.clock
                );
            }
        }
    }
}
