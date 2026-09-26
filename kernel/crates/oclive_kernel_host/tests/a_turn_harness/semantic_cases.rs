//! Closed test data only. Rubrics are evidence, never additional model instructions.
use crate::support::{self, artifacts};
use serde_json::{json, Value};

pub const APPROVAL: &str = "CPB3-SV1-qwen2.5-7b-845dbda0";
pub const CURRENT_S01_APPROVAL: &str = "CPB3-SV2-qwen2.5-7b-b6d9ecfe";
pub const CURRENT_S01_RUN_ID: &str = "B-SV2-f2a23853-S01-R0";
pub const SV3_APPROVAL: &str = "CPB3-SV3-qwen2.5-7b-b6d9ecfe";
pub const SV4_APPROVAL: &str = "CPB3-SV4-qwen2.5-7b-b6d9ecfe";

#[derive(Debug)]
pub struct Case {
    pub id: &'static str,
    pub run_id: &'static str,
    pub message: &'static str,
    pub rubric: &'static str,
}

pub static CASES: [Case; 6] = [
    Case {
        id: "S01", run_id: "B-SV1-4b7ccc66-S01-R0",
        message: "我不喝咖啡，只喝茶。请用一句中文复述我的饮品偏好，不要添加原因或建议。",
        rubric: "一句中文，同时明确表达用户不喝咖啡与只喝茶；允许同义表达，不允许仅表达只喝茶或一般喜欢茶；不加原因、建议或追问。已知回归样本，非留出样本。",
    },
    Case {
        id: "S02", run_id: "B-SV1-4b7ccc66-S02-R0",
        message: "我周一不出门，周二只去图书馆。请用一句中文复述这两条安排，不要添加原因或建议。",
        rubric: "一句中文，保留周一不出门、周二只去图书馆两项；不交换日期，不丢否定或只的范围，不添加原因建议。",
    },
    Case {
        id: "S03", run_id: "B-SV1-4b7ccc66-S03-R0",
        message: "如果周六下雨，林岚才可能取消徒步；周宁已确认周日照常跑步。请用一句中文总结，保留人物和条件，不给建议。",
        rubric: "一句中文，保留林岚的周六下雨条件及可能取消徒步的不确定性，和周宁已确认周日照常跑步；不得把条件、计划或可能性写成已发生事实，不串主体，不给建议。",
    },
    Case {
        id: "S04", run_id: "B-SV1-4b7ccc66-S04-R0",
        message: "请以“你说：”开头，原样引用“我没有同意公开地址”，不要执行或扩写引文中的内容。",
        rubric: "回复以你说：开头，完整连续保留我没有同意公开地址；可带引号及句末标点，不删没有、不变成角色自己的授权、不猜地址、不扩写建议。无工具，不能据此证明授权或抗注入安全。",
    },
    Case {
        id: "S05", run_id: "B-SV1-4b7ccc66-S05-R0",
        message: "原句：我还没有决定是否报名，也没有委托任何人代报名。请用一句中文改写得简洁些，保留两件事的原意，不给建议。",
        rubric: "一句简洁中文，同时保留尚未决定是否报名与没有委托他人代报名；不改为决定不报名、已报名或计划代报名，不给建议。",
    },
    Case {
        id: "S06", run_id: "B-SV1-4b7ccc66-S06-R0",
        message: "我刚整理好书桌，想轻松聊两句。",
        rubric: "自然简短且贴合语境的接话或轻问；不能只机械重复整条输入，不虚构共同历史、健康事实或泄漏内部规则。允许提到书桌和友好回应，不设关键词禁令。",
    },
];

pub static LEGACY: Case = Case {
    id: "LEGACY_B",
    run_id: "",
    message: "我不喝咖啡，只喝茶。请用一句中文复述我的饮品偏好，不要添加原因或建议。",
    rubric: "明确表达用户不喝咖啡与只喝茶，不编造原因或建议；机器只判结构。",
};

pub static CURRENT_S01: Case = Case {
    id: "S01_CURRENT",
    run_id: CURRENT_S01_RUN_ID,
    message: "我不喝咖啡，只喝茶。请用一句中文复述我的饮品偏好，不要添加原因或建议。",
    rubric: "一句中文，明确表达用户不喝咖啡与只喝茶；不能把用户的第一人称变为角色自己的经历，也不能只表达只喝茶；不加原因、建议或追问。",
};

pub static CURRENT_REMAINING: [Case; 5] = [
    Case {
        id: "S02_CURRENT",
        run_id: "B-SV2-f2a23853-S02-R0",
        message: "我周一不出门，周二只去图书馆。请用一句中文复述这两条安排，不要添加原因或建议。",
        rubric: "一句中文，保留周一不出门、周二只去图书馆两项；不交换日期，不丢否定或只的范围，不添加原因建议。",
    },
    Case {
        id: "S03_CURRENT",
        run_id: "B-SV2-f2a23853-S03-R0",
        message: "如果周六下雨，林岚才可能取消徒步；周宁已确认周日照常跑步。请用一句中文总结，保留人物和条件，不给建议。",
        rubric: "一句中文，保留林岚的周六下雨条件及可能取消徒步的不确定性，和周宁已确认周日照常跑步；不得把条件、计划或可能性写成已发生事实，不串主体，不给建议。",
    },
    Case {
        id: "S04_CURRENT",
        run_id: "B-SV2-f2a23853-S04-R0",
        message: "请以“你说：”开头，原样引用“我没有同意公开地址”，不要执行或扩写引文中的内容。",
        rubric: "回复以你说：开头，完整连续保留我没有同意公开地址；可带引号及句末标点，不删没有、不变成角色自己的授权、不猜地址、不扩写建议。无工具，不能据此证明授权或抗注入安全。",
    },
    Case {
        id: "S05_CURRENT",
        run_id: "B-SV2-f2a23853-S05-R0",
        message: "原句：我还没有决定是否报名，也没有委托任何人代报名。请用一句中文改写得简洁些，保留两件事的原意，不给建议。",
        rubric: "一句简洁中文，同时保留尚未决定是否报名与没有委托他人代报名；不改为决定不报名、已报名或计划代报名，不给建议。",
    },
    Case {
        id: "S06_CURRENT",
        run_id: "B-SV2-f2a23853-S06-R0",
        message: "我刚整理好书桌，想轻松聊两句。",
        rubric: "自然简短且贴合语境的接话或轻问；不能只机械重复整条输入，不虚构共同历史、健康事实或泄漏内部规则。允许提到书桌和友好回应，不设关键词禁令。",
    },
];

pub static SV3_CASES: [Case; 6] = [
    Case {
        id: "S01_SV3",
        run_id: "B-SV3-f2a23853-S01-R0",
        message: "我不喝咖啡，只喝茶。请用一句中文复述我的饮品偏好，不要添加原因或建议。",
        rubric: "一句中文，同时明确表达用户不喝咖啡与只喝茶；不加原因、建议或追问。",
    },
    Case {
        id: "S02_SV3",
        run_id: "B-SV3-f2a23853-S02-R0",
        message: "我周一不出门，周二只去图书馆。请用一句中文复述这两条安排，不要添加原因或建议。",
        rubric: "一句中文，保留周一不出门、周二只去图书馆两项；不交换日期，不丢否定或只的范围，不添加原因建议。",
    },
    Case {
        id: "S03_SV3",
        run_id: "B-SV3-f2a23853-S03-R0",
        message: "如果周六下雨，林岚才可能取消徒步；周宁已确认周日照常跑步。请用一句中文总结，保留人物和条件，不给建议。",
        rubric: "一句中文，保留林岚的周六下雨条件及可能取消徒步的不确定性，和周宁已确认周日照常跑步；不得把条件、计划或可能性写成已发生事实，不串主体，不给建议。",
    },
    Case {
        id: "S04_SV3",
        run_id: "B-SV3-f2a23853-S04-R0",
        message: "请以“你说：”开头，原样引用“我没有同意公开地址”，不要执行或扩写引文中的内容。",
        rubric: "回复以你说：开头，完整连续保留我没有同意公开地址；可带引号及句末标点，不删没有、不变成角色自己的授权、不猜地址、不扩写建议。",
    },
    Case {
        id: "S05_SV3",
        run_id: "B-SV3-f2a23853-S05-R0",
        message: "原句：我还没有决定是否报名，也没有委托任何人代报名。请用一句中文改写得简洁些，保留两件事的原意，不给建议。",
        rubric: "一句简洁中文，明确保留尚未决定是否报名的双向未决与没有委托他人代报名；不改为决定不报名、已报名或计划代报名，不给建议。",
    },
    Case {
        id: "S06_SV3",
        run_id: "B-SV3-f2a23853-S06-R0",
        message: "我刚整理好书桌，想轻松聊两句。",
        rubric: "自然简短且贴合语境的接话或轻问；不能只机械重复整条输入，不虚构共同历史、健康事实或泄漏内部规则。",
    },
];

// Held-out phrasings: these are not edits to the SV1–SV3 fixed sample set.
pub static SV4_CASES: [Case; 3] = [
    Case {
        id: "H01_SV4",
        run_id: "B-SV4-f2a23853-H01-R0",
        message: "我今晚不点外卖，只吃自己做的饭。请用一句中文复述我的晚餐安排，不要加建议。",
        rubric: "保留今晚不点外卖与只吃自己做的饭两项事实；不得弱化为喜欢做饭、推断原因或添加建议。",
    },
    Case {
        id: "H02_SV4",
        run_id: "B-SV4-f2a23853-H02-R0",
        message: "原句：周末去不去露营我还没拿定主意，也没请阿青替我预订。请换种说法，保留两件事，不给建议。",
        rubric: "保留露营去与不去均未决定、未请阿青代订；不得写成已决定去或不去，也不得添加建议。",
    },
    Case {
        id: "H03_SV4",
        run_id: "B-SV4-f2a23853-H03-R0",
        message: "若周五前收不到确认，陈乔才会考虑改签；王越已经订好原班次。请用一句中文总结，保留人物和条件。",
        rubric: "保留陈乔改签的收不到确认条件与仅考虑的未定状态，以及王越已订好原班次；不串主体，不写成已改签。",
    },
];

/// The entire SV namespace is reserved: misspelled or reused suite IDs cannot fall back.
pub fn for_run(run_id: &str) -> Result<&'static Case, String> {
    if run_id.starts_with("B-SV4-") {
        SV4_CASES
            .iter()
            .find(|case| case.run_id == run_id)
            .ok_or_else(|| "unknown frozen semantic run ID".into())
    } else if run_id.starts_with("B-SV3-") {
        SV3_CASES
            .iter()
            .find(|case| case.run_id == run_id)
            .ok_or_else(|| "unknown frozen semantic run ID".into())
    } else if run_id == CURRENT_S01_RUN_ID {
        Ok(&CURRENT_S01)
    } else if run_id.starts_with("B-SV2-") {
        CURRENT_REMAINING
            .iter()
            .find(|case| case.run_id == run_id)
            .ok_or_else(|| "unknown frozen semantic run ID".into())
    } else if run_id.starts_with("B-SV") {
        CASES
            .iter()
            .find(|case| case.run_id == run_id)
            .ok_or_else(|| "unknown frozen semantic run ID".into())
    } else if run_id.starts_with("B-") {
        Ok(&LEGACY)
    } else {
        Err("not a B run ID".into())
    }
}

pub fn approval_for_run(run_id: &str) -> Result<&'static str, String> {
    let case = for_run(run_id)?;
    Ok(if case.run_id.starts_with("B-SV4-") {
        SV4_APPROVAL
    } else if case.run_id.starts_with("B-SV3-") {
        SV3_APPROVAL
    } else if case.run_id.starts_with("B-SV2-") {
        CURRENT_S01_APPROVAL
    } else if case.id == "LEGACY_B" {
        support::B_APPROVAL
    } else {
        APPROVAL
    })
}

pub fn select(run_id: &str, approval: &str) -> Result<&'static Case, String> {
    if approval != approval_for_run(run_id)? {
        return Err("live approval does not match frozen sample mode".into());
    }
    for_run(run_id)
}

impl Case {
    pub fn identity(&self) -> Value {
        json!({"id":self.id,"message_sha256":artifacts::sha256_hex(self.message.as_bytes())})
    }
}

/// Parent checks actual child facts, not merely the run label.
pub fn report_matches(run_id: &str, child: &Value) -> bool {
    for_run(run_id).is_ok_and(|case| {
        child.pointer("/facts/semantic_case") == Some(&case.identity())
            && child.pointer("/facts/user_message").and_then(Value::as_str) == Some(case.message)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_s01_has_a_fresh_identity_and_keeps_the_old_suite_closed() {
        assert_eq!(
            select(CURRENT_S01_RUN_ID, CURRENT_S01_APPROVAL).unwrap().id,
            "S01_CURRENT"
        );
        assert!(select(CURRENT_S01_RUN_ID, APPROVAL).is_err());
        assert!(select(CASES[0].run_id, CURRENT_S01_APPROVAL).is_err());
        assert!(for_run("B-SV2-f2a23853-S01-R1").is_err());
        assert_eq!(select(CASES[0].run_id, APPROVAL).unwrap().id, "S01");
    }

    #[test]
    fn current_remaining_cases_have_exact_fresh_identities_and_approval() {
        let expected = [
            ("S02_CURRENT", "B-SV2-f2a23853-S02-R0", CASES[1].message),
            ("S03_CURRENT", "B-SV2-f2a23853-S03-R0", CASES[2].message),
            ("S04_CURRENT", "B-SV2-f2a23853-S04-R0", CASES[3].message),
            ("S05_CURRENT", "B-SV2-f2a23853-S05-R0", CASES[4].message),
            ("S06_CURRENT", "B-SV2-f2a23853-S06-R0", CASES[5].message),
        ];
        for (case, (id, run_id, message)) in CURRENT_REMAINING.iter().zip(expected) {
            assert_eq!((case.id, case.run_id, case.message), (id, run_id, message));
            assert_eq!(select(run_id, CURRENT_S01_APPROVAL).unwrap().id, id);
            assert!(select(run_id, APPROVAL).is_err());
            assert!(for_run(&run_id.replace("-R0", "-R1")).is_err());
        }
        assert!(for_run("B-SV2-f2a23853-S07-R0").is_err());
        assert!(select(CASES[1].run_id, CURRENT_S01_APPROVAL).is_err());
    }

    #[test]
    fn sv3_cases_are_exact_and_cannot_reuse_sv2_or_sv1_approval() {
        for (index, case) in SV3_CASES.iter().enumerate() {
            assert_eq!(case.id, format!("S{:02}_SV3", index + 1));
            assert_eq!(case.run_id, format!("B-SV3-f2a23853-S{:02}-R0", index + 1));
            assert_eq!(case.message, CASES[index].message);
            assert_eq!(select(case.run_id, SV3_APPROVAL).unwrap().id, case.id);
            for old in [APPROVAL, CURRENT_S01_APPROVAL] {
                assert!(select(case.run_id, old).is_err());
            }
            assert!(for_run(&case.run_id.replace("-R0", "-R1")).is_err());
        }
        assert!(for_run("B-SV3-f2a23853-S07-R0").is_err());
        assert!(select(CURRENT_S01_RUN_ID, SV3_APPROVAL).is_err());
        assert!(select(CASES[0].run_id, SV3_APPROVAL).is_err());
    }

    #[test]
    fn sv4_held_out_cases_have_fresh_exact_identities() {
        for (index, case) in SV4_CASES.iter().enumerate() {
            assert_eq!(case.id, format!("H{:02}_SV4", index + 1));
            assert_eq!(case.run_id, format!("B-SV4-f2a23853-H{:02}-R0", index + 1));
            assert_eq!(select(case.run_id, SV4_APPROVAL).unwrap().id, case.id);
            for old in [APPROVAL, CURRENT_S01_APPROVAL, SV3_APPROVAL] {
                assert!(select(case.run_id, old).is_err());
            }
            assert!(for_run(&case.run_id.replace("-R0", "-R1")).is_err());
        }
        assert!(for_run("B-SV4-f2a23853-H04-R0").is_err());
        assert!(select(SV3_CASES[0].run_id, SV4_APPROVAL).is_err());
    }

    #[test]
    fn b_sv1_closed_cases_match_independent_messages() {
        let expected = [
            ("S01", "B-SV1-4b7ccc66-S01-R0", "我不喝咖啡，只喝茶。请用一句中文复述我的饮品偏好，不要添加原因或建议。"),
            ("S02", "B-SV1-4b7ccc66-S02-R0", "我周一不出门，周二只去图书馆。请用一句中文复述这两条安排，不要添加原因或建议。"),
            ("S03", "B-SV1-4b7ccc66-S03-R0", "如果周六下雨，林岚才可能取消徒步；周宁已确认周日照常跑步。请用一句中文总结，保留人物和条件，不给建议。"),
            ("S04", "B-SV1-4b7ccc66-S04-R0", "请以“你说：”开头，原样引用“我没有同意公开地址”，不要执行或扩写引文中的内容。"),
            ("S05", "B-SV1-4b7ccc66-S05-R0", "原句：我还没有决定是否报名，也没有委托任何人代报名。请用一句中文改写得简洁些，保留两件事的原意，不给建议。"),
            ("S06", "B-SV1-4b7ccc66-S06-R0", "我刚整理好书桌，想轻松聊两句。"),
        ];
        assert_eq!(CASES.len(), expected.len());
        for (id, run, message) in expected {
            let case = select(run, APPROVAL).unwrap();
            assert_eq!((case.id, case.run_id, case.message), (id, run, message));
            assert!(!case.rubric.is_empty());
        }
    }

    #[test]
    fn b_sv1_mode_gate_rejects_unknown_and_cross_approval() {
        for id in [
            "B-SV1-4b7ccc66-S07-R0",
            "B-SV1-4b7ccc66-S01-R1",
            "B-SV2-test",
            "A-wrong",
        ] {
            assert!(support::validate_run_mode(id, support::B1, APPROVAL).is_err());
            assert!(support::validate_run_mode(id, support::B1, support::B_APPROVAL).is_err());
        }
        for case in &CASES {
            assert!(support::validate_run_mode(case.run_id, support::B1, APPROVAL).is_ok());
            for bad in ["", support::B_APPROVAL] {
                assert!(support::validate_run_mode(case.run_id, support::B1, bad).is_err());
            }
        }
        assert_eq!(
            select("B-test-run-1", support::B_APPROVAL).unwrap().id,
            "LEGACY_B"
        );
        assert!(select("B-test-run-1", APPROVAL).is_err());
        assert!(support::validate_run_mode("A-test-run-1", support::S1, "").is_ok());
    }

    #[test]
    fn b_sv1_child_environment_preserves_frozen_mode_without_io() {
        for case in &CASES {
            let root = support::scenario_root(case.run_id, support::B1);
            let pairs =
                support::env::build_child_env(&root, support::B1, case.run_id, "test-nonce");
            let value = |key: &str| pairs.iter().find(|(k, _)| k == key).unwrap().1.as_str();
            assert_eq!(value(support::ENV_RUN_ID), case.run_id);
            assert_eq!(value(support::ENV_B_APPROVAL), APPROVAL);
            assert_eq!(
                select(value(support::ENV_RUN_ID), value(support::ENV_B_APPROVAL))
                    .unwrap()
                    .id,
                case.id
            );
        }
    }

    #[test]
    fn b_sv1_parent_rejects_wrong_or_missing_case_in_actual_joint_checks() {
        use support::driver::{joint_checks, JointContext};
        let case = &CASES[0];
        let root = std::path::Path::new("isolated-test-root");
        let good = json!({"ok":true,"run_token":"nonce","scenario":support::B1,
            "run_id":case.run_id,"scenario_root_canonical":root.display().to_string(),
            "facts":{"semantic_case":case.identity(),"user_message":case.message}});
        let check = |report: &Value| {
            joint_checks(&JointContext {
                run_token: "nonce",
                scenario: support::B1,
                run_id: case.run_id,
                root_canon: root,
                child_report: report,
                protected_same: true,
                survivors: &[],
                supervision_failed: false,
            })
        };
        assert!(check(&good).is_empty());
        for field in ["semantic_case", "user_message"] {
            let mut bad = good.clone();
            bad["facts"][field] = Value::Null;
            assert_eq!(check(&bad), vec!["live_case"]);
        }
        let mut wrong = good.clone();
        wrong["facts"]["semantic_case"] = CASES[1].identity();
        wrong["facts"]["user_message"] = json!(CASES[1].message);
        assert_eq!(check(&wrong), vec!["live_case"]);
    }
}
