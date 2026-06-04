-- 用户叙事场景（与角色 current_scene 分离）；缺省与 current_scene 对齐（迁移时回填）
ALTER TABLE role_runtime ADD COLUMN user_presence_scene TEXT;

UPDATE role_runtime
SET user_presence_scene = current_scene
WHERE user_presence_scene IS NULL;
