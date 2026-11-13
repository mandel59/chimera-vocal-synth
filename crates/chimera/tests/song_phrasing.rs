//! Semantic word groups must not introduce breaths where the melody has no rest.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::sing::{plan_song, SongOptions, SungNote};
use chimera::phonetics::{songs::DEMO_SONGS, text, Plan};
use chimera::voice::human_voice::{HumanVoice, SourcePatch, VoiceTelemetry};

fn ending(legacy: bool, body: BodyMix) -> Plan {
    let lyrics = if legacy {
        "このはで、ちょいとかぶせ。"
    } else {
        "このはでちょいとかぶせ。"
    };
    let syl = text::lyrics_to_syllables(lyrics, None, &EnglishG2P::new()).unwrap();
    let demo = DEMO_SONGS
        .iter()
        .find(|s| s.name == "あんたがたどこさ")
        .unwrap();
    let shift = if body == BodyMix::MALE { -12.0 } else { 0.0 };
    let notes: Vec<_> = demo.notes[71..]
        .iter()
        .zip(syl)
        .map(|(n, syl)| SungNote {
            midi: n[0] + shift,
            beats: n[1],
            syl,
        })
        .collect();
    plan_song(
        &notes,
        &SongOptions {
            body,
            bpm: demo.bpm,
            t0: 0.3,
            vibrato: 0.0,
        },
    )
}

fn audible_e(p: &Plan, body: BodyMix) -> (f64, f64) {
    let e = p.segments.iter().find(|s| s.ipa == "e").unwrap();
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch {
        f0: Some(if body == BodyMix::MALE { 220.0 } else { 440.0 }),
        voiced: Some(true),
        pressure: Some(9.0),
        tension: Some(0.55),
        ..Default::default()
    });
    v.schedule(&p.gestures);
    let mut t = VoiceTelemetry::default();
    let mut b = [0.0f32; 48];
    let (mut audible, mut energy, mut count) = (0.0, 0.0, 0);
    while v.clock < e.t1 {
        v.render(&mut b);
        v.telemetry(&mut t);
        if t.clock > e.t0 {
            let min = t
                .ph_areas
                .iter()
                .chain(&t.or_areas)
                .copied()
                .fold(f32::INFINITY, f32::min);
            if min > 0.01 && t.voicing > 0.5 {
                audible += 0.001;
            }
            energy += b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>();
            count += b.len();
        }
    }
    (audible, (energy / count as f64).sqrt())
}

#[test]
fn antagata_breathes_at_rests_instead_of_short_grammatical_boundaries() {
    let demo = DEMO_SONGS
        .iter()
        .find(|s| s.name == "あんたがたどこさ")
        .unwrap();
    let x = text::lyrics_to_syllables(demo.lyrics, None, &EnglishG2P::new()).unwrap();
    assert_eq!(x.len(), demo.notes.len());
    let breaks: Vec<_> = x
        .iter()
        .enumerate()
        .filter(|(_, s)| s.phrase_end)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(breaks, [6, 9, 14, 19, 26, 29, 43, 57, 60, 64, 67, 80]);
}

#[test]
fn de_releases_into_an_audible_vowel_before_choi() {
    for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
        let p = ending(false, body);
        let (audible, rms) = audible_e(&p, body);
        assert!(
            audible >= 0.02 && rms > 0.005,
            "{body:?}: audible={audible}, rms={rms}"
        );
        let old = ending(true, body);
        let (before, _) = audible_e(&old, body);
        assert!(audible > before, "{body:?}: audible {before} -> {audible}");
        // Removing the spurious breath changes neither the melody nor the note grid.
        let vowel_onsets = |p: &Plan| {
            p.segments
                .iter()
                .filter(|s| s.kind == "vowel")
                .map(|s| s.t0)
                .collect::<Vec<_>>()
        };
        for (a, b) in vowel_onsets(&p).iter().zip(vowel_onsets(&old)) {
            assert!((a - b).abs() < 1e-6);
        }
    }
}
