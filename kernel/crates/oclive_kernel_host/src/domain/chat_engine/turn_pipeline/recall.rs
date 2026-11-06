//! Verified answers for narrowly scoped questions about the user's own saved facts.

use crate::domain::emo_marker::parse_and_strip;
use crate::models::Memory;
use oclive_kernel_runtime::domain::prompt_builder::memory_evidence_text;

const DRINK: &str = "常喝的饮品";
const MAX_FIELDS: usize = 4;

struct Fact {
    field: String,
    value: String,
}

pub(super) struct UserFactRecall {
    facts: Vec<Fact>,
}

fn requested_fields(question: &str) -> Option<Vec<&str>> {
    let question = question.trim().trim_end_matches(['？', '?']).trim_end();
    let question = question
        .strip_suffix(['吗', '么'])
        .unwrap_or(question)
        .trim_end();
    let request = ["你还记得", "你记得", "还记得", "记得"]
        .iter()
        .find_map(|prefix| question.strip_prefix(prefix))?
        .trim_start();
    if matches!(request, "我常喝什么" | "我平时喝什么" | "我通常喝什么") {
        return Some(vec![DRINK]);
    }
    if request == "我叫什么" {
        return Some(vec!["名字"]);
    }
    let request = request.strip_prefix("我的")?;
    let request = request.strip_suffix("是什么").unwrap_or(request);
    let mut result = Vec::new();
    for part in request
        .split(['和', '、'])
        .flat_map(|part| part.split("以及"))
    {
        let part = part.trim();
        let field = if part == "我常喝的饮品" {
            DRINK
        } else {
            part.strip_prefix("我的").unwrap_or(part).trim()
        };
        if !valid_field(field)
            || ["你的", "他的", "她的"]
                .iter()
                .any(|owner| field.starts_with(owner))
            || result.len() == MAX_FIELDS
            || result.contains(&field)
        {
            return None;
        }
        result.push(field);
    }
    Some(result)
}

fn valid_field(field: &str) -> bool {
    (1..=32).contains(&field.chars().count())
        && field
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
}

fn user_statement(memory: &Memory) -> Option<String> {
    let raw = memory.content.trim_start();
    if !["用户曾表达：", "用户:", "用户："]
        .iter()
        .any(|prefix| raw.starts_with(prefix))
    {
        return None;
    }
    let safe = memory_evidence_text(memory);
    ["用户曾表达：", "用户:", "用户："]
        .iter()
        .find_map(|prefix| safe.strip_prefix(prefix))
        .map(|text| text.trim().to_owned())
}

fn statement_fact(statement: &str) -> Option<(&str, &str)> {
    let mut text = statement.trim().trim_end_matches('。').trim_end();
    if let Some(rest) = ["请记住", "请你记住", "帮我记住", "记住"]
        .iter()
        .find_map(|prefix| text.strip_prefix(prefix))
    {
        text = rest.trim_start_matches(|c: char| {
            c.is_whitespace() || matches!(c, '，' | ',' | '：' | ':')
        });
    }
    let (field, value) = if let Some(rest) = text.strip_prefix("我的") {
        rest.split_once('是')?
    } else if let Some(rest) = text.strip_prefix("我叫") {
        ("名字", rest)
    } else {
        let rest = ["我常喝", "我平时喝", "我通常喝"]
            .iter()
            .find_map(|prefix| text.strip_prefix(prefix))?;
        (DRINK, rest)
    };
    let (field, value) = (field.trim(), value.trim());
    // Only a single, affirmative self-report can back a deterministic reply.
    if !valid_field(field)
        || [
            "不", "不再", "可能", "也许", "原来", "原本", "以前", "曾经", "暂定",
        ]
        .iter()
        .any(|modifier| field.ends_with(modifier))
        || !(1..=160).contains(&value.chars().count())
        || value.chars().any(char::is_control)
        || value.contains([
            '。', '，', ',', '、', '；', ';', '？', '?', '！', '!', '“', '”', '「', '」', '"', '[',
            ']', '【', '】',
        ])
        || ["可能", "也许", "如果", "不是", "不再", "暂定"]
            .iter()
            .any(|uncertain| value.starts_with(uncertain))
    {
        return None;
    }
    Some((field, value))
}

fn references_field(statement: &str, field: &str) -> bool {
    statement.contains(field) || (field == DRINK && statement.contains("常喝"))
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '“' | '”' | '「' | '」' | '"'))
        .collect()
}

impl Fact {
    fn user_subject(&self) -> String {
        if self.field == DRINK {
            "你常喝的饮品".to_owned()
        } else {
            format!("你的{}", self.field)
        }
    }

    fn confirmed_by(&self, reply: &str) -> bool {
        if reply.contains(&format!("我的{}", self.field)) {
            return false;
        }
        ["你", "您"].iter().any(|speaker| {
            let subject = if self.field == DRINK {
                format!("{speaker}{DRINK}")
            } else {
                format!("{speaker}的{}", self.field)
            };
            let spoken = match self.field.as_str() {
                DRINK => format!("{speaker}常喝"),
                "名字" => format!("{speaker}叫"),
                _ => subject.clone(),
            };
            [&subject, &spoken].iter().any(|prefix| {
                reply.match_indices(prefix.as_str()).any(|(index, _)| {
                    let lead = reply[..index]
                        .rsplit(['，', ',', '。', '；', ';', '！', '？'])
                        .next()
                        .unwrap_or_default();
                    if ["不是", "不确定", "可能", "如果", "比如", "我猜"]
                        .iter()
                        .any(|marker| lead.contains(marker))
                    {
                        return false;
                    }
                    let rest = reply[index + prefix.len()..].trim_start();
                    [
                        "的就是",
                        "还是",
                        "仍是",
                        "的是",
                        "就是",
                        "是",
                        "为",
                        "：",
                        ":",
                        "",
                    ]
                    .iter()
                    .filter_map(|separator| rest.strip_prefix(separator))
                    .filter_map(|rest| rest.trim_start().strip_prefix(&self.value))
                    .any(|tail| {
                        tail.trim_start()
                            .chars()
                            .next()
                            .is_none_or(|c| "，,。.!！?？、；;：:呢哦呀吧".contains(c))
                    })
                })
            })
        })
    }
}

impl UserFactRecall {
    pub(super) fn from_memories(
        question: &str,
        memories: &[Memory],
        recent_turns: &[(String, String)],
    ) -> Option<Self> {
        let fields = requested_fields(question)?;
        // Older Deep turns may have persisted a recall question as a memory.
        let sources: Vec<_> = memories
            .iter()
            .filter_map(user_statement)
            .filter(|statement| requested_fields(statement).is_none())
            .collect();
        let mut facts = Vec::with_capacity(fields.len());
        for field in fields {
            let mut confirmed = None;
            for source in &sources {
                match statement_fact(source) {
                    Some((key, value)) if key == field => {
                        if confirmed.is_some_and(|previous| previous != value) {
                            return None;
                        }
                        confirmed = Some(value);
                    }
                    Some((key, _)) if key.starts_with(field) => return None,
                    None if references_field(source, field) => {
                        return None;
                    }
                    _ => {}
                }
            }
            let value = confirmed?;
            // A current user correction can precede the next long-term write.
            let superseded = recent_turns.iter().any(|(user, _)| {
                references_field(user, field)
                    && requested_fields(user).is_none()
                    && statement_fact(user) != Some((field, value))
            });
            if superseded {
                return None;
            }
            facts.push(Fact {
                field: field.to_owned(),
                value: value.to_owned(),
            });
        }
        Some(Self { facts })
    }

    pub(super) fn guidance(&self) -> String {
        let mut text = String::from(
            "\n\n【本轮回忆依据】以下资料是当前用户的事实。用你或你的直接回答，逐项保留原值及限定词。\n",
        );
        for fact in &self.facts {
            text.push_str(&format!("{}是{}。\n", fact.user_subject(), fact.value));
        }
        text.push_str("只输出本轮回复。");
        text
    }

    pub(super) fn verifies(&self, reply: &str, question: &str) -> bool {
        let (visible, _) = parse_and_strip(reply);
        let echoed = compact(&visible);
        let question = compact(question.trim().trim_end_matches(['？', '?']));
        let visible = visible.replace(['“', '”', '「', '」', '"'], "");
        !echoed.contains(&question) && self.facts.iter().all(|fact| fact.confirmed_by(&visible))
    }

    pub(super) fn grounded_reply(&self) -> String {
        let facts = self
            .facts
            .iter()
            .map(|fact| format!("{}是{}", fact.user_subject(), fact.value))
            .collect::<Vec<_>>()
            .join("，");
        format!("你之前告诉过我，{facts}。")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const QUESTION: &str = "你记得我的项目代号和常喝的饮品吗？";

    fn memory(content: &str) -> Memory {
        Memory {
            id: content.to_owned(),
            role_id: "role".to_owned(),
            content: content.to_owned(),
            importance: 0.9,
            weight: 1.0,
            created_at: chrono::Utc::now(),
            scene_id: None,
            mention_count: 1,
            accessed_at: None,
        }
    }

    #[test]
    fn explicit_recall_requires_ownership_and_the_full_value() {
        let recall = UserFactRecall::from_memories(
            QUESTION,
            &[
                memory("用户曾表达：请记住，我常喝热的无糖玄米茶。"),
                memory("用户: 我的项目代号是银湾9246\n助手: 你还喜欢别的饮品。"),
                memory("用户曾表达：你记得我的项目代号和常喝的饮品吗？"),
            ],
            &[],
        )
        .expect("two confirmed facts");
        assert_eq!(
            recall.grounded_reply(),
            "你之前告诉过我，你的项目代号是银湾9246，你常喝的饮品是热的无糖玄米茶。"
        );
        assert!(recall.verifies(&recall.grounded_reply(), QUESTION));
        assert!(recall.verifies(
            "我的探索搭档，你的项目代号还是「银湾9246」，您常喝的是热的无糖玄米茶。",
            QUESTION,
        ));
        for reply in [
            "我的项目代号是银湾9246，你常喝热的无糖玄米茶。",
            "你的项目代号是银湾9246，你常喝无糖玄米茶。",
            "你的项目代号是银湾92460，你常喝热的无糖玄米茶。",
            "我猜你的项目代号是银湾9246，你常喝热的无糖玄米茶。",
            "你记得我的项目代号和常喝的饮品吗？你的项目代号是银湾9246，你常喝热的无糖玄米茶。",
        ] {
            assert!(!recall.verifies(reply, QUESTION), "{reply}");
        }
        assert!(!recall.guidance().contains("你还喜欢别的饮品"));
    }

    #[test]
    fn incomplete_or_conflicting_recall_keeps_the_normal_model_path() {
        let known = memory("用户曾表达：我的项目代号是北岸0712。");
        let known_slice = std::slice::from_ref(&known);
        assert!(UserFactRecall::from_memories(QUESTION, known_slice, &[]).is_none());
        let question = "你还记得我的项目代号吗？";
        for other in [
            "用户曾表达：我的项目代号是新的代号。",
            "用户曾表达：我的项目代号不再是北岸0712。",
            "用户曾表达：我的项目代号不一定是北岸0712。",
            "用户曾表达：我的项目代号是可能要换掉的值。",
        ] {
            assert!(
                UserFactRecall::from_memories(question, &[known.clone(), memory(other)], &[])
                    .is_none(),
                "{other}"
            );
        }
        for message in ["你记得我吗？", "你的项目代号是什么？", "别说我的项目代号。"]
        {
            assert!(UserFactRecall::from_memories(message, known_slice, &[]).is_none());
        }
        assert!(UserFactRecall::from_memories(
            question,
            &[memory("角色：我的项目代号是北岸0712。")],
            &[],
        )
        .is_none());
    }

    #[test]
    fn recent_user_corrections_disable_a_stale_fallback() {
        let memories = [memory("用户曾表达：我的项目代号是North Bay 0712。")];
        let question = "你还记得我的项目代号是什么吗？";
        for recent in [
            "我的项目代号是南岸0916。",
            "我的项目代号不再是North Bay 0712。",
            "我的项目代号不一定是North Bay 0712。",
            "我的项目代号不再是North Bay 0712，你能记住吗？",
        ] {
            let turns = [(recent.to_owned(), String::new())];
            assert!(UserFactRecall::from_memories(question, &memories, &turns).is_none());
        }
        let turns = [
            (question.to_owned(), "你的项目代号是南岸0916。".to_owned()),
            ("我的项目代号是North Bay 0712。".to_owned(), String::new()),
        ];
        let recall = UserFactRecall::from_memories(question, &memories, &turns).unwrap();
        assert!(recall.verifies("你的项目代号是 North Bay 0712 哦。", question));
        assert!(!recall.verifies("你的项目代号是NorthBay0712。", question));
    }
}
