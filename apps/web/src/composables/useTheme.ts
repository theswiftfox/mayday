// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { ref, watch } from 'vue'

const STORAGE_KEY = 'myday-theme'

type Theme = 'light' | 'dark' | 'system' | 'win98'

const currentTheme = ref<Theme>(
  (localStorage.getItem(STORAGE_KEY) as Theme) || 'system'
)

function applyTheme(theme: Theme) {
  document.documentElement.setAttribute('data-theme', theme)

  // Sync native title bar and window background with the selected theme in Tauri
  if ((window as any).__TAURI_INTERNALS__) {
    import('@tauri-apps/api/webviewWindow').then(({ getCurrentWebviewWindow }) => {
      const tauriTheme = theme === 'system' ? null : theme === 'dark' ? 'dark' : 'light'
      getCurrentWebviewWindow().setTheme(tauriTheme)
    }).catch((e) => console.warn('Failed to set window theme:', e))

    // Read the background color from the CSS variable so it stays in sync with main.css
    const bg = getComputedStyle(document.documentElement).getPropertyValue('--color-background').trim()
    if (bg) {
      import('@tauri-apps/api/window').then(({ getCurrentWindow }) => {
        getCurrentWindow().setBackgroundColor(bg)
      }).catch((e) => console.warn('Failed to set window background color:', e))
    }
  }
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
