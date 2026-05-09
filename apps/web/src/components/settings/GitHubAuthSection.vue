<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onUnmounted } from 'vue'
import { api } from '@/lib/api'

const props = defineProps<{
  config: {
    token: string
    username: string
    repos: string
    pollInterval: number
    oauthClientId: string
  }
}>()

const emit = defineEmits<{
  (e: 'error', msg: string): void
  (e: 'success', msg: string): void
  (e: 'update:username', username: string): void
}>()

const authStatus = ref<'none' | 'detecting' | 'device_code_pending' | 'authenticated'>('none')
const authUsername = ref('')
const authSource = ref('')
const deviceCode = ref<{ userCode: string; verificationUri: string; deviceCode: string } | null>(null)
let pollInterval: ReturnType<typeof setInterval> | null = null

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
})

function setAuthenticated(username: string, source: string) {
  authStatus.value = 'authenticated'
  authUsername.value = username
  authSource.value = source
}

// Exposed for parent to call when loading saved state
defineExpose({ setAuthenticated })

function openExternal(url: string) {
  const tauri = (window as any).__TAURI_INTERNALS__
  if (tauri) {
    tauri.invoke('plugin:shell|open', { path: url, with: undefined })
  } else {
    window.open(url, '_blank')
  }
}

async function detectGhCli() {
  authStatus.value = 'detecting'
  emit('error', '')
  try {
    const result = await api.detectGhCli()
    if (result.success) {
      setAuthenticated(result.username || '', result.source || 'gh_cli')
      emit('success', `GitHub connected via gh CLI as ${result.username}`)
    } else {
      authStatus.value = 'none'
      emit('error', result.message || 'gh CLI token not found')
    }
  } catch (e: any) {
    authStatus.value = 'none'
    emit('error', e.message || 'Failed to detect gh CLI')
  }
}

async function useManualToken() {
  const token = props.config.token
  if (!token) {
    emit('error', 'Please enter a token first')
    return
  }
  authStatus.value = 'detecting'
  emit('error', '')
  try {
    const result = await api.useManualGitHubToken(token)
    if (result.success) {
      setAuthenticated(result.username || '', 'manual')
      emit('update:username', result.username || '')
      emit('success', `GitHub connected as ${result.username}`)
    } else {
      authStatus.value = 'none'
      emit('error', result.message || 'Token validation failed')
    }
  } catch (e: any) {
    authStatus.value = 'none'
    emit('error', e.message || 'Failed to validate token')
  }
}

async function startDeviceCodeFlow() {
  const clientId = props.config.oauthClientId
  if (!clientId) {
    emit('error', 'Please enter a GitHub OAuth App Client ID first')
    return
  }

  emit('error', '')
  try {
    const result = await api.startGitHubDeviceCode(clientId)
    deviceCode.value = {
      userCode: result.userCode,
      verificationUri: result.verificationUri,
      deviceCode: result.deviceCode,
    }
    authStatus.value = 'device_code_pending'

    openExternal(result.verificationUri)

    pollInterval = setInterval(async () => {
      try {
        const pollResult = await api.pollGitHubDeviceCode(clientId, result.deviceCode)
        if (pollResult.status === 'complete') {
          setAuthenticated(pollResult.username || '', 'device_code')
          deviceCode.value = null
          emit('success', `GitHub connected as ${pollResult.username}`)
          if (pollInterval) clearInterval(pollInterval)
        }
      } catch (e: any) {
        if (pollInterval) clearInterval(pollInterval)
        authStatus.value = 'none'
        emit('error', e.message || 'Device code flow failed')
      }
    }, (result.interval || 5) * 1000)
  } catch (e: any) {
    emit('error', e.message || 'Failed to start device code flow')
  }
}
</script>

<template>
  <!-- Auth status -->
  <div v-if="authStatus === 'authenticated'" class="mb-4 p-3 bg-green-500/10 rounded border border-green-500/30">
    <p class="text-sm text-green-600 dark:text-green-400">
      Connected as <strong>{{ authUsername }}</strong>
      <span class="text-xs ml-2">(via {{ authSource }})</span>
    </p>
  </div>

  <!-- Device code pending -->
  <div v-if="authStatus === 'device_code_pending' && deviceCode" class="mb-4 p-4 bg-blue-500/10 rounded border border-blue-500/30">
    <p class="text-sm text-[var(--color-text)] mb-2">Enter this code on GitHub:</p>
    <p class="text-2xl font-mono font-bold text-[var(--color-primary)] mb-2">{{ deviceCode.userCode }}</p>
    <p class="text-xs text-[var(--color-text-muted)]">
      A browser window should have opened. If not, go to:
      <a :href="deviceCode.verificationUri" target="_blank" rel="noopener noreferrer" class="text-[var(--color-primary)] underline">{{ deviceCode.verificationUri }}</a>
    </p>
    <p class="text-xs text-[var(--color-text-muted)] mt-2">Waiting for authorization...</p>
  </div>

  <!-- Auth buttons -->
  <div v-if="authStatus !== 'authenticated'" class="mb-4 space-y-3">
    <div class="flex flex-wrap gap-3">
      <button
        type="button"
        @click="detectGhCli"
        :disabled="authStatus === 'detecting'"
        class="px-4 py-2 rounded bg-[var(--color-surface-hover)] border border-[var(--color-border)] text-[var(--color-text)] text-sm hover:bg-[var(--color-border)] disabled:opacity-50"
      >
        {{ authStatus === 'detecting' ? 'Detecting...' : 'Use gh CLI token' }}
      </button>
      <button
        type="button"
        @click="startDeviceCodeFlow"
        :disabled="authStatus === 'device_code_pending'"
        class="px-4 py-2 rounded bg-[var(--color-primary)] text-white text-sm hover:bg-[var(--color-primary-hover)] disabled:opacity-50"
      >
        Login with GitHub (Device Code)
      </button>
    </div>
    <div class="flex gap-2 items-center">
      <input v-model="config.token" type="password" placeholder="ghp_... or gho_..." class="flex-1 rounded border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] px-3 py-2 text-sm" />
      <button
        type="button"
        @click="useManualToken"
        :disabled="authStatus === 'detecting' || !config.token"
        class="px-4 py-2 rounded bg-[var(--color-surface-hover)] border border-[var(--color-border)] text-[var(--color-text)] text-sm hover:bg-[var(--color-border)] disabled:opacity-50 whitespace-nowrap"
      >
        Use Token
      </button>
    </div>
  </div>
</template>
