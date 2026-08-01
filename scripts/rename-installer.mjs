import { mkdir, readdir, rename, rm } from 'node:fs/promises'
import { resolve } from 'node:path'

const version = '26.0.0'
const directory = resolve('src-tauri/target/release/bundle/nsis')
const destination = resolve(directory, `CyreneCompass_${version}_x64-setup.exe`)
await mkdir(directory, { recursive: true })
await rm(destination, { force: true })
const source = (await readdir(directory)).find(name => name.includes(`_${version}_`) && name.endsWith('_x64-setup.exe') && name !== `CyreneCompass_${version}_x64-setup.exe`)
if (!source) throw new Error('NSIS installer was not found')
await rename(resolve(directory, source), destination)
console.log(`Installer: ${destination}`)
