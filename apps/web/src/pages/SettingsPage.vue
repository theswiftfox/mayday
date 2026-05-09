<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { api } from '@/lib/api'
import { useTheme } from '@/composables/useTheme'

const { setTheme } = useTheme()
const loading = ref(true)
const saving = ref(false)
const error = ref('')
const success = ref('')

// GitHub auth state
const ghAuthStatus = ref<'none' | 'detecting' | 'device_code_pending' | 'authenticated'>('none')
const ghAuthUsername = ref('')
const ghAuthSource = ref('')
const ghDeviceCode = ref<{ user_code: string; verification_uri: string; device_code: string } | null>(null)
let ghPollInterval: ReturnType<typeof setInterval> | null = null

// Calendar auth state
const calAuthStatus = ref<'none' | 'pending' | 'device_code_pending' | 'paste_code_pending' | 'authenticated'>('none')
let calPollInterval: ReturnType<typeof setInterval> | null = null
const calDeviceCode = ref<{ user_code: string; verification_uri: string } | null>(null)
const calManualCode = ref('')

const config = ref({
  github: { token: '', username: '', repos: '', poll_interval: 300, oauth_client_id: '' },
  jira: { host: '', email: '', api_token: '', project_keys: '', poll_interval: 300 },
  gitlab: { host: '', token: '', username: '', projects: '' as string, poll_interval: 300 },
  calendar: { source: 'ics', ics_url: '', ms_client_id: '', ms_tenant_id: '', ms_redirect_uri: '', poll_interval: 300 },
  general: { theme: 'system', refresh_on_focus: true },
})

onMounted(async () => {
  try {
    const data = await api.getConfig()
    if (data) {
      // Map API response to form fields
      if (data.github) {
        config.value.github.username = data.github.username || ''
        config.value.github.repos = (data.github.repos || []).join(', ')
        config.value.github.poll_interval = data.github.poll_interval_secs || 300
        config.value.github.oauth_client_id = data.github.oauth_client_id || ''
        if (data.github.has_token) {
          ghAuthStatus.value = 'authenticated'
          ghAuthUsername.value = data.github.username || ''
          ghAuthSource.value = data.github.token_source || 'manual'
        }
      }
      if (data.jira) {
        config.value.jira.host = data.jira.host || ''
        config.value.jira.email = data.jira.email || ''
        config.value.jira.project_keys = (data.jira.project_keys || []).join(', ')
        config.value.jira.poll_interval = data.jira.poll_interval_secs || 300
      }
      if (data.gitlab) {
        config.value.gitlab.host = data.gitlab.host || ''
        config.value.gitlab.username = data.gitlab.username || ''
        config.value.gitlab.projects = (data.gitlab.projects || []).map((p: any) => p.path).join(', ')
        config.value.gitlab.poll_interval = data.gitlab.poll_interval_secs || 300
      }
      if (data.calendar) {
        config.value.calendar.source = data.calendar.source || 'ics'
        config.value.calendar.ics_url = data.calendar.ics_url || ''
        config.value.calendar.ms_client_id = data.calendar.ms_client_id || ''
        config.value.calendar.ms_tenant_id = data.calendar.ms_tenant_id || ''
        config.value.calendar.ms_redirect_uri = data.calendar.ms_redirect_uri || ''
        config.value.calendar.poll_interval = data.calendar.poll_interval_secs || 300
        if (data.calendar.has_ms_refresh_token) {
          calAuthStatus.value = 'authenticated'
        }
      }
      if (data.general) {
        config.value.general = data.general
      }
      // Sync theme from localStorage (source of truth for immediate application)
      const storedTheme = localStorage.getItem('myday-theme')
      if (storedTheme) {
        config.value.general.theme = storedTheme
      }
    }
  } catch (e: any) {
    error.value = e.message || 'Failed to load settings'
  } finally {
    loading.value = false
  }
})

onUnmounted(() => {
  if (ghPollInterval) clearInterval(ghPollInterval)
  if (calPollInterval) clearInterval(calPollInterval)
})

async function save() {
  saving.value = true
  error.value = ''
  success.value = ''
  try {
    // Resolve GitLab project paths to IDs before saving
    const gitlabPayload: any = { ...config.value.gitlab }
    if (gitlabPayload.projects && gitlabPayload.host) {
      const paths = (gitlabPayload.projects as string)
        .split(',')
        .map((s: string) => s.trim())
        .filter((s: string) => s.length > 0)

      const resolved = await Promise.all(
        paths.map(async (path: string) => {
          try {
            return await api.resolveGitLabProject(gitlabPayload.host, path)
          } catch {
            // If resolution fails, skip this project
            return null
          }
        })
      )

      gitlabPayload.projects = resolved.filter(Boolean)
    } else {
      gitlabPayload.projects = []
    }
    delete gitlabPayload.project_ids

    await api.updateConfig({ ...config.value, gitlab: gitlabPayload })
    success.value = 'Settings saved successfully'
  } catch (e: any) {
    error.value = e.message || 'Failed to save settings'
  } finally {
    saving.value = false
  }
}

// --- GitHub Auth Methods ---

async function detectGhCli() {
  ghAuthStatus.value = 'detecting'
  error.value = ''
  try {
    const result = await api.detectGhCli()
    if (result.success) {
      ghAuthStatus.value = 'authenticated'
      ghAuthUsername.value = result.username || ''
      ghAuthSource.value = result.source || 'gh_cli'
      success.value = `GitHub connected via gh CLI as ${result.username}`
    } else {
      ghAuthStatus.value = 'none'
      error.value = result.message || 'gh CLI token not found'
    }
  } catch (e: any) {
    ghAuthStatus.value = 'none'
    error.value = e.message || 'Failed to detect gh CLI'
  }
}

async function useManualToken() {
  const token = config.value.github.token
  if (!token) {
    error.value = 'Please enter a token first'
    return
  }
  ghAuthStatus.value = 'detecting'
  error.value = ''
  try {
    const result = await api.useManualGitHubToken(token)
    if (result.success) {
      ghAuthStatus.value = 'authenticated'
      ghAuthUsername.value = result.username || ''
      ghAuthSource.value = 'manual'
      config.value.github.username = result.username || ''
      success.value = `GitHub connected as ${result.username}`
    } else {
      ghAuthStatus.value = 'none'
      error.value = result.message || 'Token validation failed'
    }
  } catch (e: any) {
    ghAuthStatus.value = 'none'
    error.value = e.message || 'Failed to validate token'
  }
}

async function startDeviceCodeFlow() {
  const clientId = config.value.github.oauth_client_id
  if (!clientId) {
    error.value = 'Please enter a GitHub OAuth App Client ID first'
    return
  }

  error.value = ''
  try {
    const result = await api.startGitHubDeviceCode(clientId)
    ghDeviceCode.value = {
      user_code: result.user_code,
      verification_uri: result.verification_uri,
      device_code: result.device_code,
    }
    ghAuthStatus.value = 'device_code_pending'

    // Open verification URL in browser
    openExternal(result.verification_uri)

    // Start polling
    ghPollInterval = setInterval(async () => {
      try {
        const pollResult = await api.pollGitHubDeviceCode(clientId, result.device_code)
        if (pollResult.status === 'complete') {
          ghAuthStatus.value = 'authenticated'
          ghAuthUsername.value = pollResult.username || ''
          ghAuthSource.value = 'device_code'
          ghDeviceCode.value = null
          success.value = `GitHub connected as ${pollResult.username}`
          if (ghPollInterval) clearInterval(ghPollInterval)
        }
      } catch (e: any) {
        if (ghPollInterval) clearInterval(ghPollInterval)
        ghAuthStatus.value = 'none'
        error.value = e.message || 'Device code flow failed'
      }
    }, (result.interval || 5) * 1000)
  } catch (e: any) {
    error.value = e.message || 'Failed to start device code flow'
  }
}

// --- Calendar Auth Methods ---

function openExternal(url: string) {
  const tauri = (window as any).__TAURI_INTERNALS__
  if (tauri) {
    tauri.invoke('plugin:shell|open', { path: url, with: undefined })
  } else {
    window.open(url, '_blank')
  }
}

async function startCalendarAuth() {
  error.value = ''

  try {
    // Save the current calendar config first (so client_id/tenant_id are persisted)
    await api.updateConfig(config.value)

    const result = await api.startCalendarAuth(config.value.calendar.source)

    // Open the auth URL in the system browser
    openExternal(result.auth_url)

    calAuthStatus.value = 'pending'

    // Poll for auth completion
    calPollInterval = setInterval(async () => {
      try {
        const status = await api.getCalendarAuthStatus()
        if (status.connected) {
          calAuthStatus.value = 'authenticated'
          success.value = 'Microsoft 365 calendar connected'
          if (calPollInterval) clearInterval(calPollInterval)
        }
      } catch {
        // Ignore poll errors, keep trying
      }
    }, 2000)

    // Stop polling after 5 minutes
    setTimeout(() => {
      if (calPollInterval && calAuthStatus.value !== 'authenticated') {
        clearInterval(calPollInterval)
        calAuthStatus.value = 'none'
        error.value = 'Authentication timed out. Please try again.'
      }
    }, 300000)
  } catch (e: any) {
    error.value = e.message || 'Failed to start calendar auth'
  }
}

async function startCalendarDeviceCode() {
  error.value = ''

  try {
    // Save the current calendar config first (so client_id/tenant_id are persisted)
    await api.updateConfig(config.value)

    const result = await api.startCalendarDeviceCode(config.value.calendar.source)
    calDeviceCode.value = {
      user_code: result.user_code,
      verification_uri: result.verification_uri,
    }
    calAuthStatus.value = 'device_code_pending'

    // Open the verification URL
    openExternal(result.verification_uri)

    // Poll for completion
    const interval = (result.interval || 5) * 1000
    calPollInterval = setInterval(async () => {
      try {
        const pollResult = await api.pollCalendarDeviceCode()
        if (pollResult.status === 'completed') {
          calAuthStatus.value = 'authenticated'
          calDeviceCode.value = null
          success.value = 'Microsoft 365 calendar connected'
          if (calPollInterval) clearInterval(calPollInterval)
        } else if (pollResult.status === 'expired' || pollResult.status === 'error') {
          calAuthStatus.value = 'none'
          calDeviceCode.value = null
          error.value = pollResult.error || 'Device code flow failed'
          if (calPollInterval) clearInterval(calPollInterval)
        }
      } catch {
        // Ignore transient poll errors, keep trying
      }
    }, interval)

    // Stop polling after 15 minutes (device codes typically expire in 15min)
    setTimeout(() => {
      if (calPollInterval && calAuthStatus.value === 'device_code_pending') {
        clearInterval(calPollInterval)
        calAuthStatus.value = 'none'
        calDeviceCode.value = null
        error.value = 'Device code expired. Please try again.'
      }
    }, 900000)
  } catch (e: any) {
    error.value = e.message || 'Failed to start device code flow'
  }
}

async function startCalendarPasteCode() {
  error.value = ''

  try {
    // Save the current calendar config first
    await api.updateConfig(config.value)

    const result = await api.startCalendarAuth(config.value.calendar.source, 'manual')

    const tauri = (window as any).__TAURI_INTERNALS__
    const isOob = config.value.calendar.ms_redirect_uri === 'urn:ietf:wg:oauth:2.0:oob'

    if (tauri && isOob) {
      // Use Tauri webview window to intercept OOB redirect automatically
      await tauri.invoke('open_auth_window', { url: result.auth_url })
      calAuthStatus.value = 'pending'

      // Poll for auth completion (the webview will exchange the code via the server)
      calPollInterval = setInterval(async () => {
        try {
          const status = await api.getCalendarAuthStatus()
          if (status.connected) {
            calAuthStatus.value = 'authenticated'
            success.value = 'Microsoft 365 calendar connected'
            if (calPollInterval) clearInterval(calPollInterval)
          }
        } catch {
          // Ignore poll errors, keep trying
        }
      }, 2000)

      // Stop polling after 5 minutes
      setTimeout(() => {
        if (calPollInterval && calAuthStatus.value !== 'authenticated') {
          clearInterval(calPollInterval)
          calAuthStatus.value = 'none'
          error.value = 'Authentication timed out. Please try again.'
        }
      }, 300000)
    } else {
      // Fallback: open in system browser, show paste code input
      openExternal(result.auth_url)
      calAuthStatus.value = 'paste_code_pending'
      calManualCode.value = ''
    }
  } catch (e: any) {
    error.value = e.message || 'Failed to start auth flow'
  }
}

async function submitManualCode() {
  if (!calManualCode.value.trim()) {
    error.value = 'Please paste the authorization code or the full redirect URL'
    return
  }

  error.value = ''
  try {
    await api.exchangeCalendarCode(calManualCode.value.trim())
    calAuthStatus.value = 'authenticated'
    calManualCode.value = ''
    success.value = 'Microsoft 365 calendar connected'
  } catch (e: any) {
    error.value = e.message || 'Failed to exchange code'
  }
}

// Apply theme immediately when changed in the select
watch(() => config.value.general.theme, (newTheme) => {
  setTheme(newTheme as 'light' | 'dark' | 'system')
})
</script>

<template>
  <div class="p-6 max-w-3xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">Settings</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <form v-else @submit.prevent="save" class="space-y-8">
      <div v-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
      <div v-if="success" class="text-green-500 bg-green-500/10 p-4 rounded">{{ success }}</div>

      <!-- General -->
      <section class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">General</h2>
        <div class="grid gap-4">
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Theme</span>
            <select v-model="config.general.theme" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2">
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="system">System</option>
            </select>
          </label>
          <label class="flex items-center gap-2 text-sm text-[var(--color-text)]">
            <input type="checkbox" v-model="config.general.refresh_on_focus" class="rounded" />
            Refresh on window focus
          </label>
        </div>
      </section>

      <!-- GitHub -->
      <section class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">GitHub</h2>

        <!-- Auth status -->
        <div v-if="ghAuthStatus === 'authenticated'" class="mb-4 p-3 bg-green-500/10 rounded border border-green-500/30">
          <p class="text-sm text-green-600 dark:text-green-400">
            Connected as <strong>{{ ghAuthUsername }}</strong>
            <span class="text-xs ml-2">(via {{ ghAuthSource }})</span>
          </p>
        </div>

        <!-- Device code pending -->
        <div v-if="ghAuthStatus === 'device_code_pending' && ghDeviceCode" class="mb-4 p-4 bg-blue-500/10 rounded border border-blue-500/30">
          <p class="text-sm text-[var(--color-text)] mb-2">Enter this code on GitHub:</p>
          <p class="text-2xl font-mono font-bold text-[var(--color-primary)] mb-2">{{ ghDeviceCode.user_code }}</p>
          <p class="text-xs text-[var(--color-text-muted)]">
            A browser window should have opened. If not, go to:
            <a :href="ghDeviceCode.verification_uri" target="_blank" class="text-[var(--color-primary)] underline">{{ ghDeviceCode.verification_uri }}</a>
          </p>
          <p class="text-xs text-[var(--color-text-muted)] mt-2">Waiting for authorization...</p>
        </div>

        <!-- Auth buttons -->
        <div v-if="ghAuthStatus !== 'authenticated'" class="mb-4 space-y-3">
          <div class="flex flex-wrap gap-3">
            <button
              type="button"
              @click="detectGhCli"
              :disabled="ghAuthStatus === 'detecting'"
              class="px-4 py-2 rounded bg-[var(--color-surface-hover)] border border-[var(--color-border)] text-[var(--color-text)] text-sm hover:bg-[var(--color-border)] disabled:opacity-50"
            >
              {{ ghAuthStatus === 'detecting' ? 'Detecting...' : 'Use gh CLI token' }}
            </button>
            <button
              type="button"
              @click="startDeviceCodeFlow"
              :disabled="ghAuthStatus === 'device_code_pending'"
              class="px-4 py-2 rounded bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
            >
              Login with GitHub (Device Code)
            </button>
          </div>
          <div class="flex gap-2 items-center">
            <input v-model="config.github.token" type="password" placeholder="ghp_... or gho_..." class="flex-1 rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2 text-sm" />
            <button
              type="button"
              @click="useManualToken"
              :disabled="ghAuthStatus === 'detecting' || !config.github.token"
              class="px-4 py-2 rounded bg-[var(--color-surface-hover)] border border-[var(--color-border)] text-[var(--color-text)] text-sm hover:bg-[var(--color-border)] disabled:opacity-50 whitespace-nowrap"
            >
              Use Token
            </button>
          </div>
        </div>

        <div class="grid gap-4">
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">OAuth App Client ID (for device code flow)</span>
            <input v-model="config.github.oauth_client_id" type="text" placeholder="Ov23li..." class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
            <span class="text-xs text-[var(--color-text-muted)]">Create one at github.com/settings/developers</span>
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Username</span>
            <input v-model="config.github.username" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Repos (comma-separated, e.g. owner/repo)</span>
            <input v-model="config.github.repos" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Poll Interval (seconds)</span>
            <input v-model.number="config.github.poll_interval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
        </div>
      </section>

      <!-- JIRA -->
      <section class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">JIRA</h2>
        <div class="grid gap-4">
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Host (e.g. yourcompany.atlassian.net)</span>
            <input v-model="config.jira.host" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Email</span>
            <input v-model="config.jira.email" type="email" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">API Token</span>
            <input v-model="config.jira.api_token" type="password" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Project Keys (comma-separated)</span>
            <input v-model="config.jira.project_keys" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Poll Interval (seconds)</span>
            <input v-model.number="config.jira.poll_interval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
        </div>
      </section>

      <!-- GitLab -->
      <section class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">GitLab</h2>
        <div class="grid gap-4">
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Host (e.g. gitlab.com)</span>
            <input v-model="config.gitlab.host" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Token</span>
            <input v-model="config.gitlab.token" type="password" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Username</span>
            <input v-model="config.gitlab.username" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Projects (comma-separated paths, e.g. group/project)</span>
            <input v-model="config.gitlab.projects" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Poll Interval (seconds)</span>
            <input v-model.number="config.gitlab.poll_interval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
        </div>
      </section>

      <!-- Calendar -->
      <section class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">Calendar</h2>
        <div class="grid gap-4">
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Source</span>
            <select v-model="config.calendar.source" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2">
              <option value="none">Disabled</option>
              <option value="ics">ICS Feed URL</option>
              <option value="microsoft">Microsoft 365 (Graph API)</option>
              <option value="ews">Microsoft 365 (EWS)</option>
            </select>
          </label>

          <!-- ICS fields -->
          <template v-if="config.calendar.source === 'ics'">
            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">ICS Feed URL</span>
              <input v-model="config.calendar.ics_url" type="url" placeholder="https://outlook.office365.com/owa/calendar/..." class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                In Outlook: Settings &rarr; Calendar &rarr; Shared calendars &rarr; Publish a calendar &rarr; copy the ICS link.
                Works with Google Calendar, Apple Calendar, or any ICS feed.
              </span>
            </label>
          </template>

          <!-- Microsoft 365 fields -->
          <template v-if="config.calendar.source === 'microsoft' || config.calendar.source === 'ews'">
            <!-- Auth status -->
            <div v-if="calAuthStatus === 'authenticated'" class="p-3 bg-green-500/10 rounded border border-green-500/30">
              <p class="text-sm text-green-600 dark:text-green-400">
                Microsoft 365 calendar connected{{ config.calendar.source === 'ews' ? ' (via EWS)' : '' }}
              </p>
            </div>

            <!-- Waiting for redirect auth -->
            <div v-if="calAuthStatus === 'pending'" class="p-4 bg-blue-500/10 rounded border border-blue-500/30">
              <p class="text-sm text-[var(--color-text)]">A sign-in window has opened.</p>
              <p class="text-xs text-[var(--color-text-muted)] mt-1">Complete the sign-in in the browser window, then this page will update automatically.</p>
            </div>

            <!-- Device code pending -->
            <div v-if="calAuthStatus === 'device_code_pending' && calDeviceCode" class="p-4 bg-blue-500/10 rounded border border-blue-500/30">
              <p class="text-sm text-[var(--color-text)]">
                Go to <a :href="calDeviceCode.verification_uri" target="_blank" class="text-[var(--color-primary)] font-semibold underline">{{ calDeviceCode.verification_uri }}</a> and enter:
              </p>
              <p class="text-2xl font-mono font-bold text-[var(--color-text)] mt-2 tracking-widest">{{ calDeviceCode.user_code }}</p>
              <p class="text-xs text-[var(--color-text-muted)] mt-2">Waiting for sign-in to complete...</p>
            </div>

            <!-- Paste code pending -->
            <div v-if="calAuthStatus === 'paste_code_pending'" class="p-4 bg-blue-500/10 rounded border border-blue-500/30">
              <p class="text-sm text-[var(--color-text)] mb-2">Sign in completed? Paste the code or the full URL from the browser address bar:</p>
              <div class="flex gap-2">
                <input
                  v-model="calManualCode"
                  type="text"
                  placeholder="Paste code or URL here..."
                  class="flex-1 rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2 text-sm font-mono"
                  @keyup.enter="submitManualCode"
                />
                <button
                  type="button"
                  @click="submitManualCode"
                  class="px-4 py-2 rounded bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)]"
                >
                  Submit
                </button>
              </div>
              <p class="text-xs text-[var(--color-text-muted)] mt-2">
                After signing in, you'll be redirected to a blank page. Copy the full URL from the address bar (it contains the code).
              </p>
            </div>

            <!-- EWS info -->
            <div v-if="config.calendar.source === 'ews'" class="p-3 bg-blue-500/10 rounded border border-blue-500/30">
              <p class="text-xs text-[var(--color-text-muted)]">
                EWS uses a different permission scope than Graph API. If your admin blocked Graph calendar
                access, EWS may still work as it uses Exchange-level permissions instead.
              </p>
            </div>

            <!-- Connect buttons -->
            <div v-if="calAuthStatus !== 'authenticated' && calAuthStatus !== 'paste_code_pending'" class="flex gap-3 flex-wrap">
              <button
                type="button"
                @click="startCalendarAuth"
                :disabled="calAuthStatus === 'pending' || calAuthStatus === 'device_code_pending'"
                class="px-4 py-2 rounded bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
              >
                {{ calAuthStatus === 'pending' ? 'Waiting...' : 'Connect (Browser Redirect)' }}
              </button>
              <button
                type="button"
                @click="startCalendarPasteCode"
                :disabled="calAuthStatus === 'pending' || calAuthStatus === 'device_code_pending'"
                class="px-4 py-2 rounded border border-[var(--color-primary)] text-[var(--color-primary)] text-sm hover:bg-[var(--color-primary)]/10 disabled:opacity-50"
              >
                Connect (Paste Code)
              </button>
              <button
                type="button"
                @click="startCalendarDeviceCode"
                :disabled="calAuthStatus === 'pending' || calAuthStatus === 'device_code_pending'"
                class="px-4 py-2 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] text-sm hover:bg-[var(--color-surface-hover)] disabled:opacity-50"
              >
                {{ calAuthStatus === 'device_code_pending' ? 'Waiting...' : 'Connect (Device Code)' }}
              </button>
            </div>

            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">Client ID (optional — leave blank to use default)</span>
              <input v-model="config.calendar.ms_client_id" type="text" placeholder="Default: Azure CLI public client" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                Only needed if the default client is blocked by your admin.
              </span>
            </label>
            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">Tenant ID (optional)</span>
              <input v-model="config.calendar.ms_tenant_id" type="text" placeholder="common" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                Leave blank for multi-tenant ("common"). Set to your org's tenant ID if required.
              </span>
            </label>
            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">Redirect URI (optional)</span>
              <input v-model="config.calendar.ms_redirect_uri" type="text" placeholder="Default: https://login.microsoftonline.com/common/oauth2/nativeclient" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                Used by "Paste Code" flow. Default shows a blank page with the code in the URL bar.
                Do NOT use urn:ietf:wg:oauth:2.0:oob (browsers can't open it).
              </span>
            </label>
          </template>

          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Poll Interval (seconds)</span>
            <input v-model.number="config.calendar.poll_interval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
        </div>
      </section>

      <button type="submit" :disabled="saving" class="px-6 py-2 rounded bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] disabled:opacity-50">
        {{ saving ? 'Saving...' : 'Save Settings' }}
      </button>
    </form>
  </div>
</template>
