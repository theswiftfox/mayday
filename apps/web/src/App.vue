<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { onMounted } from 'vue'
import AppLayout from './components/AppLayout.vue'
import { useTheme } from './composables/useTheme'
import { useUpdateChecker } from './composables/useUpdateChecker'

// Initialize theme on app load
useTheme()

const {
  isTauri,
  updateAvailable,
  updateVersion,
  updateDismissed,
  downloading,
  downloadProgress,
  downloadTotal,
  checkForUpdate,
  downloadAndInstall,
  dismiss,
  getAutoCheckEnabled,
} = useUpdateChecker()

// Auto-check for updates on launch (if enabled)
onMounted(() => {
  if (isTauri && getAutoCheckEnabled()) {
    // Small delay so the app UI renders first
    setTimeout(() => checkForUpdate(), 3000)
  }
})
</script>

<template>
  <AppLayout>
    <!-- Update notification banner -->
    <div
      v-if="isTauri && updateAvailable && !updateDismissed && !downloading"
      class="mx-6 mt-4 flex items-center justify-between gap-4 rounded-lg border border-[var(--color-primary)]/30 bg-[var(--color-primary)]/10 px-4 py-3"
    >
      <p class="text-sm text-[var(--color-text)]">
        A new version (<span class="font-mono font-medium">{{ updateVersion }}</span>) is available.
      </p>
      <div class="flex items-center gap-2">
        <button
          @click="downloadAndInstall"
          class="px-3 py-1 text-sm rounded bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)]"
        >
          Update now
        </button>
        <button
          @click="dismiss"
          class="px-3 py-1 text-sm rounded border border-[var(--color-border)] text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]"
        >
          Later
        </button>
      </div>
    </div>

    <!-- Download progress banner -->
    <div
      v-if="isTauri && downloading"
      class="mx-6 mt-4 rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] px-4 py-3"
    >
      <p class="text-sm text-[var(--color-text)] mb-2">Downloading update...</p>
      <div class="w-full bg-[var(--color-border)] rounded-full h-1.5">
        <div
          class="bg-[var(--color-primary)] h-1.5 rounded-full transition-all duration-300"
          :style="{ width: downloadTotal > 0 ? `${Math.round((downloadProgress / downloadTotal) * 100)}%` : '0%' }"
        ></div>
      </div>
    </div>

    <router-view />
  </AppLayout>
</template>
