//! Opt-in terminal articulation, independent of a language code or an IPA spelling.
use chimera::anatomy::human::{ArticKey, BodyMix};
use chimera::control::score::ControlKey;
use chimera::phonetics::planner::{plan_phrases, plan_utterance, PhraseInput, PlanOptions};
use chimera::phonetics::{ipa::tokenize_ipa, japanese, Plan, ReleaseTiming, Segment};
use chimera::voice::human_voice::{HumanVoice, SourcePatch, VoiceTelemetry};

fn json(p: &Plan) -> serde_json::Value {
    serde_json::to_value(p).unwrap()
}
fn input(text: &str) -> PhraseInput {
    let p = japanese::to_phrases(text).remove(0);
    PhraseInput {
        segments: p.segments,
        intonation: Some(p.intonation),
        pause: Some(p.pause),
        rate: p.rate,
        register: p.register,
        aspiration: p.aspiration,
    }
}
fn rms(x: &[f32]) -> f64 {
    (x.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
}
fn render(p: &Plan, body: BodyMix, f0: f64) -> Vec<f32> {
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&p.gestures);
    let mut x = vec![0.0; ((p.end + 0.35) * 48000.0) as usize];
    for b in x.chunks_mut(128) {
        v.render(b);
    }
    x
}

#[test]
fn only_a_finite_speech_final_closed_nasal_accepts_the_request() {
    for (ipa, opts) in [
        ("an a", PlanOptions::default()),
        ("as", PlanOptions::default()),
        (
            "an",
            PlanOptions {
                open_end: Some(true),
                ..Default::default()
            },
        ),
        (
            "an",
            PlanOptions {
                sung: Some(true),
                ..Default::default()
            },
        ),
    ] {
        let plain = tokenize_ipa(ipa);
        let mut marked = plain.clone();
        for s in &mut marked {
            s.release_timing = Some(ReleaseTiming::AfterBreath);
        }
        assert_eq!(
            json(&plan_utterance(&plain, &opts)),
            json(&plan_utterance(&marked, &opts)),
            "{ipa}"
        );
    }
    for ipa in ["am", "an", "aŋ", "aɴ"] {
        let plain = tokenize_ipa(ipa);
        let mut marked = plain.clone();
        marked.last_mut().unwrap().release_timing = Some(ReleaseTiming::AfterBreath);
        let a = plan_utterance(&plain, &Default::default());
        let b = plan_utterance(&marked, &Default::default());
        assert_eq!(a.segments, b.segments);
        assert_eq!(a.end, b.end);
        assert_ne!(json(&a), json(&b));
        // Voice, breath and pitch do not change; only articulation is retained.
        let source = |p: &Plan| {
            p.gestures
                .iter()
                .filter(|g| {
                    g.targets.has(ControlKey::BREATH)
                        || g.targets.has(ControlKey::ABDUCTION)
                        || g.targets.has(ControlKey::F0)
                })
                .copied()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            serde_json::to_value(source(&a)).unwrap(),
            serde_json::to_value(source(&b)).unwrap()
        );
    }
    let legacy: Segment = serde_json::from_str(r#"{"ipa":"n"}"#).unwrap();
    assert!(legacy.release_timing.is_none());
    assert!(serde_json::to_value(legacy)
        .unwrap()
        .get("releaseTiming")
        .is_none());
}

#[test]
fn adjacent_phrases_do_not_inherit_the_terminal_closure() {
    for pause in [0.0, 0.01, 0.1, 0.3] {
        let mut first = input("ほん。");
        first.pause = Some(pause);
        let next = input("あ。");
        let mut plain = first.clone();
        plain.segments.last_mut().unwrap().release_timing = None;
        let a = plan_phrases(&[plain, next.clone()], &Default::default());
        let b = plan_phrases(&[first, next], &Default::default());
        assert_eq!(a.segments, b.segments);
        assert_eq!(a.end, b.end);
        let limit = b.segments[2].t1 + pause;
        for g in &b.gestures {
            if g.t0 < b.segments[2].t0 && (g.targets.has(ArticKey::Velum) || g.strength == 40.0) {
                assert!(g.t1 + g.ramp_out <= limit + 1e-9, "{pause}: {g:?}");
            }
        }
    }
}

#[test]
fn japanese_nasal_stays_closed_while_voicing_fades_without_a_vocalic_rebound() {
    for (body, f0) in [
        (BodyMix::MALE, 112.0),
        (BodyMix::FEMALE, 210.0),
        (BodyMix::CHILD, 260.0),
    ] {
        for text in ["あん。", "いん。", "うん。", "えん。", "ほん。"] {
            for rate in [0.7, 1.0, 1.5] {
                let marked = input(text);
                assert_eq!(
                    marked.segments.last().unwrap().release_timing,
                    Some(ReleaseTiming::AfterBreath)
                );
                let mut plain = marked.clone();
                plain.segments.last_mut().unwrap().release_timing = None;
                let o = PlanOptions {
                    body,
                    f0: Some(f0),
                    rate: Some(rate),
                    ..Default::default()
                };
                let a = plan_phrases(&[plain], &o);
                let b = plan_phrases(&[marked], &o);
                assert_eq!(a.segments, b.segments);
                assert_eq!(a.end, b.end);
                let before = render(&a, body, f0);
                let after = render(&b, body, f0);
                let onset = b.segments.last().unwrap().t0;
                assert_eq!(
                    before[..(onset * 48000.0) as usize],
                    after[..(onset * 48000.0) as usize]
                );
                let end = b.end;
                let slice = |x: &[f32], a: f64, z: f64| {
                    rms(&x[(a * 48000.0) as usize..(z * 48000.0) as usize])
                };
                let tail_before = slice(&before, end + 0.01, end + 0.06);
                let tail_after = slice(&after, end + 0.01, end + 0.06);
                assert!(
                    tail_after < tail_before,
                    "{text} {body:?} {rate}: {tail_before} -> {tail_after}"
                );
                assert!(
                    slice(&after, (end - 0.025).max(onset), end) > 1e-5,
                    "nasal must remain audible"
                );
                let mut v = HumanVoice::new(48000.0, body.body(), 2);
                v.set_source(&SourcePatch::f0(f0));
                v.schedule(&b.gestures);
                let mut t = VoiceTelemetry::default();
                let mut frame = [0.0; 240];
                while v.clock < end + 0.14 {
                    v.render(&mut frame);
                    v.telemetry(&mut t);
                    if t.clock > end + 0.01 && t.voicing > 0.2 {
                        let min = t
                            .ph_areas
                            .iter()
                            .chain(&t.or_areas)
                            .copied()
                            .fold(f32::INFINITY, f32::min);
                        assert!(
                            min < 0.01 && t.velum_port > 0.1,
                            "{text}: released while voiced at {}",
                            t.clock
                        );
                    }
                }
            }
        }
    }
}
