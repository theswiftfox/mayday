<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { api } from '@/lib/api'

const props = defineProps<{
  config: {
    source: string
    icsUrl: string
    msClientId: string
    msTenantId: string
    msRedirectUri: string
    pollInterval: number
  }
  fullConfig: any
}>()

const emit = defineEmits<{
  (e: 'error', msg: string): void
  (e: 'success', msg: string): void
}>()

const authStatus = ref<'none' | 'pending' | 'device_code_pending' | 'paste_code_pending' | 'authenticated'>('none')
const deviceCode = ref<{ userCode: string; verificationUri: string } | null>(null)
const manualCode = ref('')
let pollInterval: ReturnType<typeof setInterval> | null = null
const timeouts: ReturnType<typeof setTimeout>[] = []

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
  for (const t of timeouts) clearTimeout(t)
})

function setAuthenticated() {
  authStatus.value = 'authenticated'
}

defineExpose({ setAuthenticated })

function openExternal(url: string) {
  const tauri = (window as any).__TAURI_INTERNALS__
  if (tauri) {
    tauri.invoke('plugin:shell|open', { path: url, with: undefined })
  } else {
    window.open(url, '_blank')
  }
}

async function startCalendarAuth() {
  emit('error', '')
  try {
    await api.updateConfig(props.fullConfig)
    const result = await api.startCalendarAuth(props.config.source)
    openExternal(result.authUrl)
    authStatus.value = 'pending'

    pollInterval = setInterval(async () => {
      try {
        const status = await api.getCalendarAuthStatus()
        if (status.connected) {
          authStatus.value = 'authenticated'
          emit('success', 'Microsoft 365 calendar connected')
          if (pollInterval) clearInterval(pollInterval)
        }
      } catch {
        // Ignore poll errors
      }
    }, 2000)

    timeouts.push(setTimeout(() => {
      if (pollInterval && authStatus.value !== 'authenticated') {
        clearInterval(pollInterval)
        authStatus.value = 'none'
        emit('error', 'Authentication timed out. Please try again.')
      }
    }, 300000))
  } catch (e: any) {
    emit('error', e.message || 'Failed to start calendar auth')
  }
}

async function startCalendarDeviceCode() {
  emit('error', '')
  try {
    await api.updateConfig(props.fullConfig)
    const result = await api.startCalendarDeviceCode(props.config.source)
    deviceCode.value = {
      userCode: result.userCode,
      verificationUri: result.verificationUri,
    }
    authStatus.value = 'device_code_pending'
    openExternal(result.verificationUri)

    const interval = (result.interval || 5) * 1000
    pollInterval = setInterval(async () => {
      try {
        const pollResult = await api.pollCalendarDeviceCode()
        if (pollResult.status === 'completed') {
          authStatus.value = 'authenticated'
          deviceCode.value = null
          emit('success', 'Microsoft 365 calendar connected')
          if (pollInterval) clearInterval(pollInterval)
        } else if (pollResult.status === 'expired' || pollResult.status === 'error') {
          authStatus.value = 'none'
          deviceCode.value = null
          emit('error', pollResult.error || 'Device code flow failed')
          if (pollInterval) clearInterval(pollInterval)
        }
      } catch {
        // Ignore transient poll errors
      }
    }, interval)

    timeouts.push(setTimeout(() => {
      if (pollInterval && authStatus.value === 'device_code_pending') {
        clearInterval(pollInterval)
        authStatus.value = 'none'
        deviceCode.value = null
        emit('error', 'Device code expired. Please try again.')
      }
    }, 900000))
  } catch (e: any) {
    emit('error', e.message || 'Failed to start device code flow')
  }
}

async function startCalendarPasteCode() {
  emit('error', '')
  try {
    await api.updateConfig(props.fullConfig)
    const result = await api.startCalendarAuth(props.config.source, 'manual')

    const tauri = (window as any).__TAURI_INTERNALS__
    const isOob = props.config.msRedirectUri === 'urn:ietf:wg:oauth:2.0:oob'

    if (tauri && isOob) {
      await tauri.invoke('open_auth_window', { url: result.authUrl })
      authStatus.value = 'pending'

      pollInterval = setInterval(async () => {
        try {
          const status = await api.getCalendarAuthStatus()
          if (status.connected) {
            authStatus.value = 'authenticated'
            emit('success', 'Microsoft 365 calendar connected')
            if (pollInterval) clearInterval(pollInterval)
          }
        } catch {
          // Ignore poll errors
        }
      }, 2000)

      timeouts.push(setTimeout(() => {
        if (pollInterval && authStatus.value !== 'authenticated') {
          clearInterval(pollInterval)
          authStatus.value = 'none'
          emit('error', 'Authentication timed out. Please try again.')
        }
      }, 300000))
    } else {
      openExternal(result.authUrl)
      authStatus.value = 'paste_code_pending'
      manualCode.value = ''
    }
  } catch (e: any) {
    emit('error', e.message || 'Failed to start auth flow')
  }
}

async function submitManualCode() {
  if (!manualCode.value.trim()) {
    emit('error', 'Please paste the authorization code or the full redirect URL')
    return
  }
  emit('error', '')
  try {
    await api.exchangeCalendarCode(manualCode.value.trim())
    authStatus.value = 'authenticated'
    manualCode.value = ''
    emit('success', 'Microsoft 365 calendar connected')
  } catch (e: any) {
    emit('error', e.message || 'Failed to exchange code')
  }
}
</script>

<template>
  <!-- Auth status -->
  <div v-if="authStatus === 'authenticated'" class="p-3 bg-green-500/10 rounded border border-green-500/30">
    <p class="text-sm text-green-600 dark:text-green-400">
      Microsoft 365 calendar connected{{ config.source === 'ews' ? ' (via EWS)' : '' }}
    </p>
  </div>

  <!-- Waiting for redirect auth -->
  <div v-if="authStatus === 'pending'" class="p-4 bg-blue-500/10 rounded border border-blue-500/30">
    <p class="text-sm text-[var(--color-text)]">A sign-in window has opened.</p>
    <p class="text-xs text-[var(--color-text-muted)] mt-1">Complete the sign-in in the browser window, then this page will update automatically.</p>
  </div>

  <!-- Device code pending -->
  <div v-if="authStatus === 'device_code_pending' && deviceCode" class="p-4 bg-blue-500/10 rounded border border-blue-500/30">
    <p class="text-sm text-[var(--color-text)]">
      Go to <a :href="deviceCode.verificationUri" target="_blank" rel="noopener noreferrer" class="text-[var(--color-primary)] font-semibold underline">{{ deviceCode.verificationUri }}</a> and enter:
    </p>
    <p class="text-2xl font-mono font-bold text-[var(--color-text)] mt-2 tracking-widest">{{ deviceCode.userCode }}</p>
    <p class="text-xs text-[var(--color-text-muted)] mt-2">Waiting for sign-in to complete...</p>
  </div>

  <!-- Paste code pending -->
  <div v-if="authStatus === 'paste_code_pending'" class="p-4 bg-blue-500/10 rounded border border-blue-500/30">
    <p class="text-sm text-[var(--color-text)] mb-2">Sign in completed? Paste the code or the full URL from the browser address bar:</p>
    <div class="flex gap-2">
      <input
        v-model="manualCode"
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
  <div v-if="config.source === 'ews'" class="p-3 bg-blue-500/10 rounded border border-blue-500/30">
    <p class="text-xs text-[var(--color-text-muted)]">
      EWS uses a different permission scope than Graph API. If your admin blocked Graph calendar
      access, EWS may still work as it uses Exchange-level permissions instead.
    </p>
  </div>

  <!-- Connect buttons -->
  <div v-if="authStatus !== 'authenticated' && authStatus !== 'paste_code_pending'" class="flex gap-3 flex-wrap">
    <button
      type="button"
      @click="startCalendarAuth"
      :disabled="authStatus === 'pending' || authStatus === 'device_code_pending'"
      class="px-4 py-2 rounded bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
    >
      {{ authStatus === 'pending' ? 'Waiting...' : 'Connect (Browser Redirect)' }}
    </button>
    <button
      type="button"
      @click="startCalendarPasteCode"
      :disabled="authStatus === 'pending' || authStatus === 'device_code_pending'"
      class="px-4 py-2 rounded border border-[var(--color-primary)] text-[var(--color-primary)] text-sm hover:bg-[var(--color-primary)]/10 disabled:opacity-50"
    >
      Connect (Paste Code)
    </button>
    <button
      type="button"
      @click="startCalendarDeviceCode"
      :disabled="authStatus === 'pending' || authStatus === 'device_code_pending'"
      class="px-4 py-2 rounded border border-[var(--color-border)] text-[var(--color-text-muted)] text-sm hover:bg-[var(--color-surface-hover)] disabled:opacity-50"
    >
      {{ authStatus === 'device_code_pending' ? 'Waiting...' : 'Connect (Device Code)' }}
    </button>
  </div>
</template>
