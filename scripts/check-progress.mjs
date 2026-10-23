import { readFileSync, existsSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
export function validateProgress(records, sequence) {
  if (!Array.isArray(records) || !records.length) throw Error('Empty progress register');
  const states = ['pending','active','awaiting_approval','validated','blocked'];
  let unfinished = false;
  for (const [index, row] of records.entries()) {
    if (row.id !== sequence[index]) throw Error(`Skipped or unordered step: ${row.id}`);
    if (!states.includes(row.status)) throw Error(`Invalid status: ${row.id}`);
    if (unfinished && row.status !== 'pending') throw Error(`Unapproved predecessor: ${row.id}`);
    const approved = row.approved_by?.trim() && /^\d{4}-\d{2}-\d{2}(?:T.*Z)?$/.test(row.approved_at || '') && !Number.isNaN(Date.parse(row.approved_at));
    if ((row.status === 'validated' || row.frozen) && !approved) throw Error(`Missing human approval: ${row.id}`);
    if (row.waiver) {
      if (!approved || !row.waiver.reason || !row.waiver.evidence) throw Error(`Invalid waiver: ${row.id}`);
      if (row.status === 'validated') throw Error(`Waiver must not claim technical validation: ${row.id}`);
    }
    for (const key of ['source_commit','result_commit','report_path','tests_executed','contract_changes','evidence']) {
      if (row[key] === undefined) throw Error(`Missing ${key}: ${row.id}`);
    }
    if (row.status !== 'validated' && !row.waiver) unfinished = true;
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const markdown=readFileSync('docs/PROGRESS.md','utf8');
    const match=markdown.match(/```json\s*([\s\S]*?)```/);
    if (!match) throw Error('Missing structured progress register');
    const rows=JSON.parse(match[1]);
    const sequence=[...readFileSync('docs/MASTER_PLAN.md','utf8').matchAll(/^### Étape (\d+\.\d+)\b/gm)].map(m=>m[1]);
    validateProgress(rows,sequence);
    for (const row of rows) {
      if (!existsSync(row.report_path)) throw Error(`Missing report: ${row.id}`);
      if (row.waiver && !existsSync(row.waiver.evidence)) throw Error(`Missing waiver evidence: ${row.id}`);
    }
    console.log(`Progress register valid: ${rows.length} ordered steps`);
  } catch (error) { console.error(error.message); process.exitCode=1; }
}
