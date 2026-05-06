import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/lib/api'

export interface DashboardItem {
  type: 'github_pr' | 'jira_ticket' | 'gitlab_mr' | 'gitlab_pipeline' | 'calendar_event'
  data: any
}

export interface IntegrationError {
  source: string
  message: string
}

export const useDashboardStore = defineStore('dashboard', () => {
  const items = ref<DashboardItem[]>([])
  const errors = ref<IntegrationError[]>([])
  const lastUpdated = ref<string | null>(null)
  const loading = ref(false)

  // Computed: items by type
  const githubPRs = computed(() =>
    items.value.filter((i) => i.type === 'github_pr').map((i) => i.data)
  )
  const jiraTickets = computed(() =>
    items.value.filter((i) => i.type === 'jira_ticket').map((i) => i.data)
  )
  const gitlabMRs = computed(() =>
    items.value.filter((i) => i.type === 'gitlab_mr').map((i) => i.data)
  )
  const gitlabPipelines = computed(() =>
    items.value.filter((i) => i.type === 'gitlab_pipeline').map((i) => i.data)
  )
  const calendarEvents = computed(() =>
    items.value.filter((i) => i.type === 'calendar_event').map((i) => i.data)
  )

  // Actions
  async function fetchDashboard() {
    loading.value = true
    try {
      const response = await api.getDashboard()
      items.value = response.items as DashboardItem[]
      errors.value = response.errors
      lastUpdated.value = response.last_updated
    } catch (e: any) {
      errors.value = [{ source: 'app', message: e.message }]
    } finally {
      loading.value = false
    }
  }

  return {
    items,
    errors,
    lastUpdated,
    loading,
    githubPRs,
    jiraTickets,
    gitlabMRs,
    gitlabPipelines,
    calendarEvents,
    fetchDashboard,
  }
})
