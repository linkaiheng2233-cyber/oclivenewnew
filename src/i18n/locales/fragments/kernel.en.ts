/** kernel connection — en. */
export default {
  kernel: {
    status: {
      attached: 'Shared kernel connected',
      spawned: 'Local kernel started',
      offline: 'Kernel offline · retry',
      checking: 'Checking connection…',
      offlineTapReconnect: 'Offline · tap to reconnect',
      offlineRetryFailed: 'Reconnect failed · tap to retry',
      reconnecting: 'Reconnecting to kernel…',
      reconnect: 'Reconnect kernel',
      aria: 'Kernel connection status',
    },
    chat: {
      disconnected: 'Connection to the kernel was lost. Use the status bar above to reconnect.',
    },
    diagnostics: {
      title: 'Kernel diagnostics',
      mode: 'Mode',
      port: 'Port',
      binary: 'Binary',
      tier: 'Tier',
      healthy: 'Health',
      healthyYes: 'OK',
      healthyNo: 'Unreachable',
      sharedRuntime: 'Shared runtime binary',
      sharedRuntimeMtime: 'Last modified',
      healthJson: 'Health JSON',
      refresh: 'Refresh',
      reconnect: 'Reconnect',
    },
  },
}
