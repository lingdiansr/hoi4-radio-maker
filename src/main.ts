import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { attachConsole, debug, error, info, trace, warn } from '@tauri-apps/plugin-log'
import '@mdi/font/css/materialdesignicons.css'
import vuetify from '@/plugins/vuetify'
import App from './App.vue'
import router from './router'

function forwardConsole(
  fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
  logger: (message: string) => Promise<void>
) {
  const original = console[fnName]
  console[fnName] = (message: unknown) => {
    original(message)
    logger(String(message))
  }
}

async function bootstrap() {
  forwardConsole('log', trace)
  forwardConsole('debug', debug)
  forwardConsole('info', info)
  forwardConsole('warn', warn)
  forwardConsole('error', error)

  await attachConsole()

  const app = createApp(App)
  app.use(createPinia())
  app.use(router)
  app.use(vuetify)
  app.mount('#app')
}

bootstrap()
