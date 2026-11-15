//! Japanese /b/: a moving vowel posture must not pump a sealed mouth into a pop.
use chimera::anatomy::human::{ArticKey, BodyMix};
use chimera::phonetics::{
    Plan, Segment,
    ipa::tokenize_ipa,
    japanese,
    planner::{PhraseInput, PlanOptions, plan_phrases, plan_utterance},
};
use chimera::physics::CM_H2O;
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const REPRO: &str = "ば び ぶ べ ぼ。あば あび あぶ あべ あぼ。かばん。";

fn input(text: &str) -> Vec<PhraseInput> {
    japanese::to_phrases(text)
        .into_iter()
        .map(|p| PhraseInput {
            segments: p.segments,
            intonation: Some(p.intonation),
            pause: Some(p.pause),
            rate: p.rate,
            register: p.register,
            aspiration: p.aspiration,
        })
        .collect()
}

fn unmarked(input: &[PhraseInput]) -> Vec<PhraseInput> {
    let mut json = serde_json::to_value(input).unwrap();
    for p in json.as_array_mut().unwrap() {
        for s in p["segments"].as_array_mut().unwrap() {
            s.as_object_mut().unwrap().remove("vowelTransition");
        }
    }
    serde_json::from_value(json).unwrap()
}

#[derive(Debug)]
struct Evidence {
    pressure: f64,
    closed_ms: usize,
    voice: f64,
    rms: f64,
    release_db: f64,
}

fn evidence(plan: &Plan, body: BodyMix, f0: f64) -> Vec<Evidence> {
    let mut v = HumanVoice::new(48000.0, body.body(), 2);
    v.set_source(&SourcePatch::f0(f0));
    v.schedule(&plan.gestures);
    let mut samples = vec![0.0; ((plan.end + 0.1) * 48000.0) as usize];
    let mut rows = Vec::new();
    for (i, frame) in samples.chunks_mut(48).enumerate() {
        v.render(frame);
        assert!(frame.iter().all(|x| x.is_finite()));
        let pressure = v.airway.net.p[v.airway.oral.first_node()..=v.airway.oral.last_node()]
            .iter()
            .copied()
            .fold(0.0, f64::max);
        rows.push((
            (i * 48 + frame.len()) as f64 / 48000.0,
            pressure,
            *v.airway.or_areas.last().unwrap(),
            v.glottis.voicing,
        ));
    }
    let peak = |a: f64, z: f64| {
        samples[(a * 48000.0) as usize..(z * 48000.0) as usize]
            .iter()
            .map(|s| s.abs() as f64)
            .fold(0.0, f64::max)
    };
    plan.segments
        .iter()
        .enumerate()
        .filter(|(_, s)| s.ipa == "b")
        .map(|(i, s)| {
            let n = &plan.segments[i + 1];
            let dur = n.t1 - n.t0;
            let r: Vec<_> = rows
                .iter()
                .filter(|r| r.0 >= s.t0 - 0.025 && r.0 < s.t1 + 0.025)
                .collect();
            let vowel: Vec<_> = rows
                .iter()
                .filter(|r| r.0 >= n.t0 + 0.5 * dur && r.0 < n.t0 + 0.85 * dur)
                .collect();
            let x = &samples
                [((n.t0 + 0.5 * dur) * 48000.0) as usize..((n.t0 + 0.85 * dur) * 48000.0) as usize];
            Evidence {
                pressure: r.iter().map(|r| r.1).fold(0.0, f64::max) / (8.0 * CM_H2O),
                closed_ms: r.iter().filter(|r| r.2 < 0.01).count(),
                voice: vowel.iter().map(|r| r.3).sum::<f64>() / vowel.len() as f64,
                rms: (x.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt(),
                release_db: 20.0
                    * (peak(s.t1 - 0.025, s.t1 + 0.025) / peak(n.t0 + 0.3 * dur, n.t0 + 0.8 * dur))
                        .log10(),
            }
        })
        .collect()
}

#[test]
fn japanese_b_releases_without_pumping_the_closed_mouth() {
    let marked = input(REPRO);
    let legacy = unmarked(&marked);
    for (body, f0) in [
        (BodyMix::MALE, 112.0),
        (BodyMix::FEMALE, 205.0),
        (BodyMix::CHILD, 250.0),
    ] {
        for rate in [1.0, 0.7, 1.5] {
            let o = PlanOptions {
                body,
                f0: Some(f0),
                rate: Some(rate),
                t0: 0.05,
                ..Default::default()
            };
            let before = plan_phrases(&legacy, &o);
            let after = plan_phrases(&marked, &o);
            assert_eq!(before.segments, after.segments);
            assert_eq!(before.end, after.end);
            let old = evidence(&before, body, f0);
            let new = evidence(&after, body, f0);
            assert_eq!(new.len(), 11);
            for (i, (a, b)) in old.iter().zip(&new).enumerate() {
                eprintln!("{body:?} rate={rate} b#{i}: {a:?} -> {b:?}");
                assert!(
                    b.pressure < 2.0,
                    "overpressure: {body:?} rate={rate} b#{i}: {b:?}"
                );
                assert!(
                    b.closed_ms + 1 >= a.closed_ms,
                    "lost the closure: {a:?} -> {b:?}"
                );
                if rate == 1.0 {
                    assert!(b.closed_ms >= 8, "lost the stop: {b:?}");
                }
                assert!(b.voice > 0.9 && b.rms > 0.001, "lost the vowel: {b:?}");
            }
            // The reported male /bi/ pop stood 6 dB above its following vowel.
            if body == BodyMix::MALE && rate == 1.0 {
                assert!(
                    new[6].release_db < old[6].release_db - 3.0,
                    "{old:?} -> {new:?}"
                );
            }
        }
    }
}

#[test]
fn only_plain_intervocalic_spoken_b_requests_the_transition() {
    let p = input("ば あば しんぶん あっば あびゃ");
    let requests: Vec<_> = p
        .iter()
        .flat_map(|p| &p.segments)
        .filter(|s| {
            serde_json::to_value(s)
                .unwrap()
                .get("vowelTransition")
                .is_some()
        })
        .collect();
    assert_eq!(requests.len(), 1, "only the /b/ of あば");
    assert_eq!(requests[0].ipa, "b");
    for s in japanese::kana_to_syllables("あば あび あぶ", None)
        .iter()
        .flat_map(|s| s.onset.iter().chain([&s.nucleus]).chain(&s.coda))
    {
        assert!(
            serde_json::to_value(s)
                .unwrap()
                .get("vowelTransition")
                .is_none()
        );
    }
    for s in tokenize_ipa("ababa") {
        assert!(
            serde_json::to_value(s)
                .unwrap()
                .get("vowelTransition")
                .is_none()
        );
    }
}

#[test]
fn the_transition_keeps_lip_breath_and_pitch_gestures() {
    let marked = input(REPRO);
    let legacy = unmarked(&marked);
    let keep = |plan: Plan| {
        plan.gestures
            .into_iter()
            .filter_map(|mut g| {
                let retain = [
                    ArticKey::LipAp.into(),
                    ArticKey::LipProt.into(),
                    ArticKey::LipWidth.into(),
                    chimera::control::score::ControlKey::BREATH,
                    chimera::control::score::ControlKey::F0,
                    chimera::control::score::ControlKey::ABDUCTION,
                ];
                let mut t = chimera::control::score::Targets::default();
                let mut any = false;
                for k in retain {
                    if let Some(v) = g.targets.get(k) {
                        t.set(k, v);
                        any = true;
                    }
                }
                g.targets = t;
                any.then(|| serde_json::to_value(g).unwrap())
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keep(plan_phrases(&marked, &Default::default())),
        keep(plan_phrases(&legacy, &Default::default()))
    );
}

#[test]
fn a_transition_request_needs_vowels_on_both_sides_in_one_phrase() {
    for ipa in ["ba", "ab", "mba", "aʃa"] {
        let plain = tokenize_ipa(ipa);
        let marked: Vec<Segment> = plain
            .iter()
            .map(|s| {
                let mut j = serde_json::to_value(s).unwrap();
                j["vowelTransition"] = "afterRelease".into();
                serde_json::from_value(j).unwrap()
            })
            .collect();
        assert_eq!(
            serde_json::to_value(plan_utterance(&plain, &Default::default())).unwrap(),
            serde_json::to_value(plan_utterance(&marked, &Default::default())).unwrap(),
            "{ipa}"
        );
    }
    let plain = tokenize_ipa("abi");
    let mut json = serde_json::to_value(&plain).unwrap();
    json[1]["vowelTransition"] = "afterRelease".into();
    let marked: Vec<Segment> = serde_json::from_value(json).unwrap();
    for o in [
        PlanOptions {
            sung: Some(true),
            ..Default::default()
        },
        PlanOptions {
            open_end: Some(true),
            ..Default::default()
        },
    ] {
        assert_eq!(
            serde_json::to_value(plan_utterance(&plain, &o)).unwrap(),
            serde_json::to_value(plan_utterance(&marked, &o)).unwrap()
        );
    }
    // At the exact release, the incoming zero-ramp posture has zero activation. The old
    // posture must still have weight, rather than exposing the performer's neutral mouth.
    let plan = plan_utterance(&marked, &Default::default());
    let mut score = chimera::control::score::GestureScore::new();
    for g in &plan.gestures {
        score.add(*g);
    }
    assert!(score.blend(ArticKey::TbY, plan.segments[2].t0).weight > 0.0);
}
