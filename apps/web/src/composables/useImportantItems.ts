// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { computed, type Ref } from 'vue'
import type { ImportantRules, PinnedItem } from '@/types/dashboard'
import { getItemId } from '@/types/dashboard'

/** Check if an item matches any importance rule */
function matchesRules(itemType: string, item: any, rules: ImportantRules): boolean {
  switch (itemType) {
    case 'github_pr':
      if (rules.github_action_required && item.action_required) return true
      if (rules.github_new_comments && item.has_new_comments) return true
      if (rules.github_new_commits && item.has_new_commits) return true
      if (rules.github_changes_requested && item.review_decision === 'changes_requested') return true
      return false
    case 'gitlab_mr':
      if (rules.gitlab_mr_new_comments && item.has_new_comments) return true
      if (rules.gitlab_mr_new_commits && item.has_new_commits) return true
      return false
    case 'gitlab_pipeline':
      if (rules.gitlab_pipeline_failed && item.status === 'failed') return true
      return false
    case 'jira_ticket':
      if (rules.jira_high_priority) {
        const p = item.priority?.toLowerCase()
        if (p === 'highest' || p === 'high' || p === 'critical') return true
      }
      return false
    case 'calendar_event':
      if (rules.calendar_starting_soon) {
        const start = new Date(item.start_time).getTime()
        const now = Date.now()
        const diffMin = (start - now) / 60000
        if (diffMin >= 0 && diffMin <= 15) return true
      }
      return false
    default:
      return false
  }
}

/** Check if an item is manually pinned */
function isPinned(itemType: string, item: any, pinnedItems: PinnedItem[]): boolean {
  const itemId = getItemId(itemType, item)
  return pinnedItems.some((p) => p.item_type === itemType && p.item_id === itemId)
}

export interface ImportantSplit<T = any> {
  important: T[]
  rest: T[]
}

/**
 * Split a list of items into important (rule-matched + pinned) and rest.
 * Items in the important group are removed from rest to avoid duplication.
 */
export function useImportantItems(
  items: Ref<any[]>,
  itemType: string,
  rules: Ref<ImportantRules>,
  pinnedItems: Ref<PinnedItem[]>
) {
  return computed<ImportantSplit>(() => {
    const important: any[] = []
    const rest: any[] = []

    for (const item of items.value) {
      if (matchesRules(itemType, item, rules.value) || isPinned(itemType, item, pinnedItems.value)) {
        important.push(item)
      } else {
        rest.push(item)
      }
    }

    return { important, rest }
  })
}

/**
 * Split a flat items array (with type+data) into important and rest groups.
 * Used on the dashboard where all item types are mixed together.
 */
export function useImportantItemsAll(
  items: Ref<Array<{ type: string; data: any }>>,
  rules: Ref<ImportantRules>,
  pinnedItems: Ref<PinnedItem[]>
) {
  return computed(() => {
    const important: Array<{ type: string; data: any }> = []
    const rest: Array<{ type: string; data: any }> = []

    for (const item of items.value) {
      if (
        matchesRules(item.type, item.data, rules.value) ||
        isPinned(item.type, item.data, pinnedItems.value)
      ) {
        important.push(item)
      } else {
        rest.push(item)
      }
    }

    return { important, rest }
  })
}
