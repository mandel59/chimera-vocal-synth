//! Issue #15: mora pitch must survive inside continuous long vowels and hiatus.
use chimera::anatomy::human::BodyMix;
use chimera::control::score::{ControlKey, GestureScore};
use chimera::phonetics::{
    Plan, Segment,
    ipa::tokenize_ipa,
    japanese,
    planner::{PhraseInput, PlanOptions, plan_phrases, plan_utterance},
};
use chimera::voice::human_voice::{HumanVoice, SourcePatch};

const REPRO: &str = "と＼ーおーお　おーお＼ー　あんうん";

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

fn owns_pitch_at_center(plan: &Plan, marked: &[Segment]) {
    let pitches: Vec<_> = plan
        .gestures
        .iter()
        .filter(|g| g.label_str() == "pitch")
        .collect();
    let expected: Vec<_> = marked
        .iter()
        .enumerate()
        .filter(|(_, s)| s.pitch.is_some())
        .collect();
    assert_eq!(pitches.len(), expected.len());
    let mut score = GestureScore::new();
    for g in &plan.gestures {
        score.add(*g);
    }
    for ((i, _), g) in expected.into_iter().zip(pitches) {
        let s = &plan.segments[i];
        let t = (s.t0 + s.t1) / 2.0;
        let target = g.targets.get(ControlKey::F0).unwrap();
        let actual = score.blend(ControlKey::F0, t).value;
        assert!(
            (actual - target).abs() < 1e-8,
            "mora {i} {} at {t:.4}: its target {target:.4} was diluted to {actual:.4}",
            s.ipa
        );
    }
}

#[test]
fn continuous_o_preserves_each_moras_pitch_instead_of_blending_three_morae() {
    let inputs = input(REPRO);
    assert_eq!(inputs.len(), 1, "spaces must not insert breaths");
    assert_eq!(
        inputs[0]
            .segments
            .iter()
            .map(|s| s.ipa.as_str())
            .collect::<String>(),
        "t̪oooooooooaɰ̃ɯɴ"
    );
    assert_eq!(
        inputs[0]
            .segments
            .iter()
            .filter(|s| s.pitch.is_some())
            .count(),
        13
    );
    for body in [BodyMix::MALE, BodyMix::FEMALE, BodyMix::CHILD] {
        for rate in [0.7, 1.0, 1.5] {
            let p = plan_phrases(
                &inputs,
                &PlanOptions {
                    body,
                    rate: Some(rate),
                    ..Default::default()
                },
            );
            owns_pitch_at_center(&p, &inputs[0].segments);
        }
    }
}

#[test]
fn unequal_mora_lengths_do_not_spill_pitch_into_the_short_neighbors_center() {
    let segs: Vec<_> = [0.03, 0.12, 0.03]
        .into_iter()
        .zip([0.9, 1.12, 0.9])
        .map(|(dur, pitch)| Segment {
            ipa: "o".into(),
            dur: Some(dur),
            pitch: Some(pitch),
            ..Default::default()
        })
        .collect();
    owns_pitch_at_center(&plan_utterance(&segs, &Default::default()), &segs);
}

#[test]
fn mora_pitch_changes_neither_rhythm_nor_articulation_or_breath() {
    for text in [REPRO, "きょ＼ーわ", "お＼ーお", "あいうえお", "あん うん"] {
        let marked = input(text);
        let mut plain = marked.clone();
        for p in &mut plain {
            for s in &mut p.segments {
                s.pitch = None;
            }
        }
        let a = plan_phrases(&marked, &Default::default());
        let b = plan_phrases(&plain, &Default::default());
        assert_eq!(a.segments, b.segments, "{text}");
        assert_eq!(a.end, b.end, "{text}");
        let without_pitch = |p: &Plan| {
            p.gestures
                .iter()
                .filter(|g| !g.targets.has(ControlKey::F0))
                .map(|g| serde_json::to_value(g).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(without_pitch(&a), without_pitch(&b), "{text}");
    }
    // IPA supplies no mora-pitch marks; the default remains the legacy speech contour.
    let plain = tokenize_ipa("tooo aɪ");
    assert!(plain.iter().all(|s| s.pitch.is_none()));
    let plan = plan_utterance(&plain, &Default::default());
    assert!(
        plan.gestures
            .iter()
            .filter(|g| g.label_str() == "pitch")
            .all(|g| g.ramp_in == 0.06 && g.ramp_out == 0.06)
    );

    // An explicit Hz value takes precedence over relative mora pitch, as before.
    let mut fixed = tokenize_ipa("ooo");
    for s in &mut fixed {
        s.f0 = Some(150.0);
    }
    let mut both = fixed.clone();
    for s in &mut both {
        s.pitch = Some(0.9);
    }
    for sung in [None, Some(true)] {
        let o = PlanOptions {
            sung,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(plan_utterance(&fixed, &o)).unwrap(),
            serde_json::to_value(plan_utterance(&both, &o)).unwrap()
        );
    }
}

#[test]
fn the_continuous_vowels_voice_through_the_accent_changes() {
    for (body, f0) in [
        (BodyMix::MALE, 112.0),
        (BodyMix::FEMALE, 205.0),
        (BodyMix::CHILD, 250.0),
    ] {
        for rate in [0.7, 1.0, 1.5] {
            let plan = plan_phrases(
                &input(REPRO),
                &PlanOptions {
                    body,
                    f0: Some(f0),
                    rate: Some(rate),
                    ..Default::default()
                },
            );
            let mut v = HumanVoice::new(48000.0, body.body(), 2);
            v.set_source(&SourcePatch::f0(f0));
            v.schedule(&plan.gestures);
            let mut frame = [0.0; 48];
            let mut centers = Vec::new();
            let mut next = 1; // after the initial stop: nine continuous /o/ morae
            while next <= 9 {
                v.render(&mut frame);
                assert!(frame.iter().all(|x| x.is_finite()));
                let s = &plan.segments[next];
                if v.clock >= (s.t0 + s.t1) / 2.0 {
                    centers.push(v.glottis.params.f0);
                    // The first vowel still has the initial /t/'s aspiration at its centre
                    // at 1.5x, before and after this fix. Check its voiced latter part.
                    if next == 1 {
                        while v.clock < s.t0 + 0.75 * (s.t1 - s.t0) {
                            v.render(&mut frame);
                        }
                    }
                    assert!(v.glottis.voicing > 0.9, "{body:?} rate={rate} mora={next}");
                    assert!(frame.iter().any(|x| x.abs() > 1e-4), "silent mora {next}");
                    next += 1;
                }
            }
            // Input falls inside the first long vowel, then rises and falls in おーお＼ー.
            assert!(centers[0] > centers[1], "{centers:?}");
            assert!(centers[6] > centers[5], "{centers:?}");
            assert!(centers[7] > centers[8], "{centers:?}");
        }
    }
}
