import { mkdir, readFile, readdir, rename, rm } from 'node:fs/promises'
import { resolve } from 'node:path'

const { version } = JSON.parse(await readFile(resolve('src-tauri/tauri.conf.json'), 'utf8'))
const directory = resolve('src-tauri/target/release/bundle/nsis')
const outputName = `CyreneCompass_${version}_x64-setup.exe`
const destination = resolve(directory, outputName)
await mkdir(directory, { recursive: true })
const files = await readdir(directory)
const source = files.find(name => name.includes(`_${version}_`) && name.endsWith('_x64-setup.exe') && name !== outputName)
if (source) {
  await rm(destination, { force: true })
  await rename(resolve(directory, source), destination)
} else if (!files.includes(outputName)) {
  throw new Error('NSIS installer was not found')
}
for (const name of await readdir(directory)) {
  if (name !== outputName && /_x64(?:-standard)?-setup\.exe$/i.test(name)) await rm(resolve(directory, name), { force: true })
}
console.log(`Installer: ${destination}`)
