import { trace, debug, info, warn, error } from '@tauri-apps/plugin-log'

export const logger = {
  trace: (msg: string) => trace(msg),
  debug: (msg: string) => debug(msg),
  info: (msg: string) => info(msg),
  warn: (msg: string) => warn(msg),
  error: (msg: string) => error(msg),
}
