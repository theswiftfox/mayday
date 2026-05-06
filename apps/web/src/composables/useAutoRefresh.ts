import { ref, onMounted, onUnmounted } from 'vue'
import { useWindowFocus } from '@vueuse/core'

/**
 * Composable for auto-refreshing data at intervals and on window focus
 */
export function useAutoRefresh(
  fetchFn: () => Promise<void>,
  options: { intervalMs?: number; refreshOnFocus?: boolean } = {}
) {
  const { intervalMs = 300_000, refreshOnFocus = true } = options
  const isRefreshing = ref(false)
  let intervalId: ReturnType<typeof setInterval> | null = null
  const focused = useWindowFocus()

  async function refresh() {
    if (isRefreshing.value) return
    isRefreshing.value = true
    try {
      await fetchFn()
    } finally {
      isRefreshing.value = false
    }
  }

  onMounted(() => {
    // Initial fetch
    refresh()

    // Set up polling
    if (intervalMs > 0) {
      intervalId = setInterval(refresh, intervalMs)
    }
  })

  onUnmounted(() => {
    if (intervalId) {
      clearInterval(intervalId)
    }
  })

  // Watch for focus changes
  if (refreshOnFocus) {
    let wasUnfocused = false
    import('vue').then(({ watch }) => {
      watch(focused, (isFocused) => {
        if (isFocused && wasUnfocused) {
          refresh()
        }
        wasUnfocused = !isFocused
      })
    })
  }

  return { refresh, isRefreshing }
}
