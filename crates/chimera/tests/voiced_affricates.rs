//! Physical guards for issue #8 (じゃ heard as りゃ), not a perceptual quality judge.
use chimera::anatomy::human::BodyMix;
use chimera::phonetics::{
    ipa::tokenize_ipa,
    japanese,
    planner::{plan_phrases, plan_utterance, PhraseInput, PlanOptions},
    Segment,
};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

#[derive(Debug)]
struct Evidence {
    span_ms: f64,
    release_closed_ms: usize,
    release_open_ms: usize,
    release_area_max: f64,
    closed_ms: usize,
    flow: f64,
    noise_peak: f64,
    /// mean oral noise-source level over the release (internal amplitude)
    noise: f64,
    vowel_voice: f64,
    vowel_rms: f64,
}

fn evidence(ipa: &str, body: BodyMix, f0: f64, opened: bool) -> Evidence {
    evidence_at_rate(ipa, body, f0, opened, 1.0)
}

/// `opened`: the affricate's release parts the folds, as the Japanese front end asks (typed IPA
/// gets the same setting here); otherwise the setting is taken off, which is how every other
/// language still plans the sound.
fn evidence_at_rate(ipa: &str, body: BodyMix, f0: f64, opened: bool, rate: f64) -> Evidence {
    let options = PlanOptions {
        body,
        t0: 0.05,
        f0: Some(f0),
        rate: Some(rate),
        ..Default::default()
    };
    let release = |segs: &mut Vec<Segment>| {
        for s in segs.iter_mut().filter(|s| s.ipa == "dʑ") {
            s.release_abduction = opened.then_some(japanese::VOICED_AFFRICATE_ABDUCTION);
        }
    };
    let p = if japanese::has_kana(ipa) {
        let phrases = japanese::to_phrases(ipa)
            .into_iter()
            .map(|mut p| {
                let said = p.segments.iter().find(|s| s.ipa == "dʑ").unwrap();
                assert_eq!(said.release_abduction, Some(japanese::VOICED_AFFRICATE_ABDUCTION));
                release(&mut p.segments);
                PhraseInput {
                    segments: p.segments,
                    intonation: Some(p.intonation),
                    pause: Some(p.pause),
                    rate: p.rate,
                    register: p.register,
                    aspiration: p.aspiration,
                }
            })
            .collect::<Vec<_>>();
        plan_phrases(&phrases, &options)
    } else {
        let mut segs = tokenize_ipa(ipa);
        // typed IPA says nothing about the folds: it must stay as it was
        assert!(segs.iter().all(|s| s.release_abduction.is_none()));
        release(&mut segs);
        plan_utterance(&segs, &options)
    };
    let ci = p.segments.iter().position(|s| s.kind != "vowel").unwrap();
    let start = p.segments[ci].t0;
    let end = p.segments[ci + 1..]
        .iter()
        .find(|s| s.kind == "vowel")
        .unwrap()
        .t0;
    let split = start + 0.55 * (end - start);
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&p.gestures);
    let mut b = [0.0f32; 48];
    let (mut samples, mut closed_ms, mut flow, mut count, mut noise_peak) = (0, 0, 0.0, 0, 0.0f64);
    let mut noise_sum = 0.0;
    let mut release_closed_ms = 0;
    let mut release_open_ms = 0;
    let mut release_area_max = 0.0f64;
    let (mut vowel_voice, mut vowel_energy, mut vowel_count) = (0.0, 0.0, 0);
    while samples as f64 / 48000.0 < end + 0.05 {
        v.render(&mut b);
        samples += b.len();
        let t = samples as f64 / 48000.0;
        let area = v
            .airway
            .ph_areas
            .iter()
            .chain(&v.airway.or_areas)
            .copied()
            .fold(f64::INFINITY, f64::min);
        if (start..end).contains(&t) && area < 0.01 {
            closed_ms += 1;
        }
        if (split..end).contains(&t) {
            release_area_max = release_area_max.max(area);
            if area < 0.01 {
                release_closed_ms += 1;
            } else {
                release_open_ms += 1;
            }
        }
        // Include the 15 ms source tail used by the comparison report.
        if (split..end + 0.015).contains(&t) {
            flow += v.airway.net.u[v.airway.glottis_port];
            count += 1;
            let noise = v
                .airway
                .net
                .noise_sources()
                .iter()
                .filter(|s| s.src != v.airway.glottis_port)
                .map(|s| s.level)
                .fold(0.0, f64::max);
            noise_peak = noise_peak.max(noise);
            noise_sum += noise;
        }
        if (end + 0.02..end + 0.045).contains(&t) {
            vowel_voice += v.glottis.voicing;
            vowel_energy += b.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / b.len() as f64;
            vowel_count += 1;
        }
    }
    Evidence {
        span_ms: (end - start) * 1000.0,
        release_closed_ms,
        release_open_ms,
        release_area_max,
        closed_ms,
        flow: flow / count as f64,
        noise_peak,
        noise: noise_sum / count as f64,
        vowel_voice: vowel_voice / vowel_count as f64,
        vowel_rms: (vowel_energy / vowel_count as f64).sqrt(),
    }
}

#[test]
fn dj_has_a_real_closure_and_preserves_the_following_vowel() {
    for (body, f0) in [
        (BodyMix::MALE, 110.0),
        (BodyMix::FEMALE, 200.0),
        (BodyMix::CHILD, 260.0),
    ] {
        for ipa in [
            "ˈɑdʑɑ",
            "dʑa",
            "dʑi",
            "dʑɯ",
            "dʑo",
            "じゃ",
            "じ",
            "じゅ",
            "じょ",
        ] {
            for opened in [false, true] {
                let e = evidence(ipa, body, f0, opened);
                eprintln!("{body:?} {ipa} opened={opened}: {e:?}");
                assert!(e.span_ms > 0.0 && e.release_closed_ms <= e.closed_ms);
                assert!(e.closed_ms >= 8, "closure lost: {body:?} {ipa} {e:?}");
                assert!(
                    e.vowel_voice > 0.8 && e.vowel_rms > 0.005,
                    "vowel lost: {body:?} {ipa} {e:?}"
                );
                assert!(e.flow.is_finite() && e.noise_peak.is_finite());
                assert!(
                    e.release_open_ms + e.release_closed_ms > 0 && e.release_area_max.is_finite()
                );
            }
        }
    }
}

#[test]
fn the_reference_fricative_stays_open() {
    for (body, f0) in [
        (BodyMix::MALE, 110.0),
        (BodyMix::FEMALE, 200.0),
        (BodyMix::CHILD, 260.0),
    ] {
        let e = evidence("ˈɑʑɑ", body, f0, true);
        assert_eq!(e.closed_ms, 0, "a fricative became a stop: {body:?} {e:?}");
        assert!(e.flow > 0.0 && e.noise_peak > 0.0);
        // The tap/glide is a listening control. Its closure is not prescribed here.
        eprintln!(
            "{body:?} ʑ control: {e:?}; ɾj control: {:?}",
            evidence("ˈɑɾjɑ", body, f0, false)
        );
    }
}

/// A necessary condition, not a sufficient test of perceived /dʑ/ quality.
#[test]
fn japanese_kana_has_an_open_fricative_release_before_the_vowel() {
    for (body, f0) in [
        (BodyMix::MALE, 110.0),
        (BodyMix::FEMALE, 200.0),
        (BodyMix::CHILD, 260.0),
    ] {
        for rate in [0.7, 1.0, 1.5] {
            for kana in ["じゃ", "じ", "じゅ", "じょ"] {
                let e = evidence_at_rate(kana, body, f0, true, rate);
                assert!(
                    e.release_open_ms > 0,
                    "no open release before vowel: {body:?} {kana} rate={rate} {e:?}"
                );
                assert!(
                    e.closed_ms >= 8 && e.vowel_voice > 0.8 && e.vowel_rms > 0.005,
                    "closure/vowel lost: {body:?} {kana} rate={rate} {e:?}"
                );
            }
        }
    }
}

/// Issue #8: with the folds as for the vowel, the release of じゃ had modal voicing's flow at most
/// (none while voicing had stopped in the closure), a tenth of ちゃ's noise, and was heard as りゃ.
#[test]
fn japanese_kana_parts_the_folds_for_the_frication() {
    for (body, f0) in [
        (BodyMix::MALE, 110.0),
        (BodyMix::FEMALE, 200.0),
        (BodyMix::CHILD, 260.0),
    ] {
        for rate in [0.7, 1.0, 1.5] {
            for kana in ["じゃ", "じ", "じゅ", "じょ"] {
                let shut = evidence_at_rate(kana, body, f0, false, rate);
                let e = evidence_at_rate(kana, body, f0, true, rate);
                eprintln!("{body:?} {kana} rate={rate}: flow {:.1} -> {:.1}, noise {:.1} -> {:.1}, vowel {:.2}", shut.flow, e.flow, shut.noise, e.noise, e.vowel_voice);
                assert!(
                    e.flow > 1.25 * shut.flow && e.noise > 1.25 * shut.noise,
                    "no more frication: {body:?} {kana} rate={rate} {shut:?} -> {e:?}"
                );
                assert!(
                    e.closed_ms >= 8 && e.vowel_voice > 0.8 && e.vowel_rms > 0.005,
                    "closure/vowel lost: {body:?} {kana} rate={rate} {e:?}"
                );
            }
        }
    }
}

/// Singing takes the same setting through the syllables, and the note still voices.
#[test]
fn sung_kana_parts_the_folds_too() {
    use chimera::phonetics::sing::{midi_to_hz, plan_song, SongOptions, SungNote};
    for (body, midi) in [(BodyMix::MALE, 48.0), (BodyMix::FEMALE, 60.0), (BodyMix::CHILD, 64.0)] {
        for kana in ["じゃ", "じ", "ず"] {
            let syl = japanese::kana_to_syllables(kana, None).remove(0);
            assert_eq!(syl.onset[0].release_abduction, Some(japanese::VOICED_AFFRICATE_ABDUCTION), "{kana}");
            let notes = [SungNote { midi, beats: 1.0, syl }];
            let p = plan_song(&notes, &SongOptions { body, bpm: 120.0, t0: 0.3, vibrato: 0.0 });
            assert_eq!(p.gestures.iter().filter(|g| g.label_str() == "voiced release").count(), 1, "{kana}");
            let vowel = p.segments.iter().find(|s| s.kind == "vowel").unwrap();
            let mut v = HumanVoice::new(48000.0, body.body(), 2);
            v.set_source(&SourcePatch::f0(midi_to_hz(midi)));
            v.schedule(&p.gestures);
            let mut b = [0.0f32; 48];
            let (mut voice, mut n) = (0.0, 0);
            while v.clock < vowel.t1 {
                v.render(&mut b);
                if v.clock > vowel.t0 + 0.03 {
                    voice += v.glottis.voicing;
                    n += 1;
                }
            }
            assert!(voice / n as f64 > 0.8, "the sung vowel lost its voice: {body:?} {kana} {}", voice / n as f64);
        }
    }
}
