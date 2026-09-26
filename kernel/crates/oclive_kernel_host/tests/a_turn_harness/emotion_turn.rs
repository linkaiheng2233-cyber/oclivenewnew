//! E-V1: real builtin Emotion -> Host tone/DTO/chat metadata, with a recording LLM.
//! These are implementation expectations, not claims about real people's emotions or Base semantics.

use serde_json::{json, Value};

pub const REPLY: &str = "我明白了，这就来。";
pub const CASES: &[Case] = &[
    Case {
        scenario: "e1",
        run_id: "A-EV1-4b7ccc66-E01-R0",
        message: "我很开心",
        joy: true,
    },
    Case {
        scenario: "e2",
        run_id: "A-EV1-4b7ccc66-E02-R0",
        message: "今天星期三",
        joy: false,
    },
    Case {
        scenario: "e3",
        run_id: "A-EV1-4b7ccc66-E03-R0",
        message: "我不开心",
        joy: false,
    },
    Case {
        scenario: "e4",
        run_id: "A-EV1-4b7ccc66-E04-R0",
        message: "嗯嗯好的",
        joy: false,
    },
];

pub struct Case {
    pub scenario: &'static str,
    pub run_id: &'static str,
    pub message: &'static str,
    joy: bool,
}

pub fn find(scenario: &str) -> Option<&'static Case> {
    CASES.iter().find(|c| c.scenario == scenario)
}

pub fn validate_mode(run_id: &str, scenario: &str, approval: &str) -> Result<(), String> {
    match find(scenario) {
        Some(c) if c.run_id == run_id && approval.is_empty() => Ok(()),
        _ => Err("E-V1 requires the exact scenario/run_id and no live approval".into()),
    }
}

impl Case {
    // Host labels are the Display form of the discrete Emotion enum. They are not
    // the seven-dimension score keys or the Base clue report's dimension names.
    fn label(&self) -> &'static str {
        if self.joy {
            "happy"
        } else {
            "neutral"
        }
    }

    fn scores(&self) -> Value {
        // Independent literal oracle; never call the production analyzer to compute expectations.
        json!({"joy": if self.joy {1.0} else {0.0}, "sadness":0.0, "anger":0.0,
            "fear":0.0, "surprise":0.0, "disgust":0.0,
            "neutral": if self.joy {0.0} else {1.0}})
    }

    fn tone(&self) -> &'static str {
        if self.joy {
            "用户语气线索：偏愉快、积极或感激，可先共鸣再展开（标签 happy，信号强度：强）"
        } else {
            "用户语气线索：整体较平或信息性为主，按常速自然回（标签 neutral，信号强度：弱·偏中性）"
        }
    }
}

/// Used by the actual child after the single turn and bounded shutdown, and by mutation tests.
/// Reading archived metadata does not perform a second emotion analysis.
pub fn verify(scenario: &str, prompts: &[Value], response: &Value, sqlite: &Value) -> Vec<String> {
    let Some(case) = find(scenario) else {
        return vec!["unknown Emotion scenario".into()];
    };
    let mut failures = Vec::new();
    let scores = case.scores();
    if response.get("emotion") != Some(&scores) {
        failures.push(
            "response.emotion differs from the independent seven-dimension expectation".into(),
        );
    }
    let prompt = prompts
        .first()
        .and_then(|p| p.get("prompt"))
        .and_then(Value::as_str);
    if prompts.len() != 1
        || !prompt.is_some_and(|p| {
            p.matches(case.tone()).count() == 1
                && p.contains(&format!("【最新用户消息】\n用户说: {}\n", case.message))
                && !p.contains(if case.joy {
                    "（标签 neutral，"
                } else {
                    "（标签 happy，"
                })
        })
    {
        failures.push(
            "recorded Prompt lacks the exact derived tone/input or contains a contrary label"
                .into(),
        );
    }
    let Some(rows) = sqlite.get("chat_messages").and_then(Value::as_array) else {
        failures.push("chat metadata rows missing".into());
        return failures;
    };
    let users: Vec<_> = rows.iter().filter(|r| r["sender"] == "user").collect();
    let assistants: Vec<_> = rows.iter().filter(|r| r["sender"] == "assistant").collect();
    if rows.len() != 2 || users.len() != 1 || assistants.len() != 1 {
        failures
            .push("emotion metadata must belong to exactly one user and one assistant row".into());
        return failures;
    }
    let user = users[0];
    if user["id"].as_str().is_none_or(str::is_empty)
        || user.get("id") != response.get("user_message_id")
        || user["session_id"] != crate::support::ROLE_ID
        || user["content"] != case.message
    {
        failures.push("emotion metadata user identity/content mismatch".into());
    }
    match user
        .get("metadata")
        .and_then(Value::as_str)
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
    {
        Some(meta)
            if meta.get("user_emotion_scores") == Some(&scores)
                && meta["user_emotion"] == case.label() => {}
        _ => failures.push("user metadata must retain the exact scores and derived label".into()),
    }
    match assistants[0]
        .get("metadata")
        .and_then(Value::as_str)
        .and_then(|s| serde_json::from_str::<Value>(s).ok())
    {
        Some(meta) if meta.is_object() && meta.get("user_emotion_scores").is_none() => {}
        _ => failures
            .push("assistant metadata invalid or incorrectly owns user emotion scores".into()),
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(case: &Case) -> (Vec<Value>, Value, Value) {
        (
            vec![
                json!({"prompt":format!("{}\n【最新用户消息】\n用户说: {}\n",case.tone(),case.message)}),
            ],
            json!({"emotion":case.scores(), "user_message_id":"u"}),
            json!({"chat_messages":[
                {"id":"u","session_id":"a-probe-role","sender":"user","content":case.message,
                 "metadata":json!({"user_emotion":case.label(),"user_emotion_scores":case.scores()}).to_string()},
                {"id":"a","session_id":"a-probe-role","sender":"assistant","metadata":"{}"}
            ]}),
        )
    }

    #[test]
    fn e_v1_closed_ids_and_messages_match_independent_literals() {
        let expected = [
            ("e1", "A-EV1-4b7ccc66-E01-R0", "我很开心"),
            ("e2", "A-EV1-4b7ccc66-E02-R0", "今天星期三"),
            ("e3", "A-EV1-4b7ccc66-E03-R0", "我不开心"),
            ("e4", "A-EV1-4b7ccc66-E04-R0", "嗯嗯好的"),
        ];
        assert_eq!(CASES.len(), expected.len());
        for (case, (scenario, id, message)) in CASES.iter().zip(expected) {
            assert_eq!(
                (case.scenario, case.run_id, case.message),
                (scenario, id, message)
            );
            assert!(crate::support::validate_run_mode(id, scenario, "").is_ok());
            assert!(crate::support::validate_run_mode(id, scenario, "live").is_err());
            assert!(crate::support::validate_run_mode("A-EV1-other", scenario, "").is_err());
        }
        assert!(validate_mode(CASES[0].run_id, "e2", "").is_err());
        assert!(validate_mode(CASES[0].run_id, "e5", "").is_err());
        for (driver, scenario, message) in [
            ("e1_driver_joy_emotion_turn", "e1", "我很开心"),
            ("e2_driver_no_clue_emotion_turn", "e2", "今天星期三"),
            ("e3_driver_negated_emotion_turn", "e3", "我不开心"),
            ("e4_driver_neutral_clue_emotion_turn", "e4", "嗯嗯好的"),
        ] {
            assert_eq!(crate::support::scenario_of_driver(driver), scenario);
            assert_eq!(crate::support::user_message(scenario), message);
            assert_eq!(crate::support::scripted_main_calls(scenario), 1);
            assert_eq!(
                crate::support::expected_reply(scenario),
                Some("我明白了，这就来。")
            );
            assert_eq!(crate::script_for(scenario).len(), 1);
        }
    }

    #[test]
    fn e_v1_literal_oracles_match_current_builtin_without_io() {
        use oclive_kernel_runtime::domain::emotion_analyzer::EmotionAnalyzer;
        for case in CASES {
            let actual = EmotionAnalyzer::analyze(case.message).unwrap();
            let v = json!({"joy":actual.joy,"sadness":actual.sadness,"anger":actual.anger,
                "fear":actual.fear,"surprise":actual.surprise,"disgust":actual.disgust,"neutral":actual.neutral});
            assert_eq!(v, case.scores());
            assert_eq!(actual.to_emotion().to_string(), case.label());
            assert_eq!(
                format!(
                    "用户语气线索：{}",
                    EmotionAnalyzer::format_for_prompt(&actual)
                ),
                case.tone()
            );
        }
    }

    #[test]
    fn e_v1_verifier_accepts_all_four_and_rejects_cross_case() {
        for case in CASES {
            let (p, r, s) = sample(case);
            assert!(verify(case.scenario, &p, &r, &s).is_empty());
            let wrong = if case.scenario == "e1" { "e2" } else { "e1" };
            assert!(!verify(wrong, &p, &r, &s).is_empty());
        }
    }

    #[test]
    fn e_v1_verifier_rejects_missing_or_wrong_response_scores() {
        let (p, r, s) = sample(&CASES[0]);
        for key in [
            "joy", "sadness", "anger", "fear", "surprise", "disgust", "neutral",
        ] {
            let mut bad = r.clone();
            bad["emotion"].as_object_mut().unwrap().remove(key);
            assert!(!verify("e1", &p, &bad, &s).is_empty());
            let mut bad = r.clone();
            bad["emotion"][key] = json!(0.5);
            assert!(!verify("e1", &p, &bad, &s).is_empty());
        }
    }

    #[test]
    fn e_v1_verifier_rejects_wrong_missing_or_repeated_tone() {
        let (p, r, s) = sample(&CASES[0]);
        assert!(!verify("e1", &[], &r, &s).is_empty());
        for text in [
            "我很开心".to_string(),
            p[0]["prompt"]
                .as_str()
                .unwrap()
                .replace("标签 happy", "标签 neutral"),
            format!("{}{}", p[0]["prompt"].as_str().unwrap(), CASES[0].tone()),
        ] {
            assert!(!verify("e1", &[json!({"prompt":text})], &r, &s).is_empty());
        }
    }

    #[test]
    fn e_v1_verifier_rejects_lost_swapped_or_misattributed_metadata() {
        let (p, r, s) = sample(&CASES[0]);
        for invalid in [
            Value::Null,
            json!("not-json"),
            json!("{}"),
            json!(
                json!({"user_emotion":"neutral","user_emotion_scores":CASES[1].scores()})
                    .to_string()
            ),
        ] {
            let mut bad = s.clone();
            bad["chat_messages"][0]["metadata"] = invalid;
            assert!(!verify("e1", &p, &r, &bad).is_empty());
        }
        let mut bad = s.clone();
        bad["chat_messages"][0]["id"] = json!("wrong");
        assert!(!verify("e1", &p, &r, &bad).is_empty());
        let mut bad = s.clone();
        bad["chat_messages"][1]["metadata"] = bad["chat_messages"][0]["metadata"].clone();
        assert!(!verify("e1", &p, &r, &bad).is_empty());
        let mut bad = s.clone();
        bad["chat_messages"][0]["session_id"] = json!("foreign");
        assert!(!verify("e1", &p, &r, &bad).is_empty());
    }

    #[test]
    fn e_v1_host_label_and_score_key_are_distinct_at_both_observation_points() {
        // Literal expected payloads, independent of Case::label/tone/scores and
        // the production formatter: changing an oracle must not silently change
        // this example to agree with it.
        let p = vec![json!({"prompt":
            "用户语气线索：偏愉快、积极或感激，可先共鸣再展开（标签 happy，信号强度：强）\n【最新用户消息】\n用户说: 我很开心\n"})];
        let scores = json!({"joy":1.0,"sadness":0.0,"anger":0.0,"fear":0.0,
            "surprise":0.0,"disgust":0.0,"neutral":0.0});
        let r = json!({"emotion":scores,"user_message_id":"u"});
        let s = json!({"chat_messages":[
            {"id":"u","session_id":"a-probe-role","sender":"user","content":"我很开心",
             "metadata":"{\"user_emotion\":\"happy\",\"user_emotion_scores\":{\"joy\":1.0,\"sadness\":0.0,\"anger\":0.0,\"fear\":0.0,\"surprise\":0.0,\"disgust\":0.0,\"neutral\":0.0}}"},
            {"id":"a","session_id":"a-probe-role","sender":"assistant","metadata":"{}"}
        ]});
        assert!(verify("e1", &p, &r, &s).is_empty());

        let mut bad_prompt = p.clone();
        bad_prompt[0]["prompt"] = json!(p[0]["prompt"]
            .as_str()
            .unwrap()
            .replace("标签 happy", "标签 joy"));
        assert!(!verify("e1", &bad_prompt, &r, &s).is_empty());

        let mut bad_metadata = s.clone();
        let mut meta: Value =
            serde_json::from_str(s["chat_messages"][0]["metadata"].as_str().unwrap()).unwrap();
        meta["user_emotion"] = json!("joy");
        bad_metadata["chat_messages"][0]["metadata"] = json!(meta.to_string());
        assert!(!verify("e1", &p, &r, &bad_metadata).is_empty());

        let mut bad_scores = r.clone();
        let joy = bad_scores["emotion"]
            .as_object_mut()
            .unwrap()
            .remove("joy")
            .unwrap();
        bad_scores["emotion"]["happy"] = joy;
        assert!(!verify("e1", &p, &bad_scores, &s).is_empty());

        // A correct neutral hint plus an extra happy hint must still be rejected.
        let (mut neutral_prompt, neutral_response, neutral_rows) = sample(&CASES[1]);
        neutral_prompt[0]["prompt"] = json!(format!(
            "{}（标签 happy，",
            neutral_prompt[0]["prompt"].as_str().unwrap()
        ));
        assert!(!verify("e2", &neutral_prompt, &neutral_response, &neutral_rows).is_empty());
    }
}
