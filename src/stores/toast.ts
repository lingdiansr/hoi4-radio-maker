import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useToastStore = defineStore('toast', () => {
  const show = ref(false)
  const message = ref('')
  const color = ref('error')
  const timeout = ref(5000)

  function display(msg: string, type: 'error' | 'success' = 'error', duration = 5000) {
    message.value = msg
    color.value = type === 'success' ? 'success' : 'error'
    timeout.value = duration
    show.value = true
  }

  return { show, message, color, timeout, display }
})
