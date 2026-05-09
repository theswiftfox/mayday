<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import { useFilteredGitHubPRs } from '@/composables/useFilteredItems'
import { useImportantItems } from '@/composables/useImportantItems'
import { useNow } from '@/composables/useNow'
import PRCard from '@/components/PRCard.vue'
import GitHubPRFilterPopover from '@/components/GitHubPRFilterPopover.vue'

const prs = ref<any[]>([])
const loading = ref(true)
const error = ref('')
const prefs = useDashboardPrefsStore()

onMounted(async () => {
  await prefs.load()
  try {
    const { data } = await api.getGitHubPRs()
    prs.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load GitHub PRs'
  } finally {
    loading.value = false
  }
})

// Apply shared filters
const filtered = useFilteredGitHubPRs(prs, computed(() => prefs.filters.githubPr))

// Split into important + rest
const now = useNow()
const split = useImportantItems(
  filtered,
  'github_pr',
  computed(() => prefs.importantRules),
  computed(() => prefs.pinnedItems),
  now
)

const importantPRs = computed(() => split.value.important)
const restPRs = computed(() => split.value.rest)

// Group rest by role
const reviewing = computed(() => restPRs.value.filter(pr => pr.role === 'reviewer'))
const authored = computed(() => restPRs.value.filter(pr => pr.role === 'author'))
const other = computed(() => restPRs.value.filter(pr => pr.role === 'other'))

// Group PRs by repo
function groupByRepo(prList: any[]) {
  const groups: Record<string, any[]> = {}
  for (const pr of prList) {
    if (!groups[pr.repo]) groups[pr.repo] = []
    groups[pr.repo].push(pr)
  }
  return Object.entries(groups).sort(([a], [b]) => a.localeCompare(b))
}

const reviewingByRepo = computed(() => groupByRepo(reviewing.value))
const authoredByRepo = computed(() => groupByRepo(authored.value))
const otherByRepo = computed(() => groupByRepo(other.value))
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">GitHub Pull Requests</h1>
      <div class="ml-auto">
        <GitHubPRFilterPopover />
      </div>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <!-- Important -->
      <section v-if="importantPRs.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-warning)">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          Important
          <span class="text-xs px-1.5 py-0.5 rounded-full font-medium" style="background: var(--color-surface-hover); color: var(--color-text-muted)">{{ importantPRs.length }}</span>
        </h2>
        <div class="space-y-2">
          <PRCard v-for="pr in importantPRs" :key="pr.id" :pr="pr" show-pin />
        </div>
      </section>

      <!-- Reviewing -->
      <section v-if="reviewing.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Reviewing</h2>
        <div v-for="[repo, repoPrs] in reviewingByRepo" :key="repo" class="mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-muted)] mb-2">{{ repo }}</h3>
          <div class="space-y-2">
            <PRCard v-for="pr in repoPrs" :key="pr.id" :pr="pr" show-pin />
          </div>
        </div>
      </section>

      <!-- Authored -->
      <section v-if="authored.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Authored</h2>
        <div v-for="[repo, repoPrs] in authoredByRepo" :key="repo" class="mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-muted)] mb-2">{{ repo }}</h3>
          <div class="space-y-2">
            <PRCard v-for="pr in repoPrs" :key="pr.id" :pr="pr" show-pin />
          </div>
        </div>
      </section>

      <!-- Other open PRs -->
      <section v-if="other.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Other</h2>
        <div v-for="[repo, repoPrs] in otherByRepo" :key="repo" class="mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-muted)] mb-2">{{ repo }}</h3>
          <div class="space-y-2">
            <PRCard v-for="pr in repoPrs" :key="pr.id" :pr="pr" show-pin />
          </div>
        </div>
      </section>

      <p v-if="!filtered.length" class="text-[var(--color-text-muted)]">No pull requests found.</p>
    </div>
  </div>
</template>
