-- Exact lookups used by publication verification triggers. The earlier partial
-- active-version index cannot satisfy every status='published' predicate.
-- Avoid scanning the publication history once for every imported tool.
CREATE INDEX idx_publication_jobs_tool_version_status
ON publication_jobs(tool_id,version,status);
