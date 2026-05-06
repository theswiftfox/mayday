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

export function useTheme() {
  function setTheme(theme: Theme) {
    currentTheme.value = theme
    localStorage.setItem(STORAGE_KEY, theme)
    applyTheme(theme)
  }

  // Watch for changes (e.g., from settings page)
  watch(currentTheme, (newTheme) => {
    applyTheme(newTheme)
  })

  return {
    theme: currentTheme,
    setTheme,
  }
}
