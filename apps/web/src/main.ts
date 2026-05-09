// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import './assets/main.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')

// In Tauri, intercept clicks on external links and open them via the shell plugin
// instead of trying to navigate the webview.
const tauriInternals = (window as any).__TAURI_INTERNALS__
if (tauriInternals) {
  document.addEventListener('click', (e) => {
    const anchor = (e.target as HTMLElement).closest('a[href]') as HTMLAnchorElement | null
    if (!anchor) return

    const href = anchor.getAttribute('href')
    if (href && (href.startsWith('http://') || href.startsWith('https://'))) {
      e.preventDefault()
      e.stopPropagation()
      tauriInternals.invoke('plugin:shell|open', { path: href, with: undefined })
    }
  }, true) // Use capture phase to intercept before RouterLink handles it
}
