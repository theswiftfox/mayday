// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { ref, readonly } from 'vue'

const STORAGE_KEY = 'myday-auto-update-check'
const isTauri = !!(window as any).__TAURI_INTERNALS__

/** Reactive state shared across all consumers */
const checking = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadTotal = ref(0)
const error = ref('')
const updateAvailable = ref(false)
const updateVersion = ref('')
const updateNotes = ref('')
const updateDismissed = ref(false)

/** Holds the update object returned by the check call */
let pendingUpdate: Awaited<ReturnType<typeof import('@tauri-apps/plugin-updater').check>> | null =
  null

async function checkForUpdate(): Promise<boolean> {
  if (!isTauri || checking.value) return false

  checking.value = true
  error.value = ''
  updateDismissed.value = false

  try {
    const { check } = await import('@tauri-apps/plugin-updater')
    const update = await check()

    if (update) {
      pendingUpdate = update
      updateAvailable.value = true
      updateVersion.value = update.version
      updateNotes.value = update.body ?? ''
      return true
    } else {
      updateAvailable.value = false
      pendingUpdate = null
      return false
    }
  } catch (e: any) {
    error.value = e.message || 'Failed to check for updates'
    return false
  } finally {
    checking.value = false
  }
}

async function downloadAndInstall(): Promise<void> {
  if (!pendingUpdate) {
    error.value = 'No pending update'
    return
  }

  downloading.value = true
  downloadProgress.value = 0
  downloadTotal.value = 0
  error.value = ''

  try {
    await pendingUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          downloadTotal.value = event.data.contentLength ?? 0
          downloadProgress.value = 0
          break
        case 'Progress':
          downloadProgress.value += event.data.chunkLength
          break
        case 'Finished':
          break
      }
    })

    // Relaunch after install
    const { relaunch } = await import('@tauri-apps/plugin-process')
    await relaunch()
  } catch (e: any) {
    error.value = e.message || 'Failed to install update'
    downloading.value = false
  }
}

function dismiss() {
  updateDismissed.value = true
}

function getAutoCheckEnabled(): boolean {
  const stored = localStorage.getItem(STORAGE_KEY)
  // Default to true if not set
  return stored === null ? true : stored === 'true'
}

function setAutoCheckEnabled(enabled: boolean) {
  localStorage.setItem(STORAGE_KEY, String(enabled))
}

export function useUpdateChecker() {
  return {
    // State (readonly to prevent external mutation)
    checking: readonly(checking),
    downloading: readonly(downloading),
    downloadProgress: readonly(downloadProgress),
    downloadTotal: readonly(downloadTotal),
    error: readonly(error),
    updateAvailable: readonly(updateAvailable),
    updateVersion: readonly(updateVersion),
    updateNotes: readonly(updateNotes),
    updateDismissed: readonly(updateDismissed),
    isTauri,

    // Actions
    checkForUpdate,
    downloadAndInstall,
    dismiss,
    getAutoCheckEnabled,
    setAutoCheckEnabled,
  }
}
