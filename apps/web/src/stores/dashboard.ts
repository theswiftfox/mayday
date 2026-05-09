// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/lib/api'
import { checkDemoMode } from '@/composables/useDemo'
import type {
  DashboardItem,
  IntegrationError,
  GitHubPR,
  JiraTicket,
  GitLabMR,
  GitLabPipeline,
  CalendarEvent,
} from '@/types/api'

export type { DashboardItem, IntegrationError }

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

  // Computed: group items by type in a single pass
  const grouped = computed(() => {
    const groups = {
      github_pr: [] as GitHubPR[],
      jira_ticket: [] as JiraTicket[],
      gitlab_mr: [] as GitLabMR[],
      gitlab_pipeline: [] as GitLabPipeline[],
      calendar_event: [] as CalendarEvent[],
    }
    for (const item of items.value) {
      if (item.type in groups) {
        (groups[item.type] as any[]).push(item.data)
      }
    }
    return groups
  })

  const githubPRs = computed(() => grouped.value.github_pr)
  const jiraTickets = computed(() => grouped.value.jira_ticket)
  const gitlabMRs = computed(() => grouped.value.gitlab_mr)
  const gitlabPipelines = computed(() => grouped.value.gitlab_pipeline)
  const calendarEvents = computed(() => grouped.value.calendar_event)
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
      if (checkDemoMode()) {
        const { demoDashboardItems, demoLastUpdated } = await import('@/lib/demo-data')
        items.value = demoDashboardItems
        errors.value = []
        lastUpdated.value = demoLastUpdated
      } else {
        const response = await api.getDashboard()
        items.value = response.items as DashboardItem[]
        errors.value = response.errors
        lastUpdated.value = response.lastUpdated
        saveToCache(items.value, lastUpdated.value)
      }
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
