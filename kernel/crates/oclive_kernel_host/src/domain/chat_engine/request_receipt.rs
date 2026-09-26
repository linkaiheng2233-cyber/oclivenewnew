//! One logical user send may cross transports, but enters the turn only once.
use super::conversation_state_role_id;
use crate::domain::ports::chat_request::{ChatRequestStore, Receipt};
use crate::error::{AppError, Result};
use crate::models::dto::{SendMessageRequest, SendMessageResponse};
use sha2::{Digest, Sha256};
use std::future::Future;
use std::time::Duration;

#[cfg(test)]
mod tests;

pub(super) async fn execute(
    db: &dyn ChatRequestStore,
    req: &SendMessageRequest,
    operation: impl Future<Output = Result<SendMessageResponse>>,
) -> Result<SendMessageResponse> {
    let Some(id) = req.client_request_id.as_deref() else {
        return operation.await;
    };
    let (scope, fingerprint, namespace) = identity(req, id)?;
    if !db
        .claim_chat_request(
            &scope,
            id,
            &fingerprint,
            &namespace,
            (&req.role_id, req.scene_id.as_deref().unwrap_or("default")),
        )
        .await?
    {
        return recover_existing(db, &scope, id, &fingerprint).await;
    }
    // Construction of the future above is lazy. Admission precedes ALL execution.
    let outcome = operation.await;
    // CP-INT B9-B11 R1 C3: default-off observation window. The turn has committed and the receipt
    // is still `running`; nothing below has written the outcome yet. Compiled out of normal builds.
    #[cfg(feature = "test-faults")]
    if let Ok(response) = outcome.as_ref() {
        let state_at_park = match db.read_chat_request(&scope, id, &fingerprint).await {
            Ok(Receipt::Running) => "running",
            Ok(Receipt::Completed(_)) => "completed",
            Ok(Receipt::Unconfirmed) => "unconfirmed",
            Err(_) => "unreadable",
        };
        crate::test_faults::hold_before_receipt_commit(&crate::test_faults::ReceiptWindowFacts {
            scope: &scope,
            request_id: id,
            receipt_state_at_park: state_at_park,
            response,
        })
        .await;
    }
    match outcome {
        Ok(response) => match db.complete_chat_request(&scope, id, &response).await {
            Ok(()) => Ok(response),
            Err(error) => {
                tracing::error!(target: "oclive_chat", %error, "chat outcome receipt could not be committed");
                Err(AppError::ChatRequestUnconfirmed)
            }
        },
        Err(error) => {
            tracing::warn!(target: "oclive_chat", %error, "admitted chat did not produce a confirmed result");
            if let Err(mark_error) = db.mark_chat_request_unconfirmed(&scope, id).await {
                tracing::error!(target: "oclive_chat", %mark_error, "chat receipt remains running; never reclaim");
            }
            Err(AppError::ChatRequestUnconfirmed)
        }
    }
}

pub(super) async fn recover(
    db: &dyn ChatRequestStore,
    req: &SendMessageRequest,
) -> Result<SendMessageResponse> {
    let id = req
        .client_request_id
        .as_deref()
        .ok_or(AppError::ChatRequestUnconfirmed)?;
    let (scope, fingerprint, _) = identity(req, id)?;
    recover_existing(db, &scope, id, &fingerprint).await
}

fn identity(req: &SendMessageRequest, id: &str) -> Result<(String, String, String)> {
    let parsed = uuid::Uuid::parse_str(id).map_err(|_| {
        AppError::InvalidParameter("client_request_id must be a canonical UUID".into())
    })?;
    if parsed.is_nil() || parsed.to_string() != id {
        return Err(AppError::InvalidParameter(
            "client_request_id must be a canonical non-nil UUID".into(),
        ));
    }
    if req.user_message.trim().is_empty() {
        return Err(AppError::EmptyMessage);
    }
    if req.adult.as_ref().and_then(|a| a.stage.as_ref()).is_some() {
        return Err(AppError::InvalidParameter(
            "staged adult beats use their existing generation identity".into(),
        ));
    }
    // Type-driven JSON excludes transport and client identity, retains all semantic
    // fields. Composite JSON scope avoids concatenation/sanitization collisions.
    let scope = serde_json::to_string(&(&req.role_id, &req.session_id))?;
    let mut payload = serde_json::to_value(req)?;
    if let Some(object) = payload.as_object_mut() {
        object.remove("client_request_id");
    }
    let fingerprint = format!("{:x}", Sha256::digest(serde_json::to_vec(&payload)?));
    let namespace = conversation_state_role_id(&req.role_id, req.session_id.as_deref());
    Ok((scope, fingerprint, namespace))
}

async fn recover_existing(
    db: &dyn ChatRequestStore,
    scope: &str,
    id: &str,
    fingerprint: &str,
) -> Result<SendMessageResponse> {
    // No reclaim on timeout, crash or cancellation. A later request can still
    // recover a completed response, but never launch another operation.
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match db.read_chat_request(scope, id, fingerprint).await? {
                Receipt::Completed(json) => {
                    return serde_json::from_str(&json)
                        .map_err(|_| AppError::ChatRequestUnconfirmed)
                }
                Receipt::Unconfirmed => return Err(AppError::ChatRequestUnconfirmed),
                Receipt::Running => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    })
    .await
    .unwrap_or(Err(AppError::ChatRequestUnconfirmed))
}
