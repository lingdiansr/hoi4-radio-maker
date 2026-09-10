import { invokeCommand } from '@/api/client'
import { errorMessage } from '@/utils/errors'
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
      if (!options?.silent) {
        toast.display(errorMessage(err), 'error')
      }
      console.error('[command error]', cmd, err)
      return undefined
    }
  }

  return { run }
}
