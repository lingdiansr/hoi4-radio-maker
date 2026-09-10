import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zhCN from './locales/zh-CN'

/** Every locale the app ships with; `zh-CN` is the primary authoring locale. */
export const SUPPORTED_LOCALES = ['zh-CN', 'en'] as const
export type SupportedLocale = (typeof SUPPORTED_LOCALES)[number]

/** Native labels for the language picker (never translated). */
export const LOCALE_LABELS: Record<SupportedLocale, string> = {
  'zh-CN': '简体中文',
  en: 'English',
}

/** System locale: Chinese systems get zh-CN, everything else English. */
export function detectLocale(): SupportedLocale {
  const nav = typeof navigator !== 'undefined' ? navigator.language : ''
  return nav.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en'
}

export function isSupportedLocale(value: unknown): value is SupportedLocale {
  return (
    typeof value === 'string' && (SUPPORTED_LOCALES as readonly string[]).includes(value)
  )
}

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: { 'zh-CN': zhCN, en },
})

/** Apply a locale (falling back to the system one) and keep `<html lang>` in sync. */
export function setLocale(value: unknown): SupportedLocale {
  const locale = isSupportedLocale(value) ? value : detectLocale()
  i18n.global.locale.value = locale
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale
  }
  return locale
}
