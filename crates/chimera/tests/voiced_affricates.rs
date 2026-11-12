//! Physical guards for issue #8's investigation, not a perceptual quality judge.
//! A glottal-opening candidate lives only in this diagnostic; the common planner is unchanged.
use chimera::anatomy::human::BodyMix;
use chimera::control::score::{ControlKey, Gesture, Targets};
use chimera::phonetics::{
    ipa::tokenize_ipa,
    japanese,
    planner::{plan_phrases, plan_utterance, PhraseInput, PlanOptions},
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
    vowel_voice: f64,
    vowel_rms: f64,
}

fn evidence(ipa: &str, body: BodyMix, f0: f64, candidate: bool) -> Evidence {
    evidence_at_rate(ipa, body, f0, candidate, 1.0)
}

fn evidence_at_rate(ipa: &str, body: BodyMix, f0: f64, candidate: bool, rate: f64) -> Evidence {
    let options = PlanOptions {
        body,
        t0: 0.05,
        f0: Some(f0),
        rate: Some(rate),
        ..Default::default()
    };
    let mut p = if japanese::has_kana(ipa) {
        let phrases = japanese::to_phrases(ipa)
            .into_iter()
            .map(|p| PhraseInput {
                segments: p.segments,
                intonation: Some(p.intonation),
                pause: Some(p.pause),
                rate: p.rate,
                register: p.register,
                aspiration: p.aspiration,
            })
            .collect::<Vec<_>>();
        plan_phrases(&phrases, &options)
    } else {
        plan_utterance(&tokenize_ipa(ipa), &options)
    };
    let ci = p.segments.iter().position(|s| s.kind != "vowel").unwrap();
    let start = p.segments[ci].t0;
    let end = p.segments[ci + 1..]
        .iter()
        .find(|s| s.kind == "vowel")
        .unwrap()
        .t0;
    let split = start + 0.55 * (end - start);
    if candidate {
        // Reuse the existing voiced-fricative setting, bounded inside the affricate.
        p.gestures.push(
            Gesture::new(
                split,
                split.max(end - 0.03),
                0.03f64.min(split - start),
                0.03f64.min(end - split),
                3.0,
                Targets::default().with(ControlKey::ABDUCTION, 0.035),
            )
            .label("candidate voiced release"),
        );
    }
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&p.gestures);
    let mut b = [0.0f32; 48];
    let (mut samples, mut closed_ms, mut flow, mut count, mut noise_peak) = (0, 0, 0.0, 0, 0.0f64);
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
            for candidate in [false, true] {
                let e = evidence(ipa, body, f0, candidate);
                eprintln!("{body:?} {ipa} candidate={candidate}: {e:?}");
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
        let e = evidence("ˈɑʑɑ", body, f0, false);
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
                let e = evidence_at_rate(kana, body, f0, false, rate);
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
