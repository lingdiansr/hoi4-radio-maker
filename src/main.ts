import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { debug, error, info, trace, warn } from '@tauri-apps/plugin-log'
import '@mdi/font/css/materialdesignicons.css'
import vuetify from '@/plugins/vuetify'
import App from './App.vue'
import router from './router'

function serializeArg(arg: unknown): string {
  if (arg instanceof Error) {
    return `${arg.toString()}\n${arg.stack || ''}`
  }
  if (typeof arg === 'string') {
    return arg
  }
  try {
    return JSON.stringify(arg)
  } catch {
    return String(arg)
  }
}

function forwardConsole(
  fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
  logger: (message: string) => Promise<void>
) {
  const original = console[fnName]
  console[fnName] = (...args: unknown[]) => {
    original(...args)
    const message = args.map(serializeArg).join(' ')
    logger(message)
  }
}

async function bootstrap() {
  forwardConsole('log', trace)
  forwardConsole('debug', debug)
  forwardConsole('info', info)
  forwardConsole('warn', warn)
  forwardConsole('error', error)

  const app = createApp(App)
  app.use(createPinia())
  app.use(router)
  app.use(vuetify)
  app.mount('#app')
}

bootstrap()
