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

const CACHE_KEY = 'myday_dashboard_cache'

function loadFromCache(): { items: DashboardItem[]; lastUpdated: string | null } | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY)
    if (raw) {
      return JSON.parse(raw)
    }
  } catch {
    // Ignore corrupt cache
  }
  return null
}

function saveToCache(items: DashboardItem[], lastUpdated: string | null) {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ items, lastUpdated }))
  } catch {
    // localStorage full or unavailable — non-critical
  }
}

export const useDashboardStore = defineStore('dashboard', () => {
  const cached = loadFromCache()

  const items = ref<DashboardItem[]>(cached?.items ?? [])
  const errors = ref<IntegrationError[]>([])
  const lastUpdated = ref<string | null>(cached?.lastUpdated ?? null)
  // True only on first load when there's no data to show
  const loading = ref(false)
  // True during background refreshes (existing data stays visible)
  const refreshing = ref(false)

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
  // Pre-filtered pipelines for dashboard (non-success only)
  const failedPipelines = computed(() =>
    gitlabPipelines.value.filter((p) => p.status !== 'success')
  )

  // Actions
  async function fetchDashboard() {
    const isFirstLoad = items.value.length === 0
    if (isFirstLoad) {
      loading.value = true
    } else {
      refreshing.value = true
    }
    try {
      const response = await api.getDashboard()
      items.value = response.items as DashboardItem[]
      errors.value = response.errors
      lastUpdated.value = response.last_updated
      saveToCache(items.value, lastUpdated.value)
    } catch (e: any) {
      errors.value = [{ source: 'app', message: e.message }]
    } finally {
      loading.value = false
      refreshing.value = false
    }
  }

  return {
    items,
    errors,
    lastUpdated,
    loading,
    refreshing,
    githubPRs,
    jiraTickets,
    gitlabMRs,
    gitlabPipelines,
    calendarEvents,
    failedPipelines,
    fetchDashboard,
  }
})
