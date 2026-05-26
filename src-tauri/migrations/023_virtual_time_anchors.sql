-- 现实锚点：虚拟时间 = anchor_virtual + (real_now - anchor_real) * speed
ALTER TABLE role_runtime ADD COLUMN virtual_time_anchor_real_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE role_runtime ADD COLUMN virtual_time_anchor_virtual_ms INTEGER NOT NULL DEFAULT 0;
