/** RPC request history row for dev-tools RpcTester session storage. */
export interface RpcHistoryItem {
  id: string
  method: string
  paramsText: string
  at: number
}
