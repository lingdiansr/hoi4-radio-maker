import { spawn } from 'child_process'
import { fileURLToPath } from 'url'
import { dirname, resolve } from 'path'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const out = 'ignore'
const child = spawn('bun', ['run', 'preview', '--', '--port', '4173'], {
  cwd: root,
  detached: true,
  stdio: [out, out, out],
})
child.unref()
console.log('preview server pid:', child.pid)
