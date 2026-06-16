import { invokeCommand, type AppError } from '@/api/client'
import { useToastStore } from '@/stores/toast'

export function useCommand() {
  const toast = useToastStore()

  async function run<T>(
    cmd: string,
    args?: Record<string, unknown>,
    options?: { successMsg?: string; silent?: boolean }
  ): Promise<T | undefined> {
    try {
      const result = await invokeCommand<T>(cmd, args)
      if (options?.successMsg) {
        toast.display(options.successMsg, 'success')
      }
      return result
    } catch (err) {
      const appErr = err as AppError
      if (!options?.silent) {
        toast.display(appErr.message || '操作失败', 'error')
      }
      console.error('[command error]', cmd, err)
      return undefined
    }
  }

  return { run }
}
