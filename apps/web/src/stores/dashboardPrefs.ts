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

  // ---- Getters ----

  const sectionOrder = computed(() => config.value.section_order)
  const visibleSections = computed(() => config.value.visible_sections)
  const calendarLayout = computed(() => config.value.calendar_layout)
  const importantRules = computed(() => config.value.important_rules)
  const pinnedItems = computed(() => config.value.pinned_items)
  const filters = computed(() => config.value.filters)

  // ---- Persistence helpers ----

  /** Load dashboard prefs from the server config */
  async function load() {
    try {
      const serverConfig = await api.getConfig()
      if (serverConfig.dashboard) {
        // Merge with defaults so any missing fields get default values
        config.value = { ...defaultDashboardConfig(), ...serverConfig.dashboard }
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
    for (const key of ['section_order', 'visible_sections'] as const) {
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
    if (dirty) persist()
  }

  /** Persist the current dashboard prefs to the server config */
  async function persist() {
    saving.value = true
    try {
      const serverConfig = await api.getConfig()
      serverConfig.dashboard = config.value
      await api.updateConfig(serverConfig)
    } catch {
      // Silently fail — prefs are still in memory
    } finally {
      saving.value = false
    }
  }

  // ---- Actions ----

  function setSectionOrder(order: SectionType[]) {
    config.value.section_order = order
    persist()
  }

  function setCalendarLayout(layout: 'sidebar' | 'inline') {
    config.value.calendar_layout = layout
    persist()
  }

  function setVisibleSections(sections: SectionType[]) {
    config.value.visible_sections = sections
    persist()
  }

  function toggleSectionVisibility(section: SectionType) {
    const idx = config.value.visible_sections.indexOf(section)
    if (idx >= 0) {
      config.value.visible_sections.splice(idx, 1)
    } else {
      config.value.visible_sections.push(section)
    }
    persist()
  }

  function isSectionVisible(section: SectionType): boolean {
    return config.value.visible_sections.includes(section)
  }

  function setImportantRules(rules: ImportantRules) {
    config.value.important_rules = rules
    persist()
  }

  function updateImportantRule(key: keyof ImportantRules, value: boolean) {
    config.value.important_rules[key] = value
    persist()
  }

  function setFilters(filters: DashboardFilters) {
    config.value.filters = filters
    persist()
  }

  function updateGitHubPRFilter(filter: Partial<GitHubPRFilter>) {
    config.value.filters.github_pr = { ...config.value.filters.github_pr, ...filter }
    persist()
  }

  function updateGitLabMRFilter(filter: Partial<GitLabMRFilter>) {
    config.value.filters.gitlab_mr = { ...config.value.filters.gitlab_mr, ...filter }
    persist()
  }

  function updateGitLabPipelineFilter(filter: Partial<GitLabPipelineFilter>) {
    config.value.filters.gitlab_pipeline = { ...config.value.filters.gitlab_pipeline, ...filter }
    persist()
  }

  function updateJiraTicketFilter(filter: Partial<JiraTicketFilter>) {
    config.value.filters.jira_ticket = { ...config.value.filters.jira_ticket, ...filter }
    persist()
  }

  function updateCalendarEventFilter(filter: Partial<CalendarEventFilter>) {
    config.value.filters.calendar_event = { ...config.value.filters.calendar_event, ...filter }
    persist()
  }

  function togglePin(itemType: string, item: any) {
    const itemId = getItemId(itemType, item)
    const idx = config.value.pinned_items.findIndex(
      (p) => p.item_type === itemType && p.item_id === itemId
    )
    if (idx >= 0) {
      config.value.pinned_items.splice(idx, 1)
    } else {
      config.value.pinned_items.push({ item_type: itemType, item_id: itemId })
    }
    persist()
  }

  function isPinned(itemType: string, item: any): boolean {
    const itemId = getItemId(itemType, item)
    return config.value.pinned_items.some(
      (p) => p.item_type === itemType && p.item_id === itemId
    )
  }

  /** Remove pinned items that no longer appear in the current data */
  function cleanupStalePins(currentItems: Array<{ type: string; data: any }>) {
    const validIds = new Set<string>()
    for (const item of currentItems) {
      const id = getItemId(item.type, item.data)
      if (id) validIds.add(`${item.type}:${id}`)
    }

    const before = config.value.pinned_items.length
    config.value.pinned_items = config.value.pinned_items.filter(
      (p) => validIds.has(`${p.item_type}:${p.item_id}`)
    )
    if (config.value.pinned_items.length !== before) {
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
