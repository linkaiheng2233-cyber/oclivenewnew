//! CP-INT B6：浏览器 SSE 传输 ↔ Tauri 恢复的**构造合同**（纯内存 / 无监听器）。
//!
//! 本文件证明三件在无 TCP 下仍可判定的合同：
//!   1. `POST /chat/stream` 与 `POST /chat/recover` 的请求体由**同一个** `client_request_id`
//!      与同一组语义字段构造，且分别能被 Host 的真实接收类型反序列化（前端体还逐字冻结）；
//!   2. 恢复响应的映射只可能落到 `CHAT_REQUEST_CONFLICT` / `CHAT_REQUEST_UNCONFIRMED`，
//!      404 / 500 / 401 / 503 / 坏 JSON 都不能变成"新发送"；
//!   3. 端点不可达时**真实**恢复函数返回 `CHAT_REQUEST_UNCONFIRMED`，而同一端点上的发送
//!      路径返回的是另一种错误（`KERNEL_OFFLINE`），因此不存在到 `/chat` 的回退。
//!
//! 边界：不启动监听器、不访问外部网络、不调用真实模型、不写任何 run tree。唯一的 socket
//! 行为是向**无监听器**的环回端口发起一次连接并被系统立即拒绝（见 §3）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use oclive_kernel_host::http_api::ChatApiRequest;
use oclive_kernel_types::error::AppError;
use oclive_kernel_types::models::dto::SendMessageRequest;
use oclivenewnew_tauri::kernel_attach::KernelHttpClient;
use oclivenewnew_tauri::kernel_lifecycle::KernelConnection;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

/// 一次逻辑发送的 turn 身份；流式传输与恢复必须复用它。
const TURN_ID: &str = "ae45c9f0-5a98-4ac9-a103-112c1caa1526";
const USER_TEXT: &str = "聊聊清晨的风景吧。";
const ROLE_PACK_PATH: &str = "D:/synthetic/roles/mumu";
const RECOVERED_REPLY: &str = "这就是同一次请求的最终结果。";

/// 前端 `sendMessageStream` 实际发出的请求体（字段集合与
/// `distros/shared/src/integration/crossTransportRecovery.test.ts` 中的逐字断言对应）。
/// 这里冻结同一份 wire 契约，用于跨语言字段名回归。
const FRONTEND_STREAM_BODY: &str = concat!(
    r#"{"client_request_id":"ae45c9f0-5a98-4ac9-a103-112c1caa1526","#,
    r#""role_path":"D:/synthetic/roles/mumu","message":"聊聊清晨的风景吧。","#,
    r#""scene_id":"home","session_id":null,"adult":null}"#
);

/// 前端 `recoverMessage` 交给 Tauri `recover_message` 的 `req`（未设置 session；`adult` 为
/// `undefined`，经 IPC JSON 序列化后键被丢弃 —— 因此 Rust 侧看到的就是这四个键）。
const FRONTEND_RECOVER_REQUEST: &str = concat!(
    r#"{"role_id":"mumu","user_message":"聊聊清晨的风景吧。","#,
    r#""scene_id":"home","client_request_id":"ae45c9f0-5a98-4ac9-a103-112c1caa1526"}"#
);

fn sample_request() -> SendMessageRequest {
    SendMessageRequest {
        client_request_id: Some(TURN_ID.to_string()),
        role_id: "mumu".to_string(),
        user_message: USER_TEXT.to_string(),
        scene_id: Some("home".to_string()),
        session_id: None,
        include_raw_reply: None,
        adult: None,
    }
}

/// Host 侧权威 `SendMessageResponse` 的最小完整 JSON（`/chat`、`/chat/stream`、`/chat/recover`
/// 三种传输共用同一 DTO）。
fn sample_response_json() -> Value {
    json!({
        "api_version": 1,
        "schema": 16,
        "presence_mode": "co_present",
        "relation_state": "Friend",
        "reply": RECOVERED_REPLY,
        "emotion": {
            "joy": 0.0, "sadness": 0.0, "anger": 0.0, "fear": 0.0,
            "surprise": 0.0, "disgust": 0.0, "neutral": 1.0
        },
        "bot_emotion": "neutral",
        "portrait_emotion": "neutral",
        "favorability_delta": 0.0,
        "favorability_current": 50.0,
        "events": [],
        "scene_id": "home",
        "offer_destination_picker": false,
        "offer_together_travel": false,
        "reply_is_fallback": false,
        "knowledge_chunks_in_prompt": 0,
        "timestamp": 1,
        "user_message_id": "user-recovered",
        "assistant_message_id": "assistant-recovered",
        "user_message_timestamp": "2026-09-25T00:00:00.000Z",
        "assistant_message_timestamp": "2026-09-25T00:00:01.000Z",
    })
}

/// §1 身份与语义：两个传输共用同一个 turn，却各自使用 Host 认可的不同字段名。
#[test]
fn stream_and_recover_bodies_share_one_turn_identity() {
    let req = sample_request();
    let stream = KernelHttpClient::chat_api_body(Path::new(ROLE_PACK_PATH), &req);
    let recover = KernelHttpClient::chat_recover_body(&req).expect("recover body serializes");

    // ① 身份：自动恢复复用的是流式请求已经用过的同一个规范 UUID。
    assert_eq!(stream["client_request_id"], json!(TURN_ID));
    assert_eq!(recover["client_request_id"], json!(TURN_ID));
    assert_eq!(stream["client_request_id"], recover["client_request_id"]);

    // ② 语义字段逐一对应：字段名不同（`message`/`role_path` ↔ `user_message`/`role_id`），值必须相同。
    assert_eq!(stream["message"], json!(USER_TEXT));
    assert_eq!(stream["message"], recover["user_message"]);
    assert_eq!(stream["role_path"], json!(ROLE_PACK_PATH));
    assert_eq!(recover["role_id"], json!("mumu"));
    assert_eq!(stream["scene_id"], recover["scene_id"]);
    assert_eq!(stream["session_id"], recover["session_id"]);
    assert_eq!(stream["include_raw_reply"], recover["include_raw_reply"]);
    assert_eq!(stream["adult"], recover["adult"]);

    // ③ 两种传输的形状确实不同：任一请求体都不能拿去撞另一种路由。
    for key in ["role_id", "user_message"] {
        assert!(
            stream.get(key).is_none(),
            "{key} must not travel in the /chat(/stream) body"
        );
    }
    for key in ["role_path", "message"] {
        assert!(
            recover.get(key).is_none(),
            "{key} must not travel in the /chat/recover body"
        );
    }
    assert_ne!(stream, recover);

    // ④ Host 的真实接收类型接受这两种体，且反序列化后身份与语义不变。
    let api: ChatApiRequest = serde_json::from_value(stream).expect("ChatApiRequest");
    assert_eq!(api.client_request_id.as_deref(), Some(TURN_ID));
    assert_eq!(api.role_path, ROLE_PACK_PATH);
    assert_eq!(api.message, USER_TEXT);
    assert_eq!(api.scene_id.as_deref(), Some("home"));
    assert_eq!(api.session_id, None);
    assert!(api.adult.is_none());

    let recovered: SendMessageRequest =
        serde_json::from_value(recover.clone()).expect("recover body is a bare SendMessageRequest");
    assert_eq!(recovered.client_request_id.as_deref(), Some(TURN_ID));
    assert_eq!(recovered.role_id, "mumu");
    assert_eq!(recovered.user_message, USER_TEXT);
    assert_eq!(recovered.scene_id.as_deref(), Some("home"));
    assert_eq!(recovered.session_id, None);
    assert!(recovered.adult.is_none());
    // 往返无字段损失：桥发出的恢复体与重新序列化逐字段一致。
    assert_eq!(serde_json::to_value(&recovered).unwrap(), recover);
}

/// §2 跨语言字段名 + 缺省键归一：前端体进入 Host 接收类型后，两个传输的指纹输入相同。
#[test]
fn the_frontend_wire_bodies_normalize_to_the_same_turn_identity_inputs() {
    let front_stream: Value =
        serde_json::from_str(FRONTEND_STREAM_BODY).expect("frontend body json");
    let keys: BTreeSet<&str> = front_stream
        .as_object()
        .expect("frontend body is an object")
        .keys()
        .map(String::as_str)
        .collect();
    // 浏览器 SSE 体不含 `include_raw_reply`（缺省键 → None）；键集合在此冻结。
    assert_eq!(
        keys,
        BTreeSet::from([
            "adult",
            "client_request_id",
            "message",
            "role_path",
            "scene_id",
            "session_id"
        ])
    );
    let api: ChatApiRequest =
        serde_json::from_value(front_stream.clone()).expect("frontend body is a ChatApiRequest");
    assert_eq!(api.client_request_id.as_deref(), Some(TURN_ID));
    assert_eq!(api.role_path, ROLE_PACK_PATH);
    assert_eq!(api.message, USER_TEXT);
    assert_eq!(api.scene_id.as_deref(), Some("home"));
    assert_eq!(api.session_id, None, "explicit null normalizes to None");
    assert_eq!(api.include_raw_reply, None, "absent key normalizes to None");
    assert!(api.adult.is_none());

    // 桥构造的同路由请求体语义相同（该函数目前无生产调用点，仍是同一 transport 对的一半）。
    let bridge_stream =
        KernelHttpClient::chat_api_body(Path::new(ROLE_PACK_PATH), &sample_request());
    let normalize = |body: &Value| {
        json!({
            "client_request_id": body["client_request_id"],
            "role_path": body["role_path"],
            "message": body["message"],
            "scene_id": body["scene_id"],
            "session_id": body["session_id"],
            "adult": body["adult"],
        })
    };
    assert_eq!(normalize(&bridge_stream), normalize(&front_stream));

    // 恢复路径：前端 IPC 参数与桥实际发出的恢复体是同一个结构（键缺省 == 显式 null）。
    let front_recover: SendMessageRequest =
        serde_json::from_str(FRONTEND_RECOVER_REQUEST).expect("frontend recover arg");
    assert_eq!(front_recover.client_request_id.as_deref(), Some(TURN_ID));
    assert_eq!(front_recover.role_id, "mumu");
    assert_eq!(front_recover.user_message, USER_TEXT);
    assert_eq!(front_recover.scene_id.as_deref(), Some("home"));
    assert_eq!(front_recover.session_id, None);
    let bridge_recover = KernelHttpClient::chat_recover_body(&front_recover).expect("bridge body");
    assert_eq!(
        bridge_recover,
        serde_json::to_value(&front_recover).unwrap()
    );
    assert_eq!(
        bridge_recover,
        KernelHttpClient::chat_recover_body(&sample_request()).unwrap()
    );
    // Host 的回合指纹 = 去掉 `client_request_id` 后的规范化 JSON 哈希，scope = (role_id, session_id)。
    // 两个传输进入 Host 的结构逐字段相同 ⇒ 指纹与 scope 必然相同（此处不重算哈希，只证输入相同）。
    let identity_input = |req: &SendMessageRequest| {
        let mut value = serde_json::to_value(req).unwrap();
        value.as_object_mut().unwrap().remove("client_request_id");
        value
    };
    assert_eq!(
        identity_input(&front_recover),
        identity_input(&sample_request())
    );
    assert_eq!(
        (
            front_recover.role_id.as_str(),
            front_recover.session_id.as_deref()
        ),
        (
            sample_request().role_id.as_str(),
            sample_request().session_id.as_deref()
        )
    );
}

/// §3 错误映射：恢复失败永远只能是"可确认性"边界，不是新回合。
#[test]
fn recover_response_mapping_never_produces_a_new_turn() {
    let ok = KernelHttpClient::map_recover_response(200, &sample_response_json().to_string())
        .expect("200 with a full DTO decodes");
    assert_eq!(ok.reply, RECOVERED_REPLY);
    assert_eq!(
        ok.assistant_message_id.as_deref(),
        Some("assistant-recovered")
    );

    // 复用同一身份但载荷不同：保持可识别的冲突边界（前端据此停止重试）。
    let conflict =
        json!({"error": {"code": "CHAT_REQUEST_CONFLICT", "message": "reused identity"}});
    assert!(matches!(
        KernelHttpClient::map_recover_response(409, &conflict.to_string()),
        Err(AppError::ChatRequestConflict)
    ));

    let unconfirmed =
        json!({"error": {"code": "CHAT_REQUEST_UNCONFIRMED", "message": "no receipt"}});
    let cases: [(&str, u16, String); 7] = [
        ("409 unconfirmed", 409, unconfirmed.to_string()),
        (
            "409 other kernel error",
            409,
            json!({"error": {"code": "DB_ERROR", "message": "boom"}}).to_string(),
        ),
        (
            "404 old kernel without the route",
            404,
            "<html>not found</html>".to_string(),
        ),
        ("401 stale api token", 401, "unauthorized".to_string()),
        ("503 kernel offline", 503, String::new()),
        (
            "500 host failure",
            500,
            json!({"error": {"code": "LLM_ERROR", "message": "boom"}}).to_string(),
        ),
        ("200 with a broken payload", 200, "{broken".to_string()),
    ];
    for (label, status, body) in cases {
        match KernelHttpClient::map_recover_response(status, &body) {
            Err(AppError::ChatRequestUnconfirmed) => {}
            other => panic!("{label} must stay unconfirmed, got {other:?}"),
        }
    }
}

/// §4 真实函数 + 无监听器端点：失败留在恢复路径，不会被换成一次新发送。
#[tokio::test]
async fn an_unreachable_endpoint_stays_unconfirmed_and_never_becomes_a_send() {
    // 唯一的 socket 行为：向环回端口发起连接，无监听器 → 系统立即拒绝。
    let conn = KernelConnection::new("http://127.0.0.1:1", 1);
    let req = sample_request();

    let recovered = KernelHttpClient::recover_message_via_http(&conn, &req)
        .await
        .expect_err("recovery against a dead endpoint must not resolve");
    assert!(
        matches!(recovered, AppError::ChatRequestUnconfirmed),
        "unexpected recovery error: {recovered:?}"
    );

    // 负控（非空洞性）：同一端点上的发送路径先过健康门，给出的是另一种错误。
    let sent = KernelHttpClient::send_message_via_http(&conn, Path::new(ROLE_PACK_PATH), &req)
        .await
        .expect_err("send against a dead endpoint must not resolve");
    assert!(
        matches!(sent, AppError::KernelOffline),
        "unexpected send error: {sent:?}"
    );
    assert_ne!(sent.code(), "CHAT_REQUEST_UNCONFIRMED");
    assert_eq!(recovered.code(), "CHAT_REQUEST_UNCONFIRMED");
}
