import {readFileSync} from 'node:fs';
const pkg=JSON.parse(readFileSync('package.json'));
const tauri=JSON.parse(readFileSync('src-tauri/tauri.conf.json'));
const version=process.env.GITHUB_REF_NAME?.replace(/^v/,'')??pkg.version;
for(const file of ['src-tauri/Cargo.toml','crates/wmark-core/Cargo.toml']){if(!readFileSync(file,'utf8').includes(`version = "${version}"`))throw new Error(`Version mismatch: ${file}`)}
if(pkg.version!==version||tauri.version!==version)throw new Error('Package / tag version mismatch');
console.log(`Version consistent: ${version}`);
