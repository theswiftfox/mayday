// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { ref, onUnmounted, getCurrentInstance } from 'vue'

/**
 * Returns a reactive `now` ref that updates every `intervalMs` milliseconds.
 * Useful for time-dependent computed properties (e.g., "starting soon" checks).
 */
export function useNow(intervalMs = 30_000) {
  const now = ref(Date.now())
  const timer = setInterval(() => {
    now.value = Date.now()
  }, intervalMs)

  if (getCurrentInstance()) {
    onUnmounted(() => {
      clearInterval(timer)
    })
  } else {
    console.warn('useNow() called outside component setup — timer will not be auto-cleared')
  }

  return now
}
