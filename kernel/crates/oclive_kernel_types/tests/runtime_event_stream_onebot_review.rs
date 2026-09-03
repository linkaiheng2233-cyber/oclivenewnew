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

fn live_evidence() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_live_evidence.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2.2 OneBot live evidence fixture")
}

fn timeout_evidence() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_timeout_evidence.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2.2 OneBot timeout evidence fixture")
}

fn private_store_evidence() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_private_store_evidence.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2.2 OneBot private-store evidence fixture")
}

fn crash_window_evidence() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_crash_window_evidence.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2.2 OneBot crash-window evidence fixture")
}

fn owner_lease_evidence() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_owner_lease_evidence.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2.2 OneBot owner-lease evidence fixture")
}

fn recovery_policy_evidence() -> Value {
    serde_json::from_str(include_str!(
        "fixtures/runtime_event_stream_stage_a2_onebot_recovery_policy_evidence.v1.json"
    ))
    .expect("parse Runtime Event Stream Stage A.2.2 OneBot recovery-policy evidence fixture")
}

fn assert_no_sensitive_runtime_values(value: &Value) {
    const FORBIDDEN_KEYS: &[&str] = &[
        "access_token",
        "endpoint",
        "target_id",
        "user_id",
        "group_id",
        "message",
        "message_body",
        "provider_message_id",
    ];

    match value {
        Value::Object(entries) => {
            for (key, child) in entries {
                assert!(
                    !FORBIDDEN_KEYS.contains(&key.as_str()),
                    "live evidence contains forbidden runtime key: {key}"
                );
                assert_no_sensitive_runtime_values(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_no_sensitive_runtime_values(item);
            }
        }
        Value::String(text) => {
            assert!(
                !text.starts_with("http://") && !text.starts_with("https://"),
                "live evidence contains a runtime URL"
            );
        }
        Value::Number(number) => {
            if let Some(number) = number.as_u64() {
                assert!(
                    number < 100_000,
                    "live evidence contains an identifier-like number"
                );
            }
        }
        Value::Null | Value::Bool(_) => {}
    }
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

#[test]
fn stage_a2_onebot_live_evidence_proves_only_send_and_recall() {
    let evidence = live_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(evidence["stage"], "a2_2_2_send_and_recall_probe_only");
    assert_eq!(evidence["synthetic"], false);
    assert_eq!(evidence["live_adapter_tested"], true);
    assert_eq!(evidence["production_runtime_enabled"], false);
    assert_eq!(evidence["production_ready"], false);
    assert_eq!(evidence["endpoint_scope"], "loopback");
    assert_eq!(evidence["target_kind"], "group");

    assert_eq!(evidence["preflight"]["outcome"], "accepted");
    assert_eq!(evidence["preflight"]["protocol_v11_confirmed"], true);
    assert_eq!(evidence["send"]["action"], "send_group_msg");
    assert_eq!(evidence["send"]["outcome"], "delivered");
    assert_eq!(evidence["send"]["http_status"], 200);
    assert_eq!(evidence["send"]["message_id_present"], true);
    assert_eq!(evidence["send"]["automatic_retries"], 0);
    assert_eq!(evidence["recall"]["action"], "delete_msg");
    assert_eq!(evidence["recall"]["outcome"], "acknowledged");
    assert_eq!(evidence["recall"]["http_status"], 200);

    for field in [
        "exported_access_token",
        "exported_endpoint",
        "exported_target_id",
        "exported_message_body",
        "exported_provider_message_id",
    ] {
        assert_eq!(evidence["privacy"][field], false, "privacy field {field}");
    }
    for field in [
        "adapter_private_store_tested",
        "timeout_after_possible_submission_tested",
        "multi_host_owner_lease_tested",
        "production_stream_connected",
    ] {
        assert_eq!(
            evidence["remaining_gaps"][field], false,
            "remaining gap {field}"
        );
    }

    assert_eq!(evidence["probe_success"], true);
    assert_no_sensitive_runtime_values(&evidence);
}

#[test]
fn stage_a2_onebot_timeout_evidence_proves_uncertainty_and_reconciliation_only() {
    let evidence = timeout_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(
        evidence["stage"],
        "a2_2_2_post_submission_timeout_reconciliation_probe_only"
    );
    assert_eq!(evidence["synthetic"], false);
    assert_eq!(evidence["live_adapter_tested"], true);
    assert_eq!(evidence["production_runtime_enabled"], false);
    assert_eq!(evidence["production_ready"], false);
    assert_eq!(evidence["endpoint_scope"], "loopback");
    assert_eq!(evidence["target_kind"], "group");

    assert_eq!(
        evidence["authorization"]["timeout_fault_confirmed_before_network"],
        true
    );
    assert_eq!(
        evidence["authorization"]["reconciliation_confirmed_before_network"],
        true
    );
    assert_eq!(
        evidence["injected_fault"]["kind"],
        "withhold_send_response_until_client_timeout"
    );
    assert_eq!(evidence["injected_fault"]["upstream_send_requests"], 1);
    assert_eq!(evidence["injected_fault"]["client_abort_observed"], true);

    assert_eq!(
        evidence["probe_observation"]["send_outcome"],
        "delivery_uncertain"
    );
    assert_eq!(
        evidence["probe_observation"]["send_http_status"],
        Value::Null
    );
    assert_eq!(evidence["probe_observation"]["automatic_retries"], 0);
    assert_eq!(evidence["probe_observation"]["probe_recall_attempts"], 0);
    assert_eq!(
        evidence["probe_observation"]["checkpoint_must_remain_blocked"],
        true
    );
    assert_eq!(
        evidence["upstream_observation"]["send_outcome"],
        "delivered"
    );
    assert_eq!(evidence["upstream_observation"]["send_http_status"], 200);
    assert_eq!(
        evidence["upstream_observation"]["provider_message_id_present"],
        true
    );

    assert_eq!(
        evidence["reconciliation"]["started_only_after_probe_uncertain"],
        true
    );
    assert_eq!(evidence["reconciliation"]["attempts"], 1);
    assert_eq!(evidence["reconciliation"]["outcome"], "acknowledged");
    assert_eq!(
        evidence["privacy"]["provider_locator_storage"],
        "process_memory_only"
    );
    assert_eq!(evidence["privacy"]["persisted_provider_locator"], false);
    assert_eq!(
        evidence["remaining_gaps"]["timeout_after_possible_submission_tested"],
        true
    );
    assert_eq!(
        evidence["remaining_gaps"]["explicit_reconciliation_tested"],
        true
    );
    assert_eq!(
        evidence["remaining_gaps"]["adapter_private_store_tested"],
        false
    );
    assert_eq!(
        evidence["remaining_gaps"]["multi_host_owner_lease_tested"],
        false
    );
    assert_eq!(
        evidence["remaining_gaps"]["production_stream_connected"],
        false
    );

    assert_eq!(evidence["probe_success"], true);
    assert_no_sensitive_runtime_values(&evidence);
}

#[test]
fn stage_a2_onebot_private_store_evidence_proves_encryption_and_restart_reconciliation_only() {
    let evidence = private_store_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(
        evidence["stage"],
        "a2_2_2_encrypted_private_store_restart_reconciliation_probe_only"
    );
    assert_eq!(evidence["synthetic"], false);
    assert_eq!(evidence["live_adapter_tested"], true);
    assert_eq!(evidence["production_runtime_enabled"], false);
    assert_eq!(evidence["production_ready"], false);
    assert_eq!(evidence["endpoint_scope"], "loopback");
    assert_eq!(evidence["target_kind"], "group");

    assert_eq!(evidence["send"]["outcome"], "delivered");
    assert_eq!(evidence["send"]["http_status"], 200);
    assert_eq!(evidence["send"]["automatic_retries"], 0);
    assert_eq!(
        evidence["send"]["provider_message_id_present_before_encryption"],
        true
    );

    let store = &evidence["private_store"];
    assert_eq!(store["format"], "encrypted_json_envelope");
    assert_eq!(store["algorithm"], "aes-256-gcm");
    assert_eq!(store["key_source"], "process_environment_only");
    assert_eq!(store["key_persisted_in_store"], false);
    assert_eq!(
        store["state_before_reconciliation"],
        "delivered_pending_reconciliation"
    );
    assert_eq!(
        store["state_after_reconciliation"],
        "recalled_locator_removed"
    );
    assert_eq!(
        store["sensitive_plaintext_absent_before_reconciliation"],
        true
    );
    assert_eq!(
        store["encrypted_locator_present_before_reconciliation"],
        true
    );
    assert_eq!(
        store["locator_present_after_acknowledged_reconciliation"],
        false
    );

    assert_eq!(evidence["process_boundary"]["distinct_invocations"], true);
    assert_ne!(
        evidence["process_boundary"]["prepare_instance_id"],
        evidence["process_boundary"]["reconcile_instance_id"]
    );
    assert_eq!(evidence["reconciliation"]["action"], "delete_msg");
    assert_eq!(evidence["reconciliation"]["attempts"], 1);
    assert_eq!(evidence["reconciliation"]["outcome"], "acknowledged");
    assert_eq!(evidence["reconciliation"]["http_status"], 200);

    for field in [
        "exported_store_key",
        "exported_access_token",
        "exported_endpoint",
        "exported_target_id",
        "exported_message_body",
        "exported_provider_message_id",
    ] {
        assert_eq!(evidence["privacy"][field], false, "privacy field {field}");
    }
    assert_eq!(
        evidence["remaining_gaps"]["encrypted_probe_private_store_tested"],
        true
    );
    assert_eq!(
        evidence["remaining_gaps"]["restart_reconciliation_tested"],
        true
    );
    for field in [
        "provider_accept_to_locator_persist_crash_window_closed",
        "multi_host_owner_lease_tested",
        "production_stream_connected",
    ] {
        assert_eq!(
            evidence["remaining_gaps"][field], false,
            "remaining gap {field}"
        );
    }

    assert_eq!(evidence["probe_success"], true);
    assert_no_sensitive_runtime_values(&evidence);
}

#[test]
fn stage_a2_onebot_crash_window_evidence_proves_only_napcat_group_recovery() {
    let evidence = crash_window_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(
        evidence["stage"],
        "a2_2_2_provider_accept_locator_persist_crash_recovery_probe_only"
    );
    assert_eq!(evidence["synthetic"], false);
    assert_eq!(evidence["live_adapter_tested"], true);
    assert_eq!(evidence["production_runtime_enabled"], false);
    assert_eq!(evidence["production_ready"], false);
    assert_eq!(evidence["endpoint_scope"], "loopback");
    assert_eq!(evidence["target_kind"], "group");

    let fault = &evidence["injected_fault"];
    assert_eq!(
        fault["kind"],
        "controlled_exit_after_send_ack_before_locator_persist"
    );
    assert_eq!(fault["authorized_before_network"], true);
    assert_eq!(fault["expected_exit_code"], 86);
    assert_eq!(fault["attempting_record_without_locator_observed"], true);
    assert_eq!(fault["automatic_send_retries"], 0);

    let history = &evidence["history_reconciliation"];
    assert_eq!(history["action"], "get_group_msg_history");
    assert_eq!(history["profile"], "napcat_go_cqhttp_extension");
    assert_eq!(history["read_only"], true);
    assert_eq!(history["recovery_authorized_before_network"], true);
    assert_eq!(history["outcome"], "unique_exact_own_match");
    assert_eq!(history["candidate_count"], 1);
    assert_eq!(history["exact_body_required"], true);
    assert_eq!(history["exact_target_required"], true);
    assert_eq!(history["own_account_marker_required"], true);
    assert_eq!(history["locator_persisted_before_recall"], true);

    assert_eq!(evidence["reconciliation"]["action"], "delete_msg");
    assert_eq!(evidence["reconciliation"]["attempts"], 1);
    assert_eq!(evidence["reconciliation"]["outcome"], "acknowledged");
    assert_eq!(evidence["reconciliation"]["http_status"], 200);

    let store = &evidence["private_store"];
    assert_eq!(store["format"], "encrypted_json_envelope");
    assert_eq!(store["algorithm"], "aes-256-gcm");
    assert_eq!(store["key_source"], "process_environment_only");
    assert_eq!(store["key_persisted_in_store"], false);
    assert_eq!(store["sensitive_plaintext_absent_before_recovery"], true);
    assert_eq!(store["locator_absent_at_recovery_start"], true);
    assert_eq!(store["locator_removed_after_acknowledged_recall"], true);

    for field in [
        "history_payload_exported",
        "exported_store_key",
        "exported_access_token",
        "exported_endpoint",
        "exported_target_id",
        "exported_message_body",
        "exported_provider_message_id",
    ] {
        assert_eq!(evidence["privacy"][field], false, "privacy field {field}");
    }
    assert_eq!(
        evidence["remaining_gaps"]["napcat_group_crash_window_recovery_tested"],
        true
    );
    for field in [
        "generic_onebot_crash_window_closed",
        "private_target_crash_window_closed",
        "host_level_key_recovery_tested",
        "multi_host_owner_lease_tested",
        "production_stream_connected",
    ] {
        assert_eq!(
            evidence["remaining_gaps"][field], false,
            "remaining gap {field}"
        );
    }

    assert_eq!(evidence["probe_success"], true);
    assert_no_sensitive_runtime_values(&evidence);
}

#[test]
fn stage_a2_onebot_owner_lease_evidence_proves_only_single_host_process_fencing() {
    let evidence = owner_lease_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(
        evidence["stage"],
        "a2_2_2_single_host_cross_process_owner_lease_probe_only"
    );
    assert_eq!(evidence["synthetic"], true);
    assert_eq!(evidence["live_adapter_tested"], false);
    assert_eq!(evidence["production_runtime_enabled"], false);
    assert_eq!(evidence["production_ready"], false);
    assert_eq!(evidence["endpoint_scope"], "loopback");
    assert_eq!(evidence["store_backend"], "node_sqlite_probe_only");

    let lease = &evidence["lease_contract"];
    for field in [
        "single_host_cross_process_claim_tested",
        "sqlite_begin_immediate_serializes_claims",
        "revision_cas_required",
        "lease_epoch_fencing_required",
        "expired_pre_attempt_takeover_increments_epoch",
        "stale_owner_blocked_before_effect",
    ] {
        assert_eq!(lease[field], true, "lease invariant {field}");
    }

    let delivery = &evidence["delivery_contract"];
    for field in [
        "attempting_persisted_before_provider_effect",
        "expired_attempting_blocks_automatic_failover",
        "stale_completion_rejected",
    ] {
        assert_eq!(delivery[field], true, "delivery invariant {field}");
    }
    assert_eq!(delivery["automatic_send_retries"], 0);
    assert_eq!(delivery["concurrent_provider_send_count"], 1);
    assert_eq!(delivery["pre_attempt_takeover_provider_send_count"], 1);
    assert_eq!(delivery["post_attempt_crash_provider_send_count"], 0);

    for field in [
        "exported_access_token",
        "exported_endpoint",
        "exported_target_id",
        "exported_message_body",
        "exported_provider_message_id",
        "sqlite_contains_delivery_payload_or_locator",
    ] {
        assert_eq!(evidence["privacy"][field], false, "privacy field {field}");
    }
    for field in [
        "multi_host_owner_lease_tested",
        "provider_side_fencing_available",
        "sigkill_or_power_loss_tested",
        "host_level_key_recovery_tested",
        "production_adapter_store_connected",
        "production_stream_connected",
    ] {
        assert_eq!(
            evidence["remaining_gaps"][field], false,
            "remaining gap {field}"
        );
    }

    assert_eq!(evidence["probe_success"], true);
    assert_no_sensitive_runtime_values(&evidence);
}

#[test]
fn stage_a2_onebot_recovery_policy_freezes_private_and_no_history_fail_closed() {
    let evidence = recovery_policy_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert_eq!(
        evidence["stage"],
        "a2_2_2_private_and_no_history_fail_closed_policy_probe_only"
    );
    assert_eq!(evidence["synthetic"], true);
    assert_eq!(evidence["live_adapter_tested"], false);
    assert_eq!(evidence["network_requests_performed"], 0);
    assert_eq!(evidence["production_runtime_enabled"], false);
    assert_eq!(evidence["production_ready"], false);
    assert_eq!(
        evidence["trigger_state"],
        "attempting_without_provider_locator"
    );

    let protocol = &evidence["protocol_contract"];
    assert_eq!(
        protocol["pinned_onebot_v11_commit"],
        PINNED_ONEBOT_V11_COMMIT
    );
    for field in [
        "onebot_v11_standard_history_lookup_available",
        "protocol_native_idempotency_key_available",
        "provider_side_fencing_available",
    ] {
        assert_eq!(protocol[field], false, "protocol boundary {field}");
    }

    let policy = &evidence["policy_contract"];
    assert_eq!(policy["default_decision"], "manual_reconciliation_required");
    assert_eq!(
        strings(&policy["accepted_lookup_profiles"]),
        ["napcat_go_cqhttp_extension"]
    );
    for field in [
        "lookup_profile_scope_must_match_target",
        "unique_exact_own_match_required",
        "read_only_lookup_requires_explicit_operator_action",
        "recovered_locator_must_be_encrypted_before_recall",
        "recall_requires_separate_explicit_reconciliation",
        "eligible_decision_does_not_authorize_recall",
        "original_send_must_never_be_retried_from_uncertain_state",
        "owner_failover_must_remain_blocked",
        "checkpoint_must_remain_blocked",
    ] {
        assert_eq!(policy[field], true, "policy invariant {field}");
    }

    let scenarios = evidence["scenarios"].as_array().expect("scenario array");
    assert_eq!(scenarios.len(), 9);
    let expected_ids = BTreeSet::from([
        "private_without_standard_history",
        "group_without_standard_history",
        "private_with_group_only_profile",
        "private_with_untrusted_extension",
        "group_history_unavailable",
        "group_history_not_yet_queried",
        "group_without_exact_match",
        "group_with_ambiguous_matches",
        "group_with_unique_exact_own_match",
    ]);
    let actual_ids = scenarios
        .iter()
        .map(|scenario| scenario["id"].as_str().expect("scenario id"))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_ids, expected_ids);

    let mut eligible = 0;
    for scenario in scenarios {
        let result = &scenario["result"];
        assert_eq!(result["automatic_failover_allowed"], false);
        assert_eq!(result["automatic_provider_effects_allowed"], false);
        assert_eq!(result["automatic_send_retries"], 0);
        assert_eq!(result["checkpoint_advances"], false);
        assert_eq!(result["emits_delivered"], false);
        assert_eq!(result["requires_explicit_operator_action"], true);

        if scenario["input"]["target_kind"] == "private" {
            assert_eq!(result["decision"], "manual_reconciliation_required");
            assert_eq!(result["locator_persistence_allowed"], false);
        }
        if result["decision"] == "eligible_for_explicit_locator_reconciliation" {
            eligible += 1;
            assert_eq!(scenario["id"], "group_with_unique_exact_own_match");
            assert_eq!(scenario["input"]["target_kind"], "group");
            assert_eq!(
                scenario["input"]["lookup_profile"],
                "napcat_go_cqhttp_extension"
            );
            assert_eq!(
                scenario["input"]["lookup_outcome"],
                "unique_exact_own_match"
            );
            assert_eq!(result["locator_persistence_allowed"], true);
        }
    }
    assert_eq!(eligible, 1);

    let result = &evidence["evidence_result"];
    assert_eq!(result["scenario_count"], 9);
    assert_eq!(result["private_target_fail_closed_policy_frozen"], true);
    assert_eq!(result["no_history_fail_closed_policy_frozen"], true);
    assert_eq!(result["eligible_scenario_count"], 1);
    assert_eq!(result["automatic_provider_request_count"], 0);

    for field in [
        "private_target_automatic_crash_recovery_available",
        "generic_onebot_automatic_crash_recovery_available",
        "host_level_key_recovery_tested",
        "multi_host_owner_lease_tested",
        "provider_side_fencing_available",
        "production_adapter_store_connected",
        "production_stream_connected",
    ] {
        assert_eq!(
            evidence["remaining_gaps"][field], false,
            "remaining gap {field}"
        );
    }

    assert_eq!(evidence["probe_success"], true);
    assert_no_sensitive_runtime_values(&evidence);
}
