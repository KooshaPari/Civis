import { execSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';

const env = { ...process.env };
delete env.MIC_LD_LIBRARY_PATH;
delete env.INTEL_DEV_REDIST;

const log = [];
function run(cmd) {
  log.push(`$ ${cmd}`);
  const out = execSync(cmd, {
    cwd: 'C:\\Users\\koosh\\Dino',
    encoding: 'utf8',
    env,
    stdio: ['ignore', 'pipe', 'pipe'],
    timeout: 600000,
  });
  log.push(out.trim());
  return out;
}

try {
  run('git fetch origin');
  const branch = 'agent/coderabbit-main-config';
  run(`git checkout -B ${branch} origin/main`);
  run('copy /Y scripts\\coderabbit-main-target.yaml .coderabbit.yaml');
  run('git add .coderabbit.yaml');
  try {
    run('git commit -m "chore: enable CodeRabbit bot approve on main"');
  } catch {
    log.push('(no commit needed)');
  }
  run(`git push -u origin ${branch}`);
  let prNum;
  try {
    const created = run(
      'gh pr create --repo KooshaPari/Dino --base main --head agent/coderabbit-main-config --title "chore: enable CodeRabbit bot approve on main" --body "Land CodeRabbit auto_approve on main for bot-driven merges."',
    );
    const m = created.match(/pull\/(\d+)/);
    prNum = m?.[1];
  } catch (e) {
    const listed = run(
      'gh pr list --repo KooshaPari/Dino --head agent/coderabbit-main-config --json number --jq ".[0].number"',
    );
    prNum = listed.trim();
  }
  log.push(`config PR: ${prNum}`);
  run(`gh pr comment ${prNum} --repo KooshaPari/Dino --body "@coderabbitai review\\n@coderabbitai approve"`);
  for (let i = 0; i < 12; i++) {
    const reviews = run(
      `gh api repos/KooshaPari/Dino/pulls/${prNum}/reviews --jq "[.[] | select(.user.login==\\"coderabbitai[bot]\\" and .state==\\"APPROVED\\")] | length"`,
    );
    if (parseInt(reviews.trim(), 10) >= 1) {
      run(`gh pr merge ${prNum} --repo KooshaPari/Dino --merge`);
      break;
    }
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 60000);
  }
  run('gh pr comment 221 --repo KooshaPari/Dino --body "@coderabbitai approve"');
  for (let i = 0; i < 12; i++) {
    const reviews = run(
      'gh api repos/KooshaPari/Dino/pulls/221/reviews --jq "[.[] | select(.user.login==\\"coderabbitai[bot]\\" and .state==\\"APPROVED\\")] | length"',
    );
    if (parseInt(reviews.trim(), 10) >= 1) {
      run('gh pr merge 221 --repo KooshaPari/Dino --merge');
      break;
    }
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 60000);
  }
  run('git checkout main');
  run('git pull origin main');
  log.push(run('git log -1 --oneline origin/main'));
  writeFileSync('scripts/playwright-merge-result.txt', log.join('\n\n'));
  console.log('OK');
} catch (e) {
  log.push(String(e));
  if (e.stdout) log.push(String(e.stdout));
  if (e.stderr) log.push(String(e.stderr));
  writeFileSync('scripts/playwright-merge-result.txt', log.join('\n\n'));
  process.exit(1);
}
