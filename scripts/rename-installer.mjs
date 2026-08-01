import { mkdir, readdir, rename, rm } from 'node:fs/promises'
import { resolve } from 'node:path'

const version = '26.0.0'
const variant = process.argv[2] === 'standard' ? 'standard' : 'uiaccess'
const directory = resolve('src-tauri/target/release/bundle/nsis')
const outputName = variant === 'standard'
  ? `CyreneCompass_${version}_x64-standard-setup.exe`
  : `CyreneCompass_${version}_x64-setup.exe`
const destination = resolve(directory, outputName)
await mkdir(directory, { recursive: true })
await rm(destination, { force: true })
const source = (await readdir(directory)).find(name => name.includes(`_${version}_`) && name.endsWith('_x64-setup.exe') && name !== `CyreneCompass_${version}_x64-setup.exe`)
if (!source) throw new Error('NSIS installer was not found')
await rename(resolve(directory, source), destination)
console.log(`Installer: ${destination}`)
