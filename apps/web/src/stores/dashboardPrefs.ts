// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/lib/api'
import type {
  DashboardConfig,
  SectionType,
  ImportantRules,
  DashboardFilters,
  GitHubPRFilter,
  GitLabMRFilter,
  GitLabPipelineFilter,
  JiraTicketFilter,
  CalendarEventFilter,
} from '@/types/dashboard'
import { defaultDashboardConfig, getItemId } from '@/types/dashboard'

export const useDashboardPrefsStore = defineStore('dashboardPrefs', () => {
  const config = ref<DashboardConfig>(defaultDashboardConfig())
  const loaded = ref(false)
  const saving = ref(false)

  // Debounce helper for persist — avoids rapid-fire saves during drag-reorder, etc.
  let persistTimer: ReturnType<typeof setTimeout> | null = null
  const PERSIST_DELAY = 500

  // ---- Getters ----

  const sectionOrder = computed(() => config.value.sectionOrder)
  const visibleSections = computed(() => config.value.visibleSections)
  const calendarLayout = computed(() => config.value.calendarLayout)
  const importantRules = computed(() => config.value.importantRules)
  const pinnedItems = computed(() => config.value.pinnedItems)
  const filters = computed(() => config.value.filters)

  // ---- Persistence helpers ----

  /** Load dashboard prefs from the server config */
  async function load() {
    try {
      const dashboard = await api.getDashboardConfig()
      if (dashboard && Object.keys(dashboard).length > 0) {
        const defaults = defaultDashboardConfig()
        // Deep merge: spread nested objects so missing keys get defaults
        config.value = {
          ...defaults,
          ...dashboard,
          importantRules: { ...defaults.importantRules, ...dashboard.importantRules },
          filters: {
            githubPr: { ...defaults.filters.githubPr, ...dashboard.filters?.githubPr },
            gitlabMr: { ...defaults.filters.gitlabMr, ...dashboard.filters?.gitlabMr },
            gitlabPipeline: { ...defaults.filters.gitlabPipeline, ...dashboard.filters?.gitlabPipeline },
            jiraTicket: { ...defaults.filters.jiraTicket, ...dashboard.filters?.jiraTicket },
            calendarEvent: { ...defaults.filters.calendarEvent, ...dashboard.filters?.calendarEvent },
          },
          pinnedItems: dashboard.pinnedItems ?? defaults.pinnedItems,
        }
        // Migrate old gitlab_mr/gitlab_pipeline section entries to 'gitlab'
        migrateLegacyGitlabSections()
      }
      loaded.value = true
    } catch {
      // Use defaults on error
      loaded.value = true
    }
  }

  /** Collapse old gitlab_mr / gitlab_pipeline entries into 'gitlab' */
  function migrateLegacyGitlabSections() {
    let dirty = false
    for (const key of ['sectionOrder', 'visibleSections'] as const) {
      const arr = config.value[key] as string[]
      const hasMr = arr.includes('gitlab_mr' as any)
      const hasPipeline = arr.includes('gitlab_pipeline' as any)
      const hasGitlab = arr.includes('gitlab')
      if (hasMr || hasPipeline) {
        // Insert 'gitlab' at position of first legacy entry if not already present
        if (!hasGitlab) {
          const idx = Math.min(
            hasMr ? arr.indexOf('gitlab_mr' as any) : Infinity,
            hasPipeline ? arr.indexOf('gitlab_pipeline' as any) : Infinity,
          )
          arr.splice(idx, 0, 'gitlab' as any)
        }
        // Remove legacy entries
        config.value[key] = arr.filter(
          (s) => s !== ('gitlab_mr' as any) && s !== ('gitlab_pipeline' as any)
        ) as SectionType[]
        dirty = true
      }
    }
    if (dirty) persistNow()
  }

  /** Persist the current dashboard prefs to the server config (debounced) */
  function persist() {
    if (persistTimer) clearTimeout(persistTimer)
    persistTimer = setTimeout(async () => {
      saving.value = true
      try {
        await api.updateDashboardConfig(config.value)
      } catch {
        // Silently fail — prefs are still in memory
      } finally {
        saving.value = false
      }
    }, PERSIST_DELAY)
  }

  /** Persist immediately without debounce (for use during migration) */
  async function persistNow() {
    if (persistTimer) clearTimeout(persistTimer)
    saving.value = true
    try {
      await api.updateDashboardConfig(config.value)
    } catch {
      // Silently fail
    } finally {
      saving.value = false
    }
  }

  // ---- Actions ----

  function setSectionOrder(order: SectionType[]) {
    config.value.sectionOrder = order
    persist()
  }

  function setCalendarLayout(layout: 'sidebar' | 'inline') {
    config.value.calendarLayout = layout
    persist()
  }

  function setVisibleSections(sections: SectionType[]) {
    config.value.visibleSections = sections
    persist()
  }

  function toggleSectionVisibility(section: SectionType) {
    const idx = config.value.visibleSections.indexOf(section)
    if (idx >= 0) {
      config.value.visibleSections.splice(idx, 1)
    } else {
      config.value.visibleSections.push(section)
    }
    persist()
  }

  function isSectionVisible(section: SectionType): boolean {
    return config.value.visibleSections.includes(section)
  }

  function setImportantRules(rules: ImportantRules) {
    config.value.importantRules = rules
    persist()
  }

  function updateImportantRule(key: keyof ImportantRules, value: boolean) {
    config.value.importantRules[key] = value
    persist()
  }

  function setFilters(filters: DashboardFilters) {
    config.value.filters = filters
    persist()
  }

  function updateGitHubPRFilter(filter: Partial<GitHubPRFilter>) {
    config.value.filters.githubPr = { ...config.value.filters.githubPr, ...filter }
    persist()
  }

  function updateGitLabMRFilter(filter: Partial<GitLabMRFilter>) {
    config.value.filters.gitlabMr = { ...config.value.filters.gitlabMr, ...filter }
    persist()
  }

  function updateGitLabPipelineFilter(filter: Partial<GitLabPipelineFilter>) {
    config.value.filters.gitlabPipeline = { ...config.value.filters.gitlabPipeline, ...filter }
    persist()
  }

  function updateJiraTicketFilter(filter: Partial<JiraTicketFilter>) {
    config.value.filters.jiraTicket = { ...config.value.filters.jiraTicket, ...filter }
    persist()
  }

  function updateCalendarEventFilter(filter: Partial<CalendarEventFilter>) {
    config.value.filters.calendarEvent = { ...config.value.filters.calendarEvent, ...filter }
    persist()
  }

  function togglePin(itemType: string, item: any) {
    const itemId = getItemId(itemType, item)
    const idx = config.value.pinnedItems.findIndex(
      (p) => p.itemType === itemType && p.itemId === itemId
    )
    if (idx >= 0) {
      config.value.pinnedItems.splice(idx, 1)
    } else {
      config.value.pinnedItems.push({ itemType: itemType, itemId: itemId })
    }
    persist()
  }

  function isPinned(itemType: string, item: any): boolean {
    const itemId = getItemId(itemType, item)
    return config.value.pinnedItems.some(
      (p) => p.itemType === itemType && p.itemId === itemId
    )
  }

  /** Remove pinned items that no longer appear in the current data */
  function cleanupStalePins(currentItems: Array<{ type: string; data: any }>) {
    const validIds = new Set<string>()
    for (const item of currentItems) {
      const id = getItemId(item.type, item.data)
      if (id) validIds.add(`${item.type}:${id}`)
    }

    const before = config.value.pinnedItems.length
    config.value.pinnedItems = config.value.pinnedItems.filter(
      (p) => validIds.has(`${p.itemType}:${p.itemId}`)
    )
    if (config.value.pinnedItems.length !== before) {
      persist()
    }
  }

  return {
    config,
    loaded,
    saving,
    sectionOrder,
    visibleSections,
    calendarLayout,
    importantRules,
    pinnedItems,
    filters,
    load,
    persist,
    persistNow,
    setSectionOrder,
    setCalendarLayout,
    setVisibleSections,
    toggleSectionVisibility,
    isSectionVisible,
    setImportantRules,
    updateImportantRule,
    setFilters,
    updateGitHubPRFilter,
    updateGitLabMRFilter,
    updateGitLabPipelineFilter,
    updateJiraTicketFilter,
    updateCalendarEventFilter,
    togglePin,
    isPinned,
    cleanupStalePins,
  }
})
