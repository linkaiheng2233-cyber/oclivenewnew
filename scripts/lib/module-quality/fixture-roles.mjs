import { cpSync, existsSync } from "node:fs";
import { join } from "node:path";

import { chatProRolesDir } from "../chat-pro-roles-dir.mjs";
import { assert, readJson, writeJson } from "./contracts.mjs";

const REMOTE_SLOTS = ["memory", "emotion", "prompt", "llm"];

export function prepareFixtureRoles(suite, tempRoot, repoRoot) {
  const sourceRoot = chatProRolesDir(repoRoot);
  const rolesRoot = join(tempRoot, "roles");
  const casesByRole = new Map();
  for (const testCase of suite.cases) {
    const existing = casesByRole.get(testCase.role_id) ?? [];
    existing.push(testCase);
    casesByRole.set(testCase.role_id, existing);
  }

  for (const [roleId, roleCases] of casesByRole) {
    const source = join(sourceRoot, roleId);
    const target = join(rolesRoot, roleId);
    assert(existsSync(source), `fixture role does not exist: ${source}`);
    cpSync(source, target, { recursive: true });

    const pipelinePath = join(target, "pipeline.ocblueprint");
    const pipeline = readJson(pipelinePath, `${roleId} pipeline`);
    assert(
      pipeline?.slot_registry && typeof pipeline.slot_registry === "object",
      `${roleId} pipeline has no slot_registry`,
    );
    for (const slot of REMOTE_SLOTS) {
      assert(pipeline.slot_registry[slot], `${roleId} has no ${slot} slot`);
      pipeline.slot_registry[slot].backend = "remote";
    }
    if (pipeline.slot_registry.agent) {
      pipeline.slot_registry.agent.backend = "none";
    }
    if (pipeline.slot_registry.complex_emotion) {
      pipeline.slot_registry.complex_emotion.backend = "none";
    }
    writeJson(pipelinePath, pipeline);

    const memories = roleCases.flatMap((testCase) =>
      testCase.expectations.memory.required.map((content, index) => ({
        id: `mq-${testCase.id}-${index + 1}`,
        content,
        importance: 1,
        scene_id: testCase.scene_id,
      })),
    );
    writeJson(join(target, "memory_seed.json"), {
      schema_version: 1,
      memories,
    });
  }
  return rolesRoot;
}
