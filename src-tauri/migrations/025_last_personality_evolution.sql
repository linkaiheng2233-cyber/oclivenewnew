-- 上次基于虚拟时间的阶段性性格演化锚点（毫秒）
ALTER TABLE role_runtime ADD COLUMN last_personality_evolution_virtual_ms INTEGER NOT NULL DEFAULT 0;
