//! A previous phrase's breath-out must not turn the next sung /m/ into an unvoiced release.
use chimera::anatomy::human::BodyMix;
use chimera::control::score::ControlKey;
use chimera::phonetics::sing::{SongOptions, SungNote, midi_to_hz, plan_song};
use chimera::phonetics::{Plan, english::EnglishG2P, songs::DEMO_SONGS, text};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

fn excerpt(start: usize, body: BodyMix, bpm: f64) -> (Plan, f64) {
    let demo = DEMO_SONGS
        .iter()
        .find(|s| s.name == "さくらさくら")
        .unwrap();
    let syl = text::lyrics_to_syllables(demo.lyrics, None, &EnglishG2P::new()).unwrap();
    let shift = if body == BodyMix::MALE { -12.0 } else { 0.0 };
    let notes: Vec<_> = demo
        .notes
        .iter()
        .zip(syl)
        .skip(start)
        .take(4)
        .map(|(n, syl)| SungNote {
            midi: n[0] + shift,
            beats: n[1],
            syl,
        })
        .collect();
    let hz = midi_to_hz(notes[0].midi);
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
        hz,
    )
}

#[test]
fn sakuras_mi_in_miwatasu_and_miniyukan_starts_with_a_voiced_closed_lip_nasal() {
    for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
        for bpm in [57.0, 76.0, 114.0] {
            // Last わ of やよいのそらーわ -> みわたす; last や of いざや -> みにゆかん.
            for start in [13, 43] {
                let (p, hz) = excerpt(start, body, bpm);
                let m = p.segments.iter().find(|s| s.ipa == "m").unwrap();
                let mut v = HumanVoice::new(48000.0, body.body(), 2);
                v.set_source(&SourcePatch {
                    f0: Some(hz),
                    pressure: Some(9.0),
                    tension: Some(0.55),
                    ..Default::default()
                });
                v.schedule(&p.gestures);
                let mut block = [0.0f32; 48];
                let (mut voiced_nasal, mut closed, mut energy, mut count) = (0, 0, 0.0, 0);
                let mut ms = 0;
                while (ms as f64) * 0.001 < m.t1 + 0.1 {
                    v.render(&mut block);
                    ms += 1;
                    let t = ms as f64 * 0.001;
                    assert!(block.iter().all(|x| x.is_finite()));
                    if t >= m.t0 && t < m.t1 {
                        let lip = *v.airway.or_areas.last().unwrap();
                        let velum = v.airway.net.port_area[v.airway.velum_port];
                        if lip < 0.01 && velum > 0.1 {
                            closed += 1;
                            if v.glottis.voicing > 0.5 {
                                voiced_nasal += 1;
                            }
                            energy += block.iter().map(|s| (*s as f64).powi(2)).sum::<f64>();
                            count += block.len();
                        }
                    }
                }
                let rms = (energy / count as f64).sqrt();
                eprintln!(
                    "{body:?} bpm={bpm} excerpt={start}: nasal closed {closed} ms, voiced {voiced_nasal} ms, rms {rms}"
                );
                assert!(closed >= 25, "lost /m/ lip/nasal articulation");
                assert!(
                    voiced_nasal >= 20 && rms > 0.001,
                    "voiceless /mi/: {body:?} bpm={bpm} excerpt={start}: {voiced_nasal} ms, rms {rms}"
                );
                let vowel = &p.segments[p.segments.iter().position(|s| s.ipa == "m").unwrap() + 1];
                assert_eq!(vowel.ipa, "i");
                assert!(
                    (vowel.t0 - (0.3 + 120.0 / bpm)).abs() < 1e-8,
                    "vowel left its beat"
                );
            }
        }
    }
}

#[test]
fn a_previous_phrases_exhale_ends_before_the_next_phrase_starts() {
    for name in ["さくらさくら", "Made of air"] {
        let demo = DEMO_SONGS.iter().find(|s| s.name == name).unwrap();
        let syl = text::lyrics_to_syllables(demo.lyrics, None, &EnglishG2P::new()).unwrap();
        let notes: Vec<_> = demo
            .notes
            .iter()
            .zip(syl)
            .map(|(n, syl)| SungNote {
                midi: n[0],
                beats: n[1],
                syl,
            })
            .collect();
        let p = plan_song(
            &notes,
            &SongOptions {
                body: BodyMix::FEMALE,
                bpm: demo.bpm,
                t0: 0.3,
                vibrato: 35.0,
            },
        );
        for (i, g) in p
            .gestures
            .iter()
            .enumerate()
            .filter(|(_, g)| g.label_str() == "exhale")
        {
            if let Some(next) = p.gestures[i + 1..]
                .iter()
                .find(|g| g.targets.has(ControlKey::BREATH))
            {
                assert!(
                    g.end() <= next.t0 + 1e-9,
                    "{name}: old exhale ends {} after new onset {}",
                    g.end(),
                    next.t0
                );
            }
        }
        // The final release has no following phrase and keeps the normal full fade.
        let last = p
            .gestures
            .iter()
            .rev()
            .find(|g| g.label_str() == "exhale")
            .unwrap();
        let end = p.segments.last().unwrap().t1;
        assert!((last.t1 - (end + 0.1)).abs() < 1e-8);
        assert_eq!(last.ramp_out, 0.06);
    }
}
