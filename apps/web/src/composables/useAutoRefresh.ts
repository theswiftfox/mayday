// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { ref, watch, onMounted, onUnmounted } from 'vue'
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

  let stopFocusWatcher: (() => void) | null = null

  onMounted(() => {
    // Initial fetch
    refresh()

    // Set up polling
    if (intervalMs > 0) {
      intervalId = setInterval(refresh, intervalMs)
    }

    // Watch for focus changes
    if (refreshOnFocus) {
      let wasUnfocused = false
      stopFocusWatcher = watch(focused, (isFocused) => {
        if (isFocused && wasUnfocused) {
          refresh()
        }
        wasUnfocused = !isFocused
      })
    }
  })

  onUnmounted(() => {
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
    if (stopFocusWatcher) {
      stopFocusWatcher()
      stopFocusWatcher = null
    }
  })

  return { refresh, isRefreshing }
}
