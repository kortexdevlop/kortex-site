ALTER TABLE device_sessions ADD COLUMN ai_allowed INTEGER NOT NULL DEFAULT 0 CHECK(ai_allowed IN(0,1));
