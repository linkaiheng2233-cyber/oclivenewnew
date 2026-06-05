import { invokeWithFriendlyError } from './helpers'

export interface McpToolManifest {
  name: string
  description?: string | null
}


export interface McpServerManifest {
  id: string
  name: string
  transport?: string
  url?: string | null
  command?: string | null
  args?: string[]
  tools?: McpToolManifest[]
}


export interface McpToolCallResult {
  server_id: string
  tool_name: string
  result: unknown
}


export interface AgentToolCallTrace {
  server_id: string
  tool_name: string
  params: unknown
  result: unknown
}


export interface AgentDebugTrace {
  timestamp_ms: number
  role_id: string
  session_namespace: string
  message: string
  plan: string
  tool_calls: AgentToolCallTrace[]
  reply: string
  error?: string | null
}


export async function listMcpServers(): Promise<McpServerManifest[]> {
  return invokeWithFriendlyError<McpServerManifest[]>('list_mcp_servers', {})
}


export async function listMcpTools(serverId: string): Promise<McpToolManifest[]> {
  return invokeWithFriendlyError<McpToolManifest[]>('list_mcp_tools', {
    req: { server_id: serverId },
  })
}


export async function callMcpTool(
  serverId: string,
  toolName: string,
  params: unknown = {},
): Promise<McpToolCallResult> {
  return invokeWithFriendlyError<McpToolCallResult>('call_mcp_tool', {
    req: {
      server_id: serverId,
      tool_name: toolName,
      params,
    },
  })
}


export async function getAgentDebugTraces(): Promise<AgentDebugTrace[]> {
  return invokeWithFriendlyError<AgentDebugTrace[]>('get_agent_debug_traces', {})
}


export async function clearAgentDebugTraces(): Promise<void> {
  return invokeWithFriendlyError<void>('clear_agent_debug_traces', {})
}


export interface HighRiskGrantsSnapshot {
  'mcp:http': string[]
  'mcp:stdio': string[]
  'process:spawn': string[]
  'network:*': string[]
}


export type HighRiskGrantKind
  = | 'mcp:http'
    | 'mcp:stdio'
    | 'process:spawn'
    | 'network:*'


export async function listHighRiskGrants(): Promise<HighRiskGrantsSnapshot> {
  return invokeWithFriendlyError<HighRiskGrantsSnapshot>('list_high_risk_grants', {})
}


export async function grantHighRiskCapability(
  kind: HighRiskGrantKind,
  id: string,
): Promise<void> {
  return invokeWithFriendlyError<void>('grant_high_risk_capability', {
    req: { kind, id },
  })
}


export async function revokeHighRiskCapability(
  kind: HighRiskGrantKind,
  id: string,
): Promise<void> {
  return invokeWithFriendlyError<void>('revoke_high_risk_capability', {
    req: { kind, id },
  })
}

