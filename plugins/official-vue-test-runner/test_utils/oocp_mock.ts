/**
 * Minimal OOCP v0.1-style envelopes for Vitest / Vue component tests.
 * Shapes follow `crates/oclive_core/src/oocp/mod.rs` and OOCP_SPEC_v0_1 (JSON uses snake_case params).
 */

export type OocpRequestEnvelope = {
  type: "request"
  id: string | number
  method: string
  params: Record<string, unknown>
}

export type OocpResponseEnvelope = {
  type: "response"
  id: string | number
  result: Record<string, unknown>
}

export type OocpErrorEnvelope = {
  type: "error"
  id: string | number | null
  error: {
    code: string
    message: string
    data?: Record<string, unknown>
  }
}

let __seq = 1

/** Deterministic-ish ids for snapshots; override per call if needed. */
export function nextOocpId(): string {
  return `mock-oocp-${__seq++}`
}

/** Build a client → kernel OOCP request object (plain JSON). */
export function mockOocpRequest(
  method: string,
  params: Record<string, unknown> = {},
  id: string | number = nextOocpId(),
): OocpRequestEnvelope {
  return { type: "request", id, method, params }
}

/** Build a kernel → client success response for a given request id. */
export function mockOocpResponse(
  id: string | number,
  result: Record<string, unknown>,
): OocpResponseEnvelope {
  return { type: "response", id, result }
}

/** Build an error frame (e.g. to assert your adapter surfaces failures). */
export function mockOocpError(
  id: string | number | null,
  code: string,
  message: string,
  data: Record<string, unknown> = {},
): OocpErrorEnvelope {
  return { type: "error", id, error: { code, message, data } }
}

export type TestOocpSession = {
  roleId: string
  sessionNs: string
  /** Params you can spread into `chat.send_message` tests. */
  chatSendMessageParams: Record<string, unknown>
  /** Typical `session.create` RPC request your transport layer would send. */
  sessionCreateRequest: OocpRequestEnvelope
  /** Stubbed success body for `session.create` (snake_case keys like the wire JSON). */
  sessionCreateResult: Record<string, unknown>
  /** Matching `mockOocpResponse` for `session.create`. */
  sessionCreateResponse: OocpResponseEnvelope
}

/**
 * Quick fixture: stable `session_ns`, a `session.create` request/response pair,
 * and base params for `chat.send_message`.
 */
export function createTestSession(options?: {
  roleId?: string
  sessionNs?: string
  requestId?: string | number
}): TestOocpSession {
  const roleId = options?.roleId ?? "test.role.fixture"
  const sessionNs =
    options?.sessionNs ?? `${roleId}__sess__00000000-0000-4000-8000-000000000001`
  const requestId = options?.requestId ?? nextOocpId()

  const sessionCreateResult: Record<string, unknown> = {
    session_ns: sessionNs,
    role: {
      name: "Test Role",
      scenes: ["default", "cafe"],
      interaction_mode: "chat",
    },
  }

  const sessionCreateRequest = mockOocpRequest(
    "session.create",
    { role_id: roleId, session_id: null, scene_id: "default" },
    requestId,
  )

  const sessionCreateResponse = mockOocpResponse(requestId, sessionCreateResult)

  const chatSendMessageParams: Record<string, unknown> = {
    session_ns: sessionNs,
    user_message: "hello from vitest",
    scene_id: "default",
  }

  return {
    roleId,
    sessionNs,
    chatSendMessageParams,
    sessionCreateRequest,
    sessionCreateResult,
    sessionCreateResponse,
  }
}
