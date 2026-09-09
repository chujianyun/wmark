import {readdirSync,statSync} from 'node:fs';
import {join} from 'node:path';
const dir=process.argv[2];const files=readdirSync(dir).filter(f=>f.endsWith('.dmg')||f.endsWith('-setup.exe'));
if(!files.length)throw new Error('No desktop installer was built');
for(const f of files){const size=statSync(join(dir,f)).size;if(size<1000000)throw new Error(`Unexpectedly small installer: ${f}`);console.log(`${f}: ${size} bytes`)}
