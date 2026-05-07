<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import PRCard from '@/components/PRCard.vue'

const prs = ref<any[]>([])
const loading = ref(true)
const error = ref('')

const reviewing = computed(() => prs.value.filter(pr => pr.role === 'reviewer'))
const authored = computed(() => prs.value.filter(pr => pr.role === 'author'))
const other = computed(() => prs.value.filter(pr => pr.role === 'other'))

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

onMounted(async () => {
  try {
    const { data } = await api.getGitHubPRs()
    prs.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load GitHub PRs'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">GitHub Pull Requests</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <!-- Reviewing -->
      <section v-if="reviewing.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Reviewing</h2>
        <div v-for="[repo, repoPrs] in reviewingByRepo" :key="repo" class="mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-muted)] mb-2">{{ repo }}</h3>
          <div class="space-y-2">
            <PRCard v-for="pr in repoPrs" :key="pr.id" :pr="pr" />
          </div>
        </div>
      </section>

      <!-- Authored -->
      <section v-if="authored.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Authored</h2>
        <div v-for="[repo, repoPrs] in authoredByRepo" :key="repo" class="mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-muted)] mb-2">{{ repo }}</h3>
          <div class="space-y-2">
            <PRCard v-for="pr in repoPrs" :key="pr.id" :pr="pr" />
          </div>
        </div>
      </section>

      <!-- Other open PRs -->
      <section v-if="other.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Other</h2>
        <div v-for="[repo, repoPrs] in otherByRepo" :key="repo" class="mb-4">
          <h3 class="text-sm font-medium text-[var(--color-text-muted)] mb-2">{{ repo }}</h3>
          <div class="space-y-2">
            <PRCard v-for="pr in repoPrs" :key="pr.id" :pr="pr" />
          </div>
        </div>
      </section>

      <p v-if="!prs.length" class="text-[var(--color-text-muted)]">No pull requests found.</p>
    </div>
  </div>
</template>
