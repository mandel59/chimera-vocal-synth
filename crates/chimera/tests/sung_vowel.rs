//! A planned vowel needs time to voice between two glottal opening gestures.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::english::EnglishG2P;
use chimera::phonetics::sing::{midi_to_hz, plan_song, SongOptions, SungNote};
use chimera::phonetics::{songs::DEMO_SONGS, text, Plan};
use chimera::voice::human_voice::{HumanVoice, SourcePatch, VoiceTelemetry};

fn antagata(body: BodyMix, bpm: f64, legacy: bool) -> (Plan, f64) {
    let demo = DEMO_SONGS
        .iter()
        .find(|s| s.name == "あんたがたどこさ")
        .unwrap();
    let mut syl = text::lyrics_to_syllables(demo.lyrics, None, &EnglishG2P::new()).unwrap();
    if legacy {
        for s in &mut syl {
            s.min_vowel = None;
            for c in s.onset.iter_mut().chain(&mut s.coda) {
                c.aspirated = None;
            }
        }
    }
    let shift = if body == BodyMix::MALE { -12.0 } else { 0.0 };
    let notes: Vec<_> = demo
        .notes
        .iter()
        .zip(syl)
        .map(|(n, syl)| SungNote {
            midi: n[0] + shift,
            beats: n[1],
            syl,
        })
        .collect();
    (
        plan_song(
            &notes,
            &SongOptions {
                body,
                bpm,
                t0: 0.3,
                vibrato: 35.0,
            },
        ),
        midi_to_hz(notes[0].midi),
    )
}

fn measure(p: &Plan, body: BodyMix, f0: f64, a: f64, z: f64) -> (f64, f64, f64) {
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch {
        f0: Some(f0),
        voiced: Some(true),
        pressure: Some(9.0),
        tension: Some(0.55),
        ..Default::default()
    });
    v.schedule(&p.gestures);
    let mut t = VoiceTelemetry::default();
    let mut b = [0.0f32; 48];
    let (mut voiced, mut energy, mut count, mut peak) = (0.0, 0.0, 0, 0.0f64);
    while v.clock < z {
        v.render(&mut b);
        v.telemetry(&mut t);
        if t.clock >= a {
            peak = peak.max(t.voicing);
            if t.voicing > 0.5 {
                voiced += 0.001;
            }
            energy += b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>();
            count += b.len();
        }
    }
    (voiced, (energy / count as f64).sqrt(), peak)
}

#[test]
fn short_to_in_kumamotosa_has_a_voiced_vowel_on_its_beat() {
    for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
        for bpm in [80.0, 100.0, 112.0] {
            let (p, f0) = antagata(body, bpm, false);
            // The first と is note 18, after 10.75 beats; /o/ onsets include the earlier も.
            let beat = 0.3 + 10.75 * 60.0 / bpm;
            let o = p
                .segments
                .iter()
                .find(|s| s.ipa == "o" && (s.t0 - beat).abs() < 1e-6)
                .unwrap();
            assert!(o.t1 - o.t0 >= 0.1 - 1e-9, "{body:?}, {bpm}: {o:?}");
            let (voiced, rms, peak) = measure(&p, body, f0, o.t0, o.t1);
            assert!(
                voiced >= 0.02 && rms > 0.005,
                "{body:?}, {bpm}: voiced={voiced}, rms={rms}, peak={peak}"
            );
            // The following さ still lands on the original beat: the fix does not stretch notes.
            assert!(p
                .segments
                .iter()
                .any(|s| s.ipa == "a" && (s.t0 - (beat + 0.25 * 60.0 / bpm)).abs() < 1e-6));
        }
    }
}

#[test]
fn the_reported_legacy_timing_loses_voicing() {
    let (p, f0) = antagata(BodyMix::FEMALE, 100.0, true);
    let o = p
        .segments
        .iter()
        .find(|s| s.ipa == "o" && (s.t0 - 6.75).abs() < 1e-6)
        .unwrap();
    assert!((o.t1 - o.t0 - 0.054).abs() < 1e-6);
    let (voiced, _, _) = measure(&p, BodyMix::FEMALE, f0, o.t0, o.t1);
    assert!(voiced < 0.005, "legacy unexpectedly voiced for {voiced}");
}

#[test]
fn minimum_nucleus_time_survives_continuations_but_is_not_added_to_other_languages() {
    let g = EnglishG2P::new();
    for s in ["あー", "あ -", "あ𛅧 -"] {
        let x = text::lyrics_to_syllables(s, None, &g).unwrap();
        assert!(x.iter().all(|s| s.min_vowel == Some(0.1)), "{s}: {x:?}");
    }
    for s in ["hello - world", "안녕 세상"] {
        let x = text::lyrics_to_syllables(s, None, &g).unwrap();
        assert!(x.iter().all(|s| s.min_vowel.is_none()), "{s}: {x:?}");
    }
}

#[test]
fn an_impossible_minimum_does_not_stretch_the_note_grid() {
    let x = text::lyrics_to_syllables("たさたさ", None, &EnglishG2P::new()).unwrap();
    let notes: Vec<_> = x
        .into_iter()
        .map(|syl| SungNote {
            midi: 65.0,
            beats: 0.25,
            syl,
        })
        .collect();
    let p = plan_song(
        &notes,
        &SongOptions {
            body: BodyMix::MALE,
            bpm: 240.0,
            t0: 0.3,
            vibrato: 0.0,
        },
    );
    for (i, v) in p.segments.iter().filter(|s| s.kind == "vowel").enumerate() {
        assert!((v.t0 - (0.3 + i as f64 * 0.0625)).abs() < 1e-6);
        assert!(v.t1 - v.t0 >= 0.02 - 1e-9);
    }
}
