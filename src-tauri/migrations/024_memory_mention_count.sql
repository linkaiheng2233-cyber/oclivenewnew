-- 长期记忆提及次数（艾宾浩斯强化）
ALTER TABLE long_term_memory ADD COLUMN mention_count INTEGER NOT NULL DEFAULT 1;
