import fs from 'fs';
import { execSync } from 'child_process';

const pkg = JSON.parse(fs.readFileSync('./package.json', 'utf8'));
const version = pkg.version;
const exePath = `src-tauri/target/release/bundle/nsis/spotlight-win_${version}_x64-setup.exe`;
const zipPath = `spotlight-win_${version}_x64-setup.zip`;

if (fs.existsSync(exePath)) {
  console.log(`[Spotlight-Win] Compressing ${exePath} into ${zipPath}...`);
  try {
    execSync(`powershell -Command "Compress-Archive -Path '${exePath}' -DestinationPath '${zipPath}' -Force"`, { stdio: 'inherit' });
    console.log(`[Spotlight-Win] Successfully created ${zipPath}!`);
  } catch (err) {
    console.error(`[Spotlight-Win] Failed to compress:`, err.message);
    process.exit(1);
  }
} else {
  console.error(`[Spotlight-Win] Error: Could not find the build executable at ${exePath}. Did the build fail?`);
  process.exit(1);
}
