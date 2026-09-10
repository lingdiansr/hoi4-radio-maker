import { i18n } from '@/i18n'
import type { AppError } from '@/api/client'

/**
 * Localised user-facing text for a backend error.
 *
 * Maps the `Hoi4RadioError` wire tag (`{ type, ...fields }`) onto
 * `errors.<type>`, passing the variant's own fields as interpolation params.
 * Unrecognised types fall back to the backend's `message`, then to a generic
 * label, so a new backend variant never renders as an empty toast.
 */
export function errorMessage(err: unknown): string {
 const { t, te } = i18n.global
 const e = (err ?? {}) as Partial<AppError> & Record<string, unknown>
 const type = typeof e.type === 'string' && e.type ? e.type : 'unknown'
 const key = `errors.${type}`

 if (te(key)) {
  return t(key, {
   ...e,
   message: typeof e.message === 'string' ? e.message : '',
  })
 }

 return typeof e.message === 'string' && e.message
  ? e.message
  : t('common.operationFailed')
}
