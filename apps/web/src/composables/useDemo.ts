// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
//
// Demo mode for screenshots. Activated via:
//   - ?demo query param in the URL (browser dev)
//   - Ctrl+Shift+D / Cmd+Shift+D keyboard shortcut (works in Tauri)
//   - localStorage key 'myday-demo' = 'true'

import { ref, readonly } from 'vue'

const STORAGE_KEY = 'myday-demo'

function isActive(): boolean {
  return (
    new URLSearchParams(window.location.search).has('demo') ||
    localStorage.getItem(STORAGE_KEY) === 'true'
  )
}

const demoMode = ref(isActive())

function toggle() {
  const next = !demoMode.value
  demoMode.value = next
  if (next) {
    localStorage.setItem(STORAGE_KEY, 'true')
  } else {
    localStorage.removeItem(STORAGE_KEY)
  }
}

// Global keyboard shortcut: Ctrl+Shift+D / Cmd+Shift+D
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'd') {
    e.preventDefault()
    toggle()
    // Reload to apply everywhere (dashboard store reads this at fetch time)
    window.location.reload()
  }
})

export function useDemo() {
  return {
    isDemoMode: readonly(demoMode),
    toggleDemo: toggle,
  }
}

/** Non-reactive check for use outside of Vue components (e.g. store init) */
export function checkDemoMode(): boolean {
  return demoMode.value
}
