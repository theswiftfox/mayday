// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { ref, watch } from 'vue'

const STORAGE_KEY = 'myday-theme'

type Theme = 'light' | 'dark' | 'system'

const currentTheme = ref<Theme>(
  (localStorage.getItem(STORAGE_KEY) as Theme) || 'system'
)

function applyTheme(theme: Theme) {
  document.documentElement.setAttribute('data-theme', theme)
}

// Apply immediately on module load
applyTheme(currentTheme.value)

// Single module-level watcher — no leak regardless of how many times useTheme() is called
watch(currentTheme, (newTheme) => {
  applyTheme(newTheme)
})

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme
    localStorage.setItem(STORAGE_KEY, theme)
  }

  return {
    theme: currentTheme,
    setTheme,
  }
}
