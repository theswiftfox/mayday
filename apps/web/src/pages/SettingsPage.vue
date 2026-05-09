<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { api } from '@/lib/api'
import { useTheme } from '@/composables/useTheme'
import { useUpdateChecker } from '@/composables/useUpdateChecker'
import GitHubAuthSection from '@/components/settings/GitHubAuthSection.vue'
import CalendarAuthSection from '@/components/settings/CalendarAuthSection.vue'

const { setTheme } = useTheme()
const {
  checking: updateChecking,
  downloading: updateDownloading,
  downloadProgress,
  downloadTotal,
  error: updateError,
  updateAvailable,
  updateVersion,
  updateNotes,
  isTauri,
  checkForUpdate,
  downloadAndInstall,
  getAutoCheckEnabled,
  setAutoCheckEnabled,
} = useUpdateChecker()

const autoCheckUpdates = ref(getAutoCheckEnabled())
watch(autoCheckUpdates, (v) => setAutoCheckEnabled(v))

const appVersion = ref('')
onMounted(async () => {
  if (isTauri) {
    try {
      const { getVersion } = await import('@tauri-apps/api/app')
      appVersion.value = await getVersion()
    } catch {
      // Ignore — version not critical
    }
  }
})
const loading = ref(true)
const saving = ref(false)
const error = ref('')
const success = ref('')

// Template refs for auth sub-components
const ghAuthRef = ref<InstanceType<typeof GitHubAuthSection> | null>(null)
const calAuthRef = ref<InstanceType<typeof CalendarAuthSection> | null>(null)

const config = ref({
  github: { token: '', username: '', repos: '', pollInterval: 300, oauthClientId: '' },
  jira: { host: '', email: '', apiToken: '', projectKeys: '', pollInterval: 300 },
  gitlab: { host: '', token: '', username: '', projects: '' as string, pollInterval: 300 },
  calendar: { source: 'ics', icsUrl: '', msClientId: '', msTenantId: '', msRedirectUri: '', pollInterval: 300 },
  general: { theme: 'system', refreshOnFocus: true },
})

onMounted(async () => {
  try {
    const data = await api.getConfig()
    if (data) {
      // Map API response to form fields
      if (data.github) {
        config.value.github.username = data.github.username || ''
        config.value.github.repos = (data.github.repos || []).join(', ')
        config.value.github.pollInterval = data.github.pollIntervalSecs || 300
        config.value.github.oauthClientId = data.github.oauthClientId || ''
        if (data.github.hasToken) {
          // Defer to next tick so the ref is mounted
          setTimeout(() => {
            ghAuthRef.value?.setAuthenticated(
              data.github.username || '',
              data.github.tokenSource || 'manual',
            )
          })
        }
      }
      if (data.jira) {
        config.value.jira.host = data.jira.host || ''
        config.value.jira.email = data.jira.email || ''
        config.value.jira.projectKeys = (data.jira.projectKeys || []).join(', ')
        config.value.jira.pollInterval = data.jira.pollIntervalSecs || 300
      }
      if (data.gitlab) {
        config.value.gitlab.host = data.gitlab.host || ''
        config.value.gitlab.username = data.gitlab.username || ''
        config.value.gitlab.projects = (data.gitlab.projects || []).map((p: any) => p.path).join(', ')
        config.value.gitlab.pollInterval = data.gitlab.pollIntervalSecs || 300
      }
      if (data.calendar) {
        config.value.calendar.source = data.calendar.source || 'ics'
        config.value.calendar.icsUrl = data.calendar.icsUrl || ''
        config.value.calendar.msClientId = data.calendar.msClientId || ''
        config.value.calendar.msTenantId = data.calendar.msTenantId || ''
        config.value.calendar.msRedirectUri = data.calendar.msRedirectUri || ''
        config.value.calendar.pollInterval = data.calendar.pollIntervalSecs || 300
        if (data.calendar.hasMsRefreshToken) {
          setTimeout(() => {
            calAuthRef.value?.setAuthenticated()
          })
        }
      }
      if (data.general) {
        config.value.general = {
          ...config.value.general,
          ...data.general,
        }
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
    delete gitlabPayload.projectIds

    await api.updateConfig({ ...config.value, gitlab: gitlabPayload })
    success.value = 'Settings saved successfully'
  } catch (e: any) {
    error.value = e.message || 'Failed to save settings'
  } finally {
    saving.value = false
  }
}

function onAuthError(msg: string) {
  error.value = msg
}

function onAuthSuccess(msg: string) {
  success.value = msg
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
            <input type="checkbox" v-model="config.general.refreshOnFocus" class="rounded" />
            Refresh on window focus
          </label>
        </div>
      </section>

      <!-- Updates (Tauri only) -->
      <section v-if="isTauri" class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">Updates</h2>
        <div class="grid gap-4">
          <div class="flex items-center justify-between">
            <div>
              <span v-if="appVersion" class="text-sm text-[var(--color-text-muted)]">
                Current version: <span class="font-mono text-[var(--color-text)]">{{ appVersion }}</span>
              </span>
            </div>
            <button
              type="button"
              :disabled="updateChecking || updateDownloading"
              @click="checkForUpdate"
              class="px-4 py-1.5 text-sm rounded border border-[var(--color-border)] text-[var(--color-text)] hover:bg-[var(--color-surface-hover)] disabled:opacity-50"
            >
              {{ updateChecking ? 'Checking...' : 'Check for updates' }}
            </button>
          </div>

          <label class="flex items-center gap-2 text-sm text-[var(--color-text)]">
            <input type="checkbox" v-model="autoCheckUpdates" class="rounded" />
            Check for updates automatically on launch
          </label>

          <div v-if="updateError" class="text-red-500 bg-red-500/10 p-3 rounded text-sm">
            {{ updateError }}
          </div>

          <div v-if="updateAvailable && !updateDownloading" class="bg-[var(--color-primary)]/10 border border-[var(--color-primary)]/30 p-4 rounded">
            <p class="text-sm font-medium text-[var(--color-text)]">
              Version {{ updateVersion }} is available
            </p>
            <p v-if="updateNotes" class="text-xs text-[var(--color-text-muted)] mt-1">{{ updateNotes }}</p>
            <button
              type="button"
              @click="downloadAndInstall"
              class="mt-3 px-4 py-1.5 text-sm rounded bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)]"
            >
              Download and install
            </button>
          </div>

          <div v-if="updateDownloading" class="p-4 rounded bg-[var(--color-surface-hover)]">
            <p class="text-sm text-[var(--color-text)] mb-2">Downloading update...</p>
            <div class="w-full bg-[var(--color-border)] rounded-full h-2">
              <div
                class="bg-[var(--color-primary)] h-2 rounded-full transition-all duration-300"
                :style="{ width: downloadTotal > 0 ? `${Math.round((downloadProgress / downloadTotal) * 100)}%` : '0%' }"
              ></div>
            </div>
            <p v-if="downloadTotal > 0" class="text-xs text-[var(--color-text-muted)] mt-1">
              {{ Math.round(downloadProgress / 1024) }} / {{ Math.round(downloadTotal / 1024) }} KB
            </p>
          </div>
        </div>
      </section>

      <!-- GitHub -->
      <section class="p-5 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-4">GitHub</h2>

        <GitHubAuthSection
          ref="ghAuthRef"
          :config="config.github"
          @error="onAuthError"
          @success="onAuthSuccess"
          @update:username="config.github.username = $event"
        />

        <div class="grid gap-4">
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">OAuth App Client ID (for device code flow)</span>
            <input v-model="config.github.oauthClientId" type="text" placeholder="Ov23li..." class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
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
            <input v-model.number="config.github.pollInterval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
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
            <input v-model="config.jira.apiToken" type="password" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Project Keys (comma-separated)</span>
            <input v-model="config.jira.projectKeys" type="text" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Poll Interval (seconds)</span>
            <input v-model.number="config.jira.pollInterval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
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
            <input v-model.number="config.gitlab.pollInterval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
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
              <input v-model="config.calendar.icsUrl" type="url" placeholder="https://outlook.office365.com/owa/calendar/..." class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                In Outlook: Settings &rarr; Calendar &rarr; Shared calendars &rarr; Publish a calendar &rarr; copy the ICS link.
                Works with Google Calendar, Apple Calendar, or any ICS feed.
              </span>
            </label>
          </template>

          <!-- Microsoft 365 fields -->
          <template v-if="config.calendar.source === 'microsoft' || config.calendar.source === 'ews'">
            <CalendarAuthSection
              ref="calAuthRef"
              :config="config.calendar"
              :fullConfig="config"
              @error="onAuthError"
              @success="onAuthSuccess"
            />

            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">Client ID (optional — leave blank to use default)</span>
              <input v-model="config.calendar.msClientId" type="text" placeholder="Default: Azure CLI public client" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                Only needed if the default client is blocked by your admin.
              </span>
            </label>
            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">Tenant ID (optional)</span>
              <input v-model="config.calendar.msTenantId" type="text" placeholder="common" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                Leave blank for multi-tenant ("common"). Set to your org's tenant ID if required.
              </span>
            </label>
            <label class="block">
              <span class="text-sm text-[var(--color-text-muted)]">Redirect URI (optional)</span>
              <input v-model="config.calendar.msRedirectUri" type="text" placeholder="Default: https://login.microsoftonline.com/common/oauth2/nativeclient" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
              <span class="text-xs text-[var(--color-text-muted)] mt-1 block">
                Used by "Paste Code" flow. Default shows a blank page with the code in the URL bar.
                Do NOT use urn:ietf:wg:oauth:2.0:oob (browsers can't open it).
              </span>
            </label>
          </template>

          <label class="block">
            <span class="text-sm text-[var(--color-text-muted)]">Poll Interval (seconds)</span>
            <input v-model.number="config.calendar.pollInterval" type="number" class="mt-1 block w-full rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2" />
          </label>
        </div>
      </section>

      <button type="submit" :disabled="saving" class="px-6 py-2 rounded bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] disabled:opacity-50">
        {{ saving ? 'Saving...' : 'Save Settings' }}
      </button>
    </form>
  </div>
</template>
