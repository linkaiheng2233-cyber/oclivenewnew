use std::collections::BTreeSet;

use oclive_kernel_types::RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION;
use serde::Deserialize;
use serde_json::Value;

const PINNED_ONEBOT_V11_COMMIT: &str = "d4456ee706f9ada9c2dfde56a2bcfc69752600e4";

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RequestProgress {
    NotSubmitted,
    BodyMayHaveBeenAccepted,
    ResponseReceived,
}

#[derive(Debug, Deserialize)]
struct ActionResponse {
    http_status: u16,
    well_formed_json: bool,
    status: Option<String>,
    retcode: Option<i64>,
    message_id: Option<i64>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum DeliveryOutcome {
    Delivered,
    RetryableNotSubmitted,
    Rejected,
    DeliveryUncertain,
}

#[derive(Debug, Deserialize)]
struct DeliveryCase {
    id: String,
    request_progress: RequestProgress,
    response: Option<ActionResponse>,
    expected_outcome: DeliveryOutcome,
    automatic_retry_allowed: bool,
    checkpoint_advances: bool,
    emits_delivered: bool,
    requires_reconciliation: bool,
    checkpoint_blocks: bool,
}

fn contract() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_review.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2 OneBot review fixture")
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .expect("expected string array")
        .iter()
        .map(|item| item.as_str().expect("expected string item"))
        .collect()
}

fn assert_contains_all(value: &Value, expected: &[&str]) {
    let actual = strings(value);
    for item in expected {
        assert!(actual.contains(item), "missing {item}");
    }
}

fn classify_delivery(case: &DeliveryCase) -> DeliveryOutcome {
    match case.request_progress {
        RequestProgress::NotSubmitted => DeliveryOutcome::RetryableNotSubmitted,
        RequestProgress::BodyMayHaveBeenAccepted => DeliveryOutcome::DeliveryUncertain,
        RequestProgress::ResponseReceived => {
            let Some(response) = case.response.as_ref() else {
                return DeliveryOutcome::DeliveryUncertain;
            };

            if matches!(response.http_status, 400 | 401 | 403 | 404 | 406) {
                return DeliveryOutcome::Rejected;
            }
            if response.http_status != 200 || !response.well_formed_json {
                return DeliveryOutcome::DeliveryUncertain;
            }
            if response.status.as_deref() == Some("ok")
                && response.retcode == Some(0)
                && response.message_id.is_some()
            {
                return DeliveryOutcome::Delivered;
            }
            if response.status.as_deref() == Some("failed")
                && !matches!(response.retcode, None | Some(0 | 1))
            {
                return DeliveryOutcome::Rejected;
            }

            DeliveryOutcome::DeliveryUncertain
        }
    }
}

#[test]
fn stage_a2_onebot_review_freezes_ack_and_retry_boundaries() {
    let contract = contract();
    assert_eq!(
        contract["schema_version"].as_u64(),
        Some(u64::from(RUNTIME_EVENT_STREAM_CONTRACT_SCHEMA_VERSION))
    );
    assert_eq!(contract["stage"], "a2_2_1_onebot_v11_protocol_review_only");
    assert_eq!(contract["production_runtime_enabled"], false);
    assert_eq!(contract["live_adapter_tested"], false);

    let source = &contract["protocol_source"];
    assert_eq!(
        source["repository"],
        "https://github.com/botuniverse/onebot-11"
    );
    assert_eq!(source["commit"], PINNED_ONEBOT_V11_COMMIT);
    let documents = strings(&source["documents"]);
    assert_eq!(documents.len(), 5);
    for document in documents {
        assert!(document.contains(PINNED_ONEBOT_V11_COMMIT));
        assert!(!document.contains("/master/"));
    }

    let profile = &contract["adapter_profile"];
    assert_eq!(profile["adapter_id"], "reference.qq.onebot_v11_http");
    assert_eq!(profile["channel"], "qq_text");
    assert_eq!(profile["transport"], "http_json_post");
    assert_eq!(profile["send_method"], "POST");
    assert_eq!(profile["content_type"], "application/json");
    assert_eq!(
        strings(&profile["send_actions"]),
        ["send_private_msg", "send_group_msg"]
    );
    assert_eq!(profile["recall_action"], "delete_msg");
    assert_eq!(profile["plain_text_auto_escape_default"], true);
    assert_eq!(
        strings(&profile["forbidden_send_suffixes"]),
        ["_async", "_rate_limited"]
    );
    assert_eq!(profile["protocol_native_idempotency_key"], false);
    assert_eq!(profile["websocket_echo_is_correlation_only"], true);

    let delivery = &contract["delivery_contract"];
    assert_contains_all(
        &delivery["states"],
        &[
            "prepared",
            "attempting",
            "delivered",
            "rejected",
            "delivery_uncertain",
        ],
    );
    assert_eq!(
        strings(&delivery["delivered_requires"]),
        [
            "http_status_200",
            "status_ok",
            "retcode_0",
            "data_message_id_present"
        ]
    );
    assert_eq!(
        delivery["automatic_retry_only_when"],
        "request_proven_not_submitted"
    );
    assert_eq!(
        delivery["timeout_after_possible_submission"],
        "delivery_uncertain"
    );
    assert_eq!(delivery["async_response"], "delivery_uncertain");
    assert_eq!(
        delivery["malformed_or_incomplete_success"],
        "delivery_uncertain"
    );
    assert_eq!(delivery["checkpoint_advances_only_on"], "delivered");
    assert_eq!(delivery["delivery_uncertain_blocks_failover"], true);

    let cases: Vec<DeliveryCase> =
        serde_json::from_value(contract["cases"].clone()).expect("parse delivery cases");
    let mut ids = BTreeSet::new();
    for case in &cases {
        assert!(ids.insert(case.id.as_str()), "duplicate case: {}", case.id);
        let outcome = classify_delivery(case);
        assert_eq!(outcome, case.expected_outcome, "{} outcome", case.id);
        assert_eq!(
            case.automatic_retry_allowed,
            outcome == DeliveryOutcome::RetryableNotSubmitted,
            "{} automatic retry",
            case.id
        );
        assert_eq!(
            case.checkpoint_advances,
            outcome == DeliveryOutcome::Delivered,
            "{} checkpoint",
            case.id
        );
        assert_eq!(
            case.emits_delivered,
            outcome == DeliveryOutcome::Delivered,
            "{} delivered",
            case.id
        );
        assert_eq!(
            case.requires_reconciliation,
            outcome == DeliveryOutcome::DeliveryUncertain,
            "{} reconciliation",
            case.id
        );
        assert_eq!(
            case.checkpoint_blocks,
            matches!(
                outcome,
                DeliveryOutcome::Rejected | DeliveryOutcome::DeliveryUncertain
            ),
            "{} checkpoint block",
            case.id
        );
    }
    assert_eq!(ids.len(), 9);
}

#[test]
fn stage_a2_onebot_review_keeps_governance_gaps_explicit() {
    let contract = contract();
    let privacy = &contract["receipt_privacy_contract"];
    assert_contains_all(
        &privacy["stream_safe_receipt_fields"],
        &[
            "adapter_id",
            "receipt_ref",
            "outcome",
            "attempt",
            "acknowledged_at",
        ],
    );
    assert_contains_all(
        &privacy["adapter_private_encrypted_fields"],
        &[
            "destination_id",
            "message_body_while_pending",
            "provider_message_id",
        ],
    );
    assert_contains_all(
        &privacy["forbidden_stream_receipt_fields"],
        &[
            "access_token",
            "user_id",
            "group_id",
            "message",
            "provider_message_id",
        ],
    );
    assert_eq!(
        privacy["provider_locator_deleted_after_recall_window_or_session_erasure"],
        true
    );

    let deletion = &contract["deletion_contract"];
    assert_eq!(deletion["recall_action"], "delete_msg");
    assert_eq!(
        strings(&deletion["recall_ack_requires"]),
        ["http_status_200", "status_ok", "retcode_0"]
    );
    assert_eq!(
        deletion["recall_is_provider_message_recall_not_privacy_erasure"],
        true
    );
    assert_eq!(
        deletion["external_recall_failure_blocks_local_erasure_completion"],
        false
    );
    assert_eq!(
        deletion["completion_must_report_external_recall_as_unverified"],
        true
    );
    assert_eq!(
        strings(&deletion["required_local_order"]).first().copied(),
        Some("block_new_deliveries_and_leases")
    );
    assert_eq!(
        strings(&deletion["required_local_order"]).last().copied(),
        Some("report_only_aggregate_external_recall_status")
    );

    let multi_host = &contract["multi_host_contract"];
    assert_eq!(multi_host["single_delivery_owner_lease_required"], true);
    assert_eq!(
        multi_host["fencing_token_required_on_claim_and_receipt"],
        true
    );
    assert_eq!(multi_host["onebot_standard_fencing"], false);
    assert_eq!(
        multi_host["automatic_failover_after_delivery_uncertain"],
        false
    );
    assert_eq!(multi_host["local_sqlite_multi_host_ready"], false);
    assert_eq!(
        multi_host["external_broker_trigger"],
        "same_session_active_on_multiple_hosts"
    );

    let security = &contract["security_contract"];
    assert_eq!(
        security["authorization_transport"],
        "bearer_access_token_header"
    );
    for field in [
        "credentials_outside_event_stream_trace_and_logs",
        "loopback_default",
        "remote_endpoint_requires_explicit_network_grant_and_tls",
        "get_with_message_query_forbidden",
        "plain_text_defaults_to_auto_escape",
    ] {
        assert_eq!(security[field], true, "security boundary {field}");
    }

    assert_eq!(contract["review_result"]["protocol_mapping_feasible"], true);
    assert_eq!(contract["review_result"]["production_ready"], false);
    assert_eq!(
        contract["review_result"]["next_stage"],
        "a2_2_2_live_adapter_evidence"
    );
    assert_contains_all(
        &contract["known_gaps"],
        &[
            "no_onebot_runtime_or_qq_account_started",
            "no_provider_delivery_or_recall_executed",
            "no_onebot_standard_idempotency_or_fencing",
            "no_production_adapter_store_or_owner_lease",
            "no_multi_host_delivery_proof",
            "no_production_stream_runtime",
        ],
    );
}
