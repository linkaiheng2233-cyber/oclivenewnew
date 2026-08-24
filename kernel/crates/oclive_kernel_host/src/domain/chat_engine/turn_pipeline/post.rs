//! Main LLM call entrypoints; post-LLM orchestration lives in [`post_llm`].

use crate::domain::chat_llm_fallback::{fallback_reply_for_llm_failure, FallbackReplyContext};
use crate::domain::chat_turn_rules::{
    soft_append_guard, strip_hallucination_tokens, strip_leading_role_label,
    trim_template_repeat_reply,
};
use crate::domain::slot_runner::SlotRunner;
use std::sync::Arc;
#[cfg(feature = "dual_core")]
use std::sync::Mutex;
use std::time::Instant;

use super::super::turn_context::TurnContext;
use super::super::turn_error::TurnResult;
use super::pre::{latest_recent_turn_pair, MainLlmOutput, MiddleOutput, PreLlmOutput};
use oclive_kernel_contracts::LlmGenerateOpts;
#[cfg(feature = "dual_core")]
use oclive_validation::plugin_backends_for_slot_entry;

mod post_llm;

pub(super) use post_llm::post_llm;

#[cfg(feature = "dual_core")]
fn selected_lora_llm(
    ctx: &TurnContext<'_>,
) -> Option<(String, String, Arc<dyn oclive_kernel_contracts::LlmClient>)> {
    let plugin_id = ctx.state.session_cache.expert_lora_plugin_id(ctx.srid)?;
    let registry = match ctx.session_config.slot_registry.as_ref() {
        Some(registry) => registry,
        None => {
            tracing::warn!(
                target: "oclive_expert",
                error_code = "LORA_ADAPTER_INVALID",
                session_ns = %ctx.srid,
                plugin_id = %plugin_id,
                "clearing LoRA selection because effective slot_registry is missing"
            );
            ctx.state
                .session_cache
                .set_expert_lora_plugin(ctx.srid, None);
            return None;
        }
    };
    let selection =
        match crate::domain::expert_routing::resolve_lora_llm_selection(registry, &plugin_id) {
            Ok(selection) => selection,
            Err(message) => {
                tracing::warn!(
                    target: "oclive_expert",
                    error_code = "LORA_ADAPTER_INVALID",
                    session_ns = %ctx.srid,
                    plugin_id = %plugin_id,
                    reason = %message,
                    "clearing invalid LoRA selection and using the normal LLM path"
                );
                ctx.state
                    .session_cache
                    .set_expert_lora_plugin(ctx.srid, None);
                return None;
            }
        };
    if let Err(message) = ctx
        .state
        .directory_plugins
        .ensure_rpc_url(&selection.plugin_id)
    {
        tracing::warn!(
            target: "oclive_expert",
            error_code = "LORA_ADAPTER_UNAVAILABLE",
            session_ns = %ctx.srid,
            plugin_id = %selection.plugin_id,
            slot_key = %selection.slot_key,
            reason = %message,
            "LoRA plugin unavailable; using the normal LLM path"
        );
        return None;
    }
    let backends = plugin_backends_for_slot_entry(&selection.entry);
    let llm = ctx.state.plugins.llm_for_plugin_backends(&backends);
    Some((selection.slot_key, selection.plugin_id, llm))
}

fn main_llm_generate_opts(ctx: &TurnContext<'_>, middle: &MiddleOutput) -> LlmGenerateOpts {
    let mut opts = if middle.use_ollama_prefix_opts {
        LlmGenerateOpts::deep_prefix_cache()
    } else {
        LlmGenerateOpts::interactive()
    };
    let Some(profile) = ctx
        .role
        .runtime_config
        .as_ref()
        .and_then(|config| config.inference_profile.as_ref())
    else {
        return opts;
    };

    apply_inference_profile(&mut opts, profile);
    opts
}

fn apply_inference_profile(
    opts: &mut LlmGenerateOpts,
    profile: &oclive_validation::InferenceProfileConfig,
) {
    if let Some(ref generation) = profile.generation {
        opts.temperature = generation.temperature;
        opts.top_p = generation.top_p;
        opts.max_output_tokens = generation
            .maximum_output_tokens
            .or(generation.preferred_output_tokens);
    }
    if let Some(ref context) = profile.context {
        opts.preferred_context_tokens = context.preferred_tokens;
    }
    if profile
        .performance_intent
        .as_ref()
        .and_then(|intent| intent.prefer_model_residency)
        == Some(false)
    {
        // Ollama treats an omitted keep_alive as its own residency default.
        // An explicit zero asks it to unload the model after this request.
        opts.keep_alive = Some("0".to_string());
    }
}

fn compact_dialogue_text(text: &str) -> String {
    text.chars()
        .filter(|ch| {
            !ch.is_whitespace()
                && !matches!(
                    ch,
                    '，' | ','
                        | '。'
                        | '.'
                        | '！'
                        | '!'
                        | '？'
                        | '?'
                        | '：'
                        | ':'
                        | '；'
                        | ';'
                        | '“'
                        | '”'
                        | '「'
                        | '」'
                        | '『'
                        | '』'
                        | '…'
                )
        })
        .collect()
}

fn rejected_reply_reason(
    raw: &str,
    user_message: &str,
    previous_reply: &str,
) -> Option<&'static str> {
    let (visible, _) = crate::domain::emo_marker::parse_and_strip(raw);
    let compact = compact_dialogue_text(&visible);
    if compact.is_empty() {
        return Some("没有可显示台词");
    }
    let user_compact = compact_dialogue_text(user_message);
    if !user_compact.is_empty() && compact == user_compact {
        return Some("原样照抄用户消息");
    }
    let visible_trimmed = visible.trim_start();
    let user_trimmed = user_message.trim();
    if user_trimmed.chars().count() >= 4
        && visible_trimmed
            .strip_prefix(user_trimmed)
            .is_some_and(|rest| rest.starts_with('\r') || rest.starts_with('\n'))
    {
        return Some("先逐字复述用户消息再作答");
    }
    if user_compact.chars().count() >= 4
        && compact.len() > user_compact.len()
        && compact.starts_with(user_compact.as_str())
    {
        return Some("先复述用户整句再继续作答");
    }
    let previous_compact = compact_dialogue_text(previous_reply);
    if !previous_compact.is_empty() && compact == previous_compact {
        return Some("原样重复上一轮回复");
    }
    if compact.chars().count() >= 4
        && user_compact.contains(compact.as_str())
        && (visible.trim_end().ends_with('？') || visible.trim_end().ends_with('?'))
    {
        return Some("把用户问题截短后反问回去");
    }
    if visible
        .chars()
        .any(|ch| matches!(ch as u32, 0x1F300..=0x1FAFF))
    {
        return Some("包含未请求的 emoji");
    }
    if matches!(visible.trim_end().chars().last(), Some('/' | '\\')) {
        return Some("句尾包含孤立斜杠");
    }
    if visible.matches("明白了").count() > 1 {
        return Some("同一回复重复确认语");
    }
    let fabricates_current_work = ["当前任务", "这次任务", "交付任务", "用户会不高兴"]
        .iter()
        .any(|phrase| visible.contains(phrase));
    let user_supplied_work = ["任务", "工作", "交付", "项目", "代码", "编译"]
        .iter()
        .any(|phrase| user_message.contains(phrase));
    if fabricates_current_work && !user_supplied_work {
        return Some("在普通闲聊中虚构当前工作任务");
    }
    let user_requested_pause = [
        "停一下",
        "先停",
        "停在这里",
        "安静一下",
        "不想说话",
        "不想热闹",
    ]
    .iter()
    .any(|phrase| user_message.contains(phrase));
    if user_requested_pause
        && [
            "需要帮忙",
            "尽管告诉",
            "需要我",
            "有需要",
            "想聊的话",
            "你今天",
            "我们可以",
            "也许",
            "希望你",
            "想你",
            "小G",
            "说说",
            "愿意的时候",
            "再说",
            "想说就说",
            "要是想说",
            "坐在这边",
            "如果",
            "随时",
            "想哭",
            "哭出来",
            "眼泪",
            "需要的话",
        ]
        .iter()
        .any(|phrase| visible.contains(phrase))
    {
        return Some("用户要求暂停后仍继续提供或索取互动");
    }
    if user_requested_pause && (visible.contains('？') || visible.contains('?')) {
        return Some("用户要求暂停后仍继续追问");
    }
    if visible.contains("用户：") || visible.contains("用户:") {
        return Some("包含用户说话人标签");
    }
    let greeting_only = ["你好", "早上好", "晚上好"]
        .iter()
        .any(|phrase| user_message.contains(phrase));
    if greeting_only && !user_message.contains("天气") && visible.contains("天气") {
        return Some("问候时臆测未提供的现实天气");
    }
    if user_message.contains("什么都做不好")
        && ["每个人", "学习和成长", "重要的是", "一起看看"]
            .iter()
            .any(|phrase| visible.contains(phrase))
    {
        return Some("用泛化鸡汤覆盖用户的具体挫败");
    }
    if (user_message.contains("你认同吗")
        || user_message.contains("最后直接回答")
        || user_message.contains("最后说说"))
        && (visible.contains('？') || visible.contains('?'))
    {
        return Some("明确要求直接回答后仍追加反问");
    }
    const META_PHRASES: &[&str] = &[
        "按照角色设定",
        "根据角色设定",
        "按照设定",
        "按照这个设定",
        "设定进行对话",
        "最新用户消息",
        "我将按照角色",
        "以角色身份",
        "的身份回复",
        "的角色出发",
        "角色台词",
        "进行回复",
        "生成回复",
        "角色设定",
        "对话规则",
        "内部字段",
        "回复质量锚点",
        "安全回退",
    ];
    META_PHRASES
        .iter()
        .any(|phrase| visible.contains(phrase))
        .then_some("包含生成过程或角色设定元话术")
}

fn prompt_contract_rejection(raw: &str, prompt: &str) -> Option<&'static str> {
    let (visible, _) = crate::domain::emo_marker::parse_and_strip(raw);
    let visible = visible.trim();
    let compact = compact_dialogue_text(visible);

    for line in prompt.lines().map(str::trim) {
        if let Some(value) = line
            .strip_prefix("【候选必须包含：")
            .and_then(|rest| rest.strip_suffix('】'))
        {
            let matched = value
                .split('|')
                .map(str::trim)
                .filter(|term| !term.is_empty())
                .any(|term| visible.contains(term));
            if !matched {
                return Some("未满足角色包当前消息的必要语义");
            }
        } else if let Some(value) = line
            .strip_prefix("【候选不得包含：")
            .and_then(|rest| rest.strip_suffix('】'))
        {
            let violated = value
                .split('|')
                .map(str::trim)
                .filter(|term| !term.is_empty())
                .any(|term| {
                    visible.match_indices(term).any(|(start, _)| {
                        let prefix = &visible[..start];
                        !prefix.ends_with("不再") && !prefix.ends_with("不会再")
                    })
                });
            if violated {
                return Some("包含角色包为当前消息禁止的措辞");
            }
        } else if let Some(value) = line
            .strip_prefix("【候选必须以：")
            .and_then(|rest| rest.strip_suffix('】'))
        {
            let matched = value
                .split('|')
                .map(str::trim)
                .filter(|term| !term.is_empty())
                .any(|term| visible.starts_with(term));
            if !matched {
                return Some("未以角色包要求的主体或结论起句");
            }
        } else if let Some(value) = line
            .strip_prefix("【候选不得等于：")
            .and_then(|rest| rest.strip_suffix('】'))
        {
            let violated = value
                .split('|')
                .map(compact_dialogue_text)
                .filter(|term| !term.is_empty())
                .any(|term| compact == term);
            if violated {
                return Some("候选只有口头禅或模板短句");
            }
        } else if let Some(value) = line
            .strip_prefix("【短语次数：")
            .and_then(|rest| rest.strip_suffix('】'))
        {
            if let Some((phrase, count)) = value.rsplit_once('=') {
                if let Ok(expected) = count.trim().parse::<usize>() {
                    let compact_phrase = compact_dialogue_text(phrase.trim());
                    let actual = if compact_phrase.is_empty() {
                        0
                    } else {
                        compact.matches(compact_phrase.as_str()).count()
                    };
                    if actual != expected {
                        return Some("角色口头禅次数不符合当前消息约束");
                    }
                }
            }
        }
    }
    None
}

fn prompt_contract_safe_reply(prompt: &str) -> Option<String> {
    prompt.lines().map(str::trim).find_map(|line| {
        line.strip_prefix("【安全回退：")
            .and_then(|rest| rest.strip_suffix('】'))
            .map(str::trim)
            .filter(|reply| !reply.is_empty())
            .map(str::to_string)
    })
}

fn candidate_rejection_reason(
    raw: &str,
    user_message: &str,
    previous_reply: &str,
    recent_turns: &[(String, String)],
    prompt: &str,
    role_name: &str,
) -> Option<&'static str> {
    if strip_leading_role_label(raw, role_name).trim() != raw.trim() {
        return Some("包含角色说话人标签");
    }
    let (visible, _) = crate::domain::emo_marker::parse_and_strip(raw);
    let compact_role = compact_dialogue_text(role_name);
    if !compact_role.is_empty()
        && compact_dialogue_text(&visible).starts_with(compact_role.as_str())
    {
        return Some("用角色名作第三人称自述");
    }
    let compact_candidate = compact_dialogue_text(&visible);
    let compact_user = compact_dialogue_text(user_message);
    if !compact_candidate.is_empty()
        && recent_turns.iter().any(|(recent_user, recent_reply)| {
            compact_dialogue_text(recent_user) != compact_user
                && compact_dialogue_text(recent_reply) == compact_candidate
        })
    {
        return Some("重复了更早话题的完整回复");
    }
    rejected_reply_reason(raw, user_message, previous_reply)
        .or_else(|| prompt_contract_rejection(raw, prompt))
}

fn selected_anchor_for_repair(prompt: &str) -> &str {
    let Some(end) = prompt.rfind("【对话硬约束】") else {
        return "";
    };
    let before_guardrails = &prompt[..end];
    let start = before_guardrails
        .rfind("【常驻锚点】")
        .or_else(|| before_guardrails.rfind("【当前消息专项校准】"));
    start.map_or("", |start| before_guardrails[start..].trim())
}

fn build_reply_repair_prompt(
    role: &crate::models::Role,
    user_message: &str,
    rejected_raw: &str,
    reason: &str,
    original_prompt: &str,
) -> String {
    let persona = role
        .deep_capsule
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(role.core_personality.as_str());
    let persona: String = persona.chars().take(1_200).collect();
    let (rejected_visible, _) = crate::domain::emo_marker::parse_and_strip(rejected_raw);
    format!(
        "你是{}。\n\n【角色短胶囊】\n{}\n\n【最新用户消息】\n{}\n\n【本轮专项约束】\n{}\n\n【上一候选被拒原因】\n{}\n\n【不得复用的上一候选】\n{}\n\n重新写一条只回应最新消息的自然角色台词，严格满足所有‘候选必须／不得／短语次数’约束。只输出台词正文，不输出分析、规则、标题、说话人标签、JSON 或情绪标记。",
        role.name,
        persona.trim(),
        user_message.trim(),
        selected_anchor_for_repair(original_prompt),
        reason,
        rejected_visible.trim()
    )
}

fn rejected_reply_fallback(user_message: &str) -> String {
    if ["停一下", "安静一下", "不想说话", "先停", "到这里"]
        .iter()
        .any(|phrase| user_message.contains(phrase))
    {
        "好，我们先停在这里。".to_string()
    } else {
        "我在听。请继续，我会直接回应眼前这句话。".to_string()
    }
}

pub(crate) async fn run_main_llm(
    ctx: &TurnContext<'_>,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
) -> TurnResult<MainLlmOutput> {
    let role = ctx.role;
    let user_message = ctx.req.user_message.as_str();
    let pl = &ctx.pl;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let mut llm_fallback_reason = None;
    let ollama_opts = Some(main_llm_generate_opts(ctx, middle));
    #[cfg(feature = "dual_core")]
    let selected_lora = selected_lora_llm(ctx);
    #[cfg(feature = "dual_core")]
    let generation = async {
        if let Some((slot_key, plugin_id, llm)) = selected_lora.as_ref() {
            tracing::info!(
                target: "oclive_expert",
                session_ns = %ctx.srid,
                plugin_id = %plugin_id,
                slot_key = %slot_key,
                "generating reply with selected LoRA directory LLM"
            );
            match SlotRunner::generate_llm_single(
                llm,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                ollama_opts.as_ref(),
            )
            .await
            {
                Ok(out) => Ok(out),
                Err(error) => {
                    tracing::warn!(
                        target: "oclive_expert",
                        error_code = "LORA_ADAPTER_GENERATE_FAILED",
                        session_ns = %ctx.srid,
                        plugin_id = %plugin_id,
                        slot_key = %slot_key,
                        reason = %error,
                        "LoRA generation failed; clearing selection and retrying the normal LLM"
                    );
                    ctx.state
                        .session_cache
                        .set_expert_lora_plugin(ctx.srid, None);
                    SlotRunner::generate_llm(
                        pl,
                        pre.memory.ollama_model.as_str(),
                        &middle.prompt,
                        ollama_opts.as_ref(),
                    )
                    .await
                }
            }
        } else {
            SlotRunner::generate_llm(
                pl,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                ollama_opts.as_ref(),
            )
            .await
        }
    };
    #[cfg(not(feature = "dual_core"))]
    let generation = SlotRunner::generate_llm(
        pl,
        pre.memory.ollama_model.as_str(),
        &middle.prompt,
        ollama_opts.as_ref(),
    );
    let mut reply_out = match generation.await {
        Ok(out) => out,
        Err(e) => {
            let reason = e.to_frontend_error();
            tracing::warn!("{path_label} LLM generate failed, fallback: {reason}");
            main_llm_fallback = true;
            llm_fallback_reason = Some(reason);
            let fallback = fallback_reply_for_llm_failure(
                role,
                &middle.personality,
                user_message,
                &FallbackReplyContext {
                    relation_before: pre.relation.relation_before.as_str(),
                    relation_preview: middle.relation_after.as_str(),
                    favorability_before: pre.relation.favorability_before,
                    event_type: &middle.ai_event_type,
                    impact_factor: middle.ai_impact_factor_final,
                },
            );
            oclive_kernel_contracts::LlmGenerateOutcome {
                reply: fallback,
                prompt_eval_ms: None,
            }
        }
    };
    let (_, previous_assistant_reply) = latest_recent_turn_pair(&pre.memory.recent_turns);
    if !main_llm_fallback {
        if let Some(mut reason) = candidate_rejection_reason(
            &reply_out.reply,
            user_message,
            previous_assistant_reply.as_str(),
            &pre.memory.recent_turns,
            &middle.prompt,
            role.name.as_str(),
        ) {
            let mut repaired_ok = false;
            for attempt in 1..=2 {
                let repair_prompt = build_reply_repair_prompt(
                    role,
                    user_message,
                    &reply_out.reply,
                    reason,
                    &middle.prompt,
                );
                match SlotRunner::generate_llm(
                    pl,
                    pre.memory.ollama_model.as_str(),
                    &repair_prompt,
                    ollama_opts.as_ref(),
                )
                .await
                {
                    Ok(repaired) => {
                        if let Some(next_reason) = candidate_rejection_reason(
                            &repaired.reply,
                            user_message,
                            previous_assistant_reply.as_str(),
                            &pre.memory.recent_turns,
                            &middle.prompt,
                            role.name.as_str(),
                        ) {
                            reason = next_reason;
                            continue;
                        }
                        tracing::info!(
                            target: "oclive_turn",
                            rejected_reason = reason,
                            repair_attempt = attempt,
                            "main reply candidate was rewritten"
                        );
                        reply_out = repaired;
                        repaired_ok = true;
                        break;
                    }
                    Err(_) => break,
                }
            }
            if !repaired_ok {
                if let Some(pack_reply) =
                    prompt_contract_safe_reply(&middle.prompt).filter(|reply| {
                        candidate_rejection_reason(
                            reply,
                            user_message,
                            previous_assistant_reply.as_str(),
                            &pre.memory.recent_turns,
                            &middle.prompt,
                            role.name.as_str(),
                        )
                        .is_none()
                    })
                {
                    tracing::warn!(
                        target: "oclive_turn",
                        rejected_reason = reason,
                        "main reply repair failed twice; using role-pack safe reply"
                    );
                    reply_out = oclive_kernel_contracts::LlmGenerateOutcome {
                        reply: pack_reply,
                        prompt_eval_ms: None,
                    };
                } else {
                    tracing::warn!(
                        target: "oclive_turn",
                        rejected_reason = reason,
                        "main reply repair failed twice; using non-empty safe fallback"
                    );
                    main_llm_fallback = true;
                    llm_fallback_reason = Some(format!("REPLY_QUALITY_REJECTED: {reason}"));
                    reply_out = oclive_kernel_contracts::LlmGenerateOutcome {
                        reply: rejected_reply_fallback(user_message),
                        prompt_eval_ms: None,
                    };
                }
            }
        }
    }
    if let (Some(hash), Some(len), Some(hit)) = (
        middle.prompt_stable_hash,
        middle.prompt_stable_len,
        middle.prefix_cache_expected_hit,
    ) {
        tracing::debug!(
            target: "oclive_turn",
            prefix_hash = hash,
            stable_len = len,
            cache_expected_hit = hit,
            prompt_eval_ms = ?reply_out.prompt_eval_ms,
            "prompt prefix cache llm metrics"
        );
    }
    let reply_raw = strip_leading_role_label(&reply_out.reply, role.name.as_str());
    let llm_prompt_eval_ms = reply_out.prompt_eval_ms;
    let main_llm_ms = t_main_llm.elapsed().as_millis() as u64;
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &trim_template_repeat_reply(previous_assistant_reply.as_str(), &reply_raw),
        &middle.ai_event_type,
        middle.ai_impact_factor_final,
        middle.relation_after.as_str(),
    ));

    Ok(MainLlmOutput {
        reply,
        main_llm_fallback,
        llm_fallback_reason,
        main_llm_ms,
        llm_prompt_eval_ms,
    })
}

pub(crate) async fn run_main_llm_stream(
    ctx: &TurnContext<'_>,
    path_label: &str,
    pre: &PreLlmOutput,
    middle: &MiddleOutput,
    on_token: oclive_kernel_contracts::LlmTokenSink,
) -> TurnResult<MainLlmOutput> {
    let role = ctx.role;
    let user_message = ctx.req.user_message.as_str();
    let pl = &ctx.pl;
    let t_main_llm = Instant::now();
    let mut main_llm_fallback = false;
    let mut llm_fallback_reason = None;
    let ollama_opts = Some(main_llm_generate_opts(ctx, middle));
    #[cfg(feature = "dual_core")]
    let selected_lora = selected_lora_llm(ctx);
    #[cfg(feature = "dual_core")]
    let generation = async {
        if let Some((slot_key, plugin_id, llm)) = selected_lora.as_ref() {
            tracing::info!(
                target: "oclive_expert",
                session_ns = %ctx.srid,
                plugin_id = %plugin_id,
                slot_key = %slot_key,
                "streaming reply with selected LoRA directory LLM"
            );
            let streamed = Arc::new(Mutex::new(String::new()));
            let streamed_for_sink = Arc::clone(&streamed);
            let downstream = Arc::clone(&on_token);
            let passthrough_sink: oclive_kernel_contracts::LlmTokenSink = Arc::new(move |token| {
                if let Ok(mut output) = streamed_for_sink.lock() {
                    output.push_str(token);
                }
                downstream(token);
            });
            match SlotRunner::generate_llm_stream_single(
                llm,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                passthrough_sink,
                ollama_opts.as_ref(),
            )
            .await
            {
                Ok(out) => Ok(out),
                Err(error) => {
                    let partial = streamed
                        .lock()
                        .map(|output| output.clone())
                        .unwrap_or_default();
                    ctx.state
                        .session_cache
                        .set_expert_lora_plugin(ctx.srid, None);
                    if !partial.is_empty() {
                        tracing::warn!(
                            target: "oclive_expert",
                            error_code = "LORA_ADAPTER_STREAM_PARTIAL",
                            session_ns = %ctx.srid,
                            plugin_id = %plugin_id,
                            slot_key = %slot_key,
                            emitted_bytes = partial.len(),
                            reason = %error,
                            "LoRA stream failed after emitting output; preserving the partial reply without duplicate fallback tokens"
                        );
                        return Ok(oclive_kernel_contracts::LlmGenerateOutcome {
                            reply: partial,
                            prompt_eval_ms: None,
                        });
                    }
                    tracing::warn!(
                        target: "oclive_expert",
                        error_code = "LORA_ADAPTER_GENERATE_FAILED",
                        session_ns = %ctx.srid,
                        plugin_id = %plugin_id,
                        slot_key = %slot_key,
                        reason = %error,
                        "LoRA stream failed before first token; retrying the normal LLM"
                    );
                    SlotRunner::generate_llm_stream(
                        pl,
                        pre.memory.ollama_model.as_str(),
                        &middle.prompt,
                        Arc::clone(&on_token),
                        ollama_opts.as_ref(),
                    )
                    .await
                }
            }
        } else {
            SlotRunner::generate_llm_stream(
                pl,
                pre.memory.ollama_model.as_str(),
                &middle.prompt,
                Arc::clone(&on_token),
                ollama_opts.as_ref(),
            )
            .await
        }
    };
    #[cfg(not(feature = "dual_core"))]
    let generation = SlotRunner::generate_llm_stream(
        pl,
        pre.memory.ollama_model.as_str(),
        &middle.prompt,
        Arc::clone(&on_token),
        ollama_opts.as_ref(),
    );
    let reply_out = match generation.await {
        Ok(out) => out,
        Err(e) => {
            let reason = e.to_frontend_error();
            tracing::warn!("{path_label} LLM generate_stream failed, fallback: {reason}");
            main_llm_fallback = true;
            llm_fallback_reason = Some(reason);
            let fallback = fallback_reply_for_llm_failure(
                role,
                &middle.personality,
                user_message,
                &FallbackReplyContext {
                    relation_before: pre.relation.relation_before.as_str(),
                    relation_preview: middle.relation_after.as_str(),
                    favorability_before: pre.relation.favorability_before,
                    event_type: &middle.ai_event_type,
                    impact_factor: middle.ai_impact_factor_final,
                },
            );
            on_token(fallback.as_str());
            oclive_kernel_contracts::LlmGenerateOutcome {
                reply: fallback,
                prompt_eval_ms: None,
            }
        }
    };
    let reply_raw = strip_leading_role_label(&reply_out.reply, role.name.as_str());
    let llm_prompt_eval_ms = reply_out.prompt_eval_ms;
    let main_llm_ms = t_main_llm.elapsed().as_millis() as u64;
    let (_, previous_assistant_reply) = latest_recent_turn_pair(&pre.memory.recent_turns);
    let reply = strip_hallucination_tokens(&soft_append_guard(
        &trim_template_repeat_reply(previous_assistant_reply.as_str(), &reply_raw),
        &middle.ai_event_type,
        middle.ai_impact_factor_final,
        middle.relation_after.as_str(),
    ));

    Ok(MainLlmOutput {
        reply,
        main_llm_fallback,
        llm_fallback_reason,
        main_llm_ms,
        llm_prompt_eval_ms,
    })
}

#[cfg(test)]
mod inference_profile_tests {
    use super::*;

    #[test]
    fn reply_quality_rejects_empty_echo_repeat_and_meta_but_accepts_normal_text() {
        assert_eq!(
            rejected_reply_reason(
                "[EMO]{\"labels\":[\"neutral\"],\"intensity\":0.3}[/EMO]",
                "停一下",
                ""
            ),
            Some("没有可显示台词")
        );
        assert_eq!(
            rejected_reply_reason("我想你了。", "我想你了。", ""),
            Some("原样照抄用户消息")
        );
        assert_eq!(
            rejected_reply_reason("我想你了，也会想起以前。", "我想你了。", ""),
            Some("先复述用户整句再继续作答")
        );
        assert_eq!(
            rejected_reply_reason("你怎么这么笨？\n\n这不是有效评价。", "你怎么这么笨？", ""),
            Some("先逐字复述用户消息再作答")
        );
        assert_eq!(
            rejected_reply_reason("上一条回复。", "换个话题", "上一条回复"),
            Some("原样重复上一轮回复")
        );
        assert_eq!(
            rejected_reply_reason("你认同吗？", "我不是你的下属，你认同吗？", ""),
            Some("把用户问题截短后反问回去")
        );
        assert_eq!(
            rejected_reply_reason("好的，我明白了。那就先停。明白了。", "停一下", ""),
            Some("同一回复重复确认语")
        );
        assert_eq!(
            rejected_reply_reason("你好呀。/", "你好", ""),
            Some("句尾包含孤立斜杠")
        );
        assert_eq!(
            rejected_reply_reason("好呀！😄", "想个小游戏", ""),
            Some("包含未请求的 emoji")
        );
        assert!(rejected_reply_reason("我将按照角色设定进行回复。", "你好", "").is_some());
        assert!(rejected_reply_reason("安全回退：先查 PID。", "怎么查", "").is_some());
        assert!(rejected_reply_reason("好的，我们按照设定来。", "先停在这里", "").is_some());
        assert!(rejected_reply_reason("理解了，用户：我不会立刻开心。", "开玩笑的", "").is_some());
        assert_eq!(
            rejected_reply_reason("你好，今天天气不错。", "你好", ""),
            Some("问候时臆测未提供的现实天气")
        );
        assert_eq!(
            rejected_reply_reason(
                "我还真有点饿，不过得先把这次任务搞定。",
                "你是不是又饿了？",
                ""
            ),
            Some("在普通闲聊中虚构当前工作任务")
        );
        assert_eq!(
            rejected_reply_reason(
                "好，我们停在这里。如果有需要帮忙的地方，尽管告诉我。",
                "先停在这里",
                ""
            ),
            Some("用户要求暂停后仍继续提供或索取互动")
        );
        assert_eq!(
            rejected_reply_reason(
                "一次失败谁都会遇到，重要的是学习和成长。",
                "我觉得自己什么都做不好",
                ""
            ),
            Some("用泛化鸡汤覆盖用户的具体挫败")
        );
        assert_eq!(
            rejected_reply_reason("好，我们先停在这里。", "我只想停一下", ""),
            None
        );
        assert_eq!(
            candidate_rejection_reason(
                "【GPT 龙娘】好，我们先停一下。",
                "我只想停一下",
                "",
                &[],
                "",
                "GPT龙娘"
            ),
            Some("包含角色说话人标签")
        );
        assert_eq!(
            candidate_rejection_reason(
                "奶龙娘眨了眨眼睛，说道：先查 PID。",
                "怎么查端口？",
                "",
                &[],
                "",
                "奶龙娘"
            ),
            Some("用角色名作第三人称自述")
        );
        let recent = vec![(
            "我想你了。".to_string(),
            "听到你这么说，我很开心，我也会想你。".to_string(),
        )];
        assert_eq!(
            candidate_rejection_reason(
                "听到你这么说，我很开心，我也会想你。",
                "我不需要你邀请我。",
                "",
                &recent,
                "",
                "温柔房东"
            ),
            Some("重复了更早话题的完整回复")
        );
    }

    #[test]
    fn role_pack_prompt_contract_checks_required_forbidden_prefix_and_phrase_count() {
        let prompt = "【候选必须包含：不建议|不要先】\n【候选不得包含：先试试】\n【候选必须以：不建议|不要】\n【短语次数：菲比啾比=1】";
        assert_eq!(
            prompt_contract_rejection("不建议先试试。菲比啾比。", prompt),
            Some("包含角色包为当前消息禁止的措辞")
        );
        assert_eq!(
            prompt_contract_rejection("不建议先清理。菲比啾比。菲比啾比。", prompt),
            Some("角色口头禅次数不符合当前消息约束")
        );
        assert_eq!(
            prompt_contract_rejection("不建议先清理。菲比啾比。", prompt),
            None
        );
        assert_eq!(
            prompt_contract_rejection("可以不说。菲比……啾比……", "【短语次数：菲比啾比=1】"),
            None
        );
        let safe_prompt = format!("{prompt}\n【安全回退：不建议先清理。菲比啾比。】");
        assert_eq!(
            prompt_contract_safe_reply(&safe_prompt).as_deref(),
            Some("不建议先清理。菲比啾比。")
        );
        assert_eq!(
            prompt_contract_rejection(
                "明白，我接受你的选择，不再邀请。",
                "【候选不得包含：邀请|下次】"
            ),
            None
        );
    }

    #[test]
    fn reply_repair_prompt_keeps_only_persona_latest_message_and_selected_anchor() {
        let role = crate::models::Role {
            name: "测试角色".to_string(),
            core_personality: "完整核心".to_string(),
            deep_capsule: Some("短胶囊".to_string()),
            ..crate::models::Role::default()
        };
        let original = "很长的旧内容\n【常驻锚点】\n只答当前。\n【当前消息专项校准】\n【候选必须以：我】\n【对话硬约束】\n后续规则";
        let repair = build_reply_repair_prompt(
            &role,
            "你怎么看我？",
            "你对我是朋友。",
            "主体倒置",
            original,
        );
        assert!(repair.contains("短胶囊"));
        assert!(repair.contains("你怎么看我"));
        assert!(repair.contains("【候选必须以：我】"));
        assert!(!repair.contains("很长的旧内容"));
        assert!(!repair.contains("后续规则"));
    }

    #[test]
    fn portable_inference_profile_maps_to_request_options() {
        let profile: oclive_validation::InferenceProfileConfig =
            serde_json::from_value(serde_json::json!({
                "generation": {
                    "temperature": 0.7,
                    "top_p": 0.85,
                    "preferred_output_tokens": 512,
                    "maximum_output_tokens": 1024
                },
                "context": { "preferred_tokens": 16384 },
                "performance_intent": { "prefer_model_residency": false }
            }))
            .expect("valid inference profile");
        let mut opts = LlmGenerateOpts::interactive();

        apply_inference_profile(&mut opts, &profile);

        assert_eq!(opts.temperature, Some(0.7));
        assert_eq!(opts.top_p, Some(0.85));
        assert_eq!(opts.max_output_tokens, Some(1024));
        assert_eq!(opts.preferred_context_tokens, Some(16384));
        assert_eq!(opts.keep_alive.as_deref(), Some("0"));
    }
}
