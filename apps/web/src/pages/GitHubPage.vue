<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import PRCard from '@/components/PRCard.vue'

const prs = ref<any[]>([])
const loading = ref(true)
const error = ref('')

const authored = computed(() => prs.value.filter(pr => pr.role === 'author'))
const reviewing = computed(() => prs.value.filter(pr => pr.role === 'reviewer'))

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
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">← Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">GitHub Pull Requests</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <section v-if="authored.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Authored</h2>
        <div class="space-y-3">
          <PRCard v-for="pr in authored" :key="pr.id" :pr="pr" />
        </div>
      </section>

      <section v-if="reviewing.length">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Reviewing</h2>
        <div class="space-y-3">
          <PRCard v-for="pr in reviewing" :key="pr.id" :pr="pr" />
        </div>
      </section>

      <p v-if="!prs.length" class="text-[var(--color-text-muted)]">No pull requests found.</p>
    </div>
  </div>
</template>
