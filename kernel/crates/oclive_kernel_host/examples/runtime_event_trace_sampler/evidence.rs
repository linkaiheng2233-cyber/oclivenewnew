use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use anyhow::{bail, Context};
use chrono::{SecondsFormat, Utc};
use oclive_kernel_host::infrastructure::sqlite_pool;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::Row;

use super::contract::{ExpectedEvent, SampleContract, AFTER_RESTART, BEFORE_RESTART};

const DATABASE_FILE_NAME: &str = "runtime-event-trace-shadow.samples.sqlite3";
const EVIDENCE_FILE_NAME: &str = "runtime-event-trace-shadow.evidence.json";
const SUMMARY_FILE_NAME: &str = "runtime-event-trace-shadow.summary.md";

#[derive(Debug)]
struct PersistedEvent {
    position: u64,
    event_id: String,
    kind: String,
    source: String,
    source_weight_bps: u16,
    correlation_id: String,
    causation_id: Option<String>,
    ring_sequence: u64,
    depth: u16,
}

#[derive(Serialize)]
struct SampleEvidence {
    schema_version: u16,
    evidence_kind: String,
    authoritative_runtime_stream: bool,
    behavior_driver: bool,
    synthetic_only: bool,
    generated_at: String,
    source_commit: String,
    source_worktree_dirty: bool,
    scenario_contract: String,
    scenario_contract_sha256: String,
    raw_trace_database: String,
    summary: EvidenceSummary,
    observations: EvidenceObservations,
    limitations: Vec<String>,
    scenarios: Vec<ScenarioEvidence>,
}

#[derive(Serialize)]
struct EvidenceSummary {
    scenarios: usize,
    records: usize,
    root_records: usize,
    derived_records: usize,
    distinct_event_kinds: usize,
    kernel_starts: u16,
}

#[derive(Serialize)]
struct EvidenceObservations {
    positions_contiguous: bool,
    causation_resolved: bool,
    position_continues_after_restart: bool,
    ring_sequence_reset_after_restart: bool,
    privacy_minimized_schema: bool,
}

#[derive(Serialize)]
struct ScenarioEvidence {
    id: String,
    phase: String,
    action: String,
    records: Vec<EventEvidence>,
}

#[derive(Serialize)]
struct EventEvidence {
    index: usize,
    position: u64,
    kind: String,
    source: String,
    source_weight_bps: u16,
    causation_index: Option<usize>,
    ring_sequence: u64,
    depth: u16,
}

pub(crate) struct EvidenceCounts {
    pub(crate) scenarios: usize,
    pub(crate) records: usize,
}

pub(crate) async fn collect_and_write(
    contract: &SampleContract,
    scenario_contract: String,
    contract_text: &str,
    trace_path: &Path,
    output_dir: &Path,
    source_commit: String,
    source_worktree_dirty: bool,
) -> anyhow::Result<EvidenceCounts> {
    let (persisted, schema_columns) = read_trace_samples(trace_path).await?;
    let scenarios = normalize_and_validate(contract, &persisted)?;
    let observations = validate_observations(contract, &persisted, &schema_columns)?;

    let root_records = persisted
        .iter()
        .filter(|event| event.causation_id.is_none())
        .count();
    let derived_records = persisted.len().saturating_sub(root_records);
    let distinct_event_kinds = persisted
        .iter()
        .map(|event| event.kind.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let summary = EvidenceSummary {
        scenarios: scenarios.len(),
        records: persisted.len(),
        root_records,
        derived_records,
        distinct_event_kinds,
        kernel_starts: 2,
    };

    std::fs::create_dir_all(output_dir).context("create trace sample evidence directory")?;
    let database_path = output_dir.join(DATABASE_FILE_NAME);
    std::fs::copy(trace_path, &database_path).with_context(|| {
        format!(
            "copy trace sample database from {} to {}",
            trace_path.display(),
            database_path.display()
        )
    })?;

    let contract_hash = format!("{:x}", Sha256::digest(contract_text.as_bytes()));
    let evidence = SampleEvidence {
        schema_version: 1,
        evidence_kind: contract.evidence_kind.clone(),
        authoritative_runtime_stream: false,
        behavior_driver: false,
        synthetic_only: true,
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        source_commit,
        source_worktree_dirty,
        scenario_contract,
        scenario_contract_sha256: contract_hash,
        raw_trace_database: DATABASE_FILE_NAME.into(),
        summary,
        observations,
        limitations: vec![
            "Synthetic Event Ring scenarios only; no user, role-memory, Prompt, or model content is collected."
                .into(),
            "Raw SQLite is local design evidence, not an authoritative Runtime Event Stream or recovery log."
                .into(),
            "No consumer, checkpoint, replay, retention, output-delivery, or state-commit behavior is implemented."
                .into(),
            "Timing and throughput require separate long-run, production-like observation.".into(),
        ],
        scenarios,
    };

    let mut evidence_json =
        serde_json::to_string_pretty(&evidence).context("serialize trace sample evidence")?;
    evidence_json.push('\n');
    std::fs::write(output_dir.join(EVIDENCE_FILE_NAME), evidence_json)
        .context("write trace sample evidence")?;
    std::fs::write(
        output_dir.join(SUMMARY_FILE_NAME),
        render_markdown_summary(&evidence),
    )
    .context("write trace sample summary")?;

    Ok(EvidenceCounts {
        scenarios: evidence.summary.scenarios,
        records: evidence.summary.records,
    })
}

async fn read_trace_samples(
    trace_path: &Path,
) -> anyhow::Result<(Vec<PersistedEvent>, Vec<String>)> {
    let pool = sqlite_pool::connect_file(trace_path)
        .await
        .with_context(|| format!("open trace sample database {}", trace_path.display()))?;
    let rows = sqlx::query(
        "SELECT position, event_id, event_kind, source, source_weight_bps, correlation_id, \
         causation_id, ring_sequence, event_depth \
         FROM runtime_event_trace_records ORDER BY position",
    )
    .fetch_all(&pool)
    .await
    .context("read trace sample records")?;
    let mut persisted = Vec::with_capacity(rows.len());
    for row in rows {
        persisted.push(PersistedEvent {
            position: checked_u64(row.try_get("position")?, "position")?,
            event_id: row.try_get("event_id")?,
            kind: row.try_get("event_kind")?,
            source: row.try_get("source")?,
            source_weight_bps: checked_u16(row.try_get("source_weight_bps")?, "source_weight_bps")?,
            correlation_id: row.try_get("correlation_id")?,
            causation_id: row.try_get("causation_id")?,
            ring_sequence: checked_u64(row.try_get("ring_sequence")?, "ring_sequence")?,
            depth: checked_u16(row.try_get("event_depth")?, "event_depth")?,
        });
    }
    let schema_columns = sqlx::query("PRAGMA table_info(runtime_event_trace_records)")
        .fetch_all(&pool)
        .await
        .context("inspect trace sample schema")?
        .into_iter()
        .map(|row| row.get::<String, _>("name"))
        .collect();
    pool.close().await;
    Ok((persisted, schema_columns))
}

fn checked_u64(value: i64, field: &str) -> anyhow::Result<u64> {
    u64::try_from(value).with_context(|| format!("trace {field} is negative"))
}

fn checked_u16(value: i64, field: &str) -> anyhow::Result<u16> {
    u16::try_from(value).with_context(|| format!("trace {field} is outside u16"))
}

fn normalize_and_validate(
    contract: &SampleContract,
    persisted: &[PersistedEvent],
) -> anyhow::Result<Vec<ScenarioEvidence>> {
    let mut by_scenario = HashMap::<&str, Vec<&PersistedEvent>>::new();
    for event in persisted {
        by_scenario
            .entry(event.correlation_id.as_str())
            .or_default()
            .push(event);
    }

    let mut scenarios = Vec::with_capacity(contract.scenarios.len());
    for scenario in &contract.scenarios {
        let records = by_scenario
            .remove(scenario.id.as_str())
            .with_context(|| format!("missing trace sample scenario {}", scenario.id))?;
        if records.len() != scenario.expected_events.len() {
            bail!(
                "trace sample scenario {} expected {} records, observed {}",
                scenario.id,
                scenario.expected_events.len(),
                records.len()
            );
        }
        let local_indexes = records
            .iter()
            .enumerate()
            .map(|(index, event)| (event.event_id.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut normalized = Vec::with_capacity(records.len());
        let mut observed_contract = Vec::with_capacity(records.len());
        for (index, event) in records.iter().enumerate() {
            let causation_index = event
                .causation_id
                .as_deref()
                .map(|causation_id| {
                    local_indexes.get(causation_id).copied().with_context(|| {
                        format!(
                            "{} record {index} has causation outside its synthetic scenario",
                            scenario.id
                        )
                    })
                })
                .transpose()?;
            observed_contract.push(ExpectedEvent {
                kind: event.kind.clone(),
                source: event.source.clone(),
                source_weight_bps: event.source_weight_bps,
                causation_index,
                depth: event.depth,
            });
            normalized.push(EventEvidence {
                index,
                position: event.position,
                kind: event.kind.clone(),
                source: event.source.clone(),
                source_weight_bps: event.source_weight_bps,
                causation_index,
                ring_sequence: event.ring_sequence,
                depth: event.depth,
            });
        }
        if observed_contract != scenario.expected_events {
            bail!(
                "trace sample scenario {} drifted: expected {:?}, observed {:?}",
                scenario.id,
                scenario.expected_events,
                observed_contract
            );
        }
        scenarios.push(ScenarioEvidence {
            id: scenario.id.clone(),
            phase: scenario.phase.clone(),
            action: scenario.action.clone(),
            records: normalized,
        });
    }
    if !by_scenario.is_empty() {
        let mut unexpected = by_scenario.keys().copied().collect::<Vec<_>>();
        unexpected.sort_unstable();
        bail!("unexpected trace sample correlations: {unexpected:?}");
    }
    Ok(scenarios)
}

fn validate_observations(
    contract: &SampleContract,
    persisted: &[PersistedEvent],
    schema_columns: &[String],
) -> anyhow::Result<EvidenceObservations> {
    let positions_contiguous = persisted
        .iter()
        .enumerate()
        .all(|(index, event)| event.position == index as u64 + 1);
    let event_ids = persisted
        .iter()
        .map(|event| event.event_id.as_str())
        .collect::<BTreeSet<_>>();
    let causation_resolved = persisted.iter().all(|event| {
        event
            .causation_id
            .as_deref()
            .is_none_or(|causation_id| event_ids.contains(causation_id))
    });
    let phase_by_scenario = contract
        .scenarios
        .iter()
        .map(|scenario| (scenario.id.as_str(), scenario.phase.as_str()))
        .collect::<HashMap<_, _>>();
    let before = persisted
        .iter()
        .filter(|event| phase_by_scenario[event.correlation_id.as_str()] == BEFORE_RESTART)
        .collect::<Vec<_>>();
    let after = persisted
        .iter()
        .filter(|event| phase_by_scenario[event.correlation_id.as_str()] == AFTER_RESTART)
        .collect::<Vec<_>>();
    let position_continues_after_restart =
        before
            .last()
            .zip(after.first())
            .is_some_and(|(last_before, first_after)| {
                last_before.position.checked_add(1) == Some(first_after.position)
            });
    let ring_sequence_reset_after_restart = before
        .iter()
        .map(|event| event.ring_sequence)
        .max()
        .zip(after.first())
        .is_some_and(|(last_ring_sequence, first_after)| {
            first_after.ring_sequence == 1 && last_ring_sequence > first_after.ring_sequence
        });
    let privacy_minimized_schema = ["payload", "metadata", "stream_key"]
        .iter()
        .all(|forbidden| !schema_columns.iter().any(|column| column == forbidden));

    let observations = EvidenceObservations {
        positions_contiguous,
        causation_resolved,
        position_continues_after_restart,
        ring_sequence_reset_after_restart,
        privacy_minimized_schema,
    };
    if !observations.positions_contiguous
        || !observations.causation_resolved
        || !observations.position_continues_after_restart
        || !observations.ring_sequence_reset_after_restart
        || !observations.privacy_minimized_schema
    {
        bail!("runtime event trace sample invariants failed");
    }
    Ok(observations)
}

fn render_markdown_summary(evidence: &SampleEvidence) -> String {
    let mut output = format!(
        "# Runtime Event Trace Shadow Synthetic Samples\n\n\
         - Source commit: `{}`\n\
         - Source worktree dirty: `{}`\n\
         - Scope: synthetic-only, privacy-minimized, behavior-neutral design evidence\n\
         - Authority: not a Runtime Event Stream and not a behavior input\n\n\
         ## Scenarios\n\n\
         | Scenario | Phase | Records | Durable positions | Ring sequences |\n\
         | --- | --- | ---: | --- | --- |\n",
        evidence.source_commit, evidence.source_worktree_dirty
    );
    for scenario in &evidence.scenarios {
        let positions = scenario
            .records
            .iter()
            .map(|record| record.position.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let ring_sequences = scenario
            .records
            .iter()
            .map(|record| record.ring_sequence.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} |\n",
            scenario.id,
            scenario.phase,
            scenario.records.len(),
            positions,
            ring_sequences
        ));
    }
    output.push_str(&format!(
        "\n## Observations\n\n\
         - Durable positions are contiguous: `{}`\n\
         - Causation references resolve: `{}`\n\
         - Durable position continues after restart: `{}`\n\
         - Event Ring sequence resets after restart: `{}`\n\
         - Trace table excludes payload, metadata, and stream key: `{}`\n\n\
         ## Limitations\n\n",
        evidence.observations.positions_contiguous,
        evidence.observations.causation_resolved,
        evidence.observations.position_continues_after_restart,
        evidence.observations.ring_sequence_reset_after_restart,
        evidence.observations.privacy_minimized_schema
    ));
    for limitation in &evidence.limitations {
        output.push_str(&format!("- {limitation}\n"));
    }
    output
}
