import fs from 'node:fs'
import path from 'node:path'

function isFile(candidate) {
  try {
    return fs.statSync(candidate).isFile()
  }
  catch {
    return false
  }
}

function versionedLlamaCppCandidates(root) {
  if (!fs.existsSync(root))
    return []
  try {
    return fs
      .readdirSync(root, { withFileTypes: true })
      .filter(entry => entry.isDirectory())
      .map(entry => entry.name)
      .sort((left, right) => right.localeCompare(left, undefined, { numeric: true }))
      .map(name => path.join(root, name, process.platform === 'win32' ? 'llama-server.exe' : 'llama-server'))
  }
  catch {
    return []
  }
}

/**
 * Resolve an unpacked llama.cpp runtime beside a development checkout.
 *
 * Release builds continue to use the signed runtime-pack manifest under app
 * data. This fallback is deliberately limited to dev launchers and never
 * overrides an explicit `OCLIVE_LLAMA_SERVER_PATH`.
 */
export function findWorkspaceDevLlamaServer(repoRoot) {
  const executable = process.platform === 'win32' ? 'llama-server.exe' : 'llama-server'
  const workspaceRoot = path.dirname(path.resolve(repoRoot))
  const roots = [workspaceRoot, path.resolve(repoRoot)]
  const candidates = []
  for (const root of roots) {
    candidates.push(
      path.join(root, 'components', 'llm-runtime', 'bin', executable),
      path.join(root, 'components', 'llama.cpp', executable),
      ...versionedLlamaCppCandidates(path.join(root, 'components', 'llama.cpp')),
    )
  }
  return candidates.find(isFile) ?? null
}

export function resolveChatProDevRuntimeEnv(repoRoot, sourceEnv = process.env) {
  const env = { ...sourceEnv }
  if (env.OCLIVE_LLAMA_SERVER_PATH?.trim()) {
    return { env, inferredRuntimePath: null }
  }
  const inferredRuntimePath = findWorkspaceDevLlamaServer(repoRoot)
  if (inferredRuntimePath)
    env.OCLIVE_LLAMA_SERVER_PATH = inferredRuntimePath
  return { env, inferredRuntimePath }
}
