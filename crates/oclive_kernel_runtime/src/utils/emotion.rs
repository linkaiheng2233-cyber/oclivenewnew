use crate::utils::ollama::ollama_generate;
use tokio::time::{timeout, Duration};

/// 通过 Ollama AI 分析用户输入的情绪
/// 返回词汇：happy, sad, angry, shy, confused, neutral
/// 失败或超时时返回 neutral 作为默认情绪
pub async fn analyze_emotion(message: &str) -> Result<String, String> {
    let prompt = format!(
        r#"分析下面这句话的情绪，只输出一个词：happy, sad, angry, shy, confused, neutral

用户说："{}"

只输出一个词，不要其他文字。"#,
        message
    );

    match timeout(
        Duration::from_secs(8),
        ollama_generate("qwen2.5:7b", &prompt, 10, 0.1),
    )
    .await
    {
        Ok(Ok(emotion)) => {
            let e = emotion.trim().to_lowercase();
            match e.as_str() {
                "happy" | "sad" | "angry" | "shy" | "confused" => Ok(e),
                _ => Ok("neutral".to_string()),
            }
        }
        Ok(Err(e)) => {
            log::warn!("[情绪分析] AI 调用失败: {}, 降级为 neutral", e);
            Ok("neutral".to_string())
        }
        Err(_) => {
            log::warn!("[情绪分析] 调用超时，降级为 neutral");
            Ok("neutral".to_string())
        }
    }
}
