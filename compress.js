import fs from 'fs';
import { execSync } from 'child_process';

const pkg = JSON.parse(fs.readFileSync('./package.json', 'utf8'));
const version = pkg.version;

const targets = [
  {
    path: `src-tauri/target/release/bundle/msi/spotlight-win_${version}_x64_en-US.msi`,
    zip: `spotlight-win_${version}_x64_en-US.msi.zip`,
    label: 'MSI Installer'
  },
  {
    path: `src-tauri/target/release/bundle/nsis/spotlight-win_${version}_x64-setup.exe`,
    zip: `spotlight-win_${version}_x64-setup.zip`,
    label: 'NSIS Setup'
  }
];

let foundAny = false;

for (const target of targets) {
  if (fs.existsSync(target.path)) {
    foundAny = true;
    console.log(`[Spotlight-Win] Found ${target.label} at ${target.path}`);
    console.log(`[Spotlight-Win] Compressing into ${target.zip}...`);
    try {
      execSync(`powershell -Command "Compress-Archive -Path '${target.path}' -DestinationPath '${target.zip}' -Force"`, { stdio: 'inherit' });
      console.log(`[Spotlight-Win] Successfully created ${target.zip}!`);
    } catch (err) {
      console.error(`[Spotlight-Win] Failed to compress ${target.zip}:`, err.message);
    }
  }
}

if (!foundAny) {
  console.warn(`[Spotlight-Win] Notice: No installer bundles found under src-tauri/target/release/bundle/.`);
}
