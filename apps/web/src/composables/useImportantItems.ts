// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { computed, type Ref } from 'vue'
import type { ImportantRules, PinnedItem } from '@/types/dashboard'
import { getItemId } from '@/types/dashboard'

/** Check if an item matches any importance rule */
function matchesRules(itemType: string, item: any, rules: ImportantRules, now: number): boolean {
  switch (itemType) {
    case 'github_pr':
      if (rules.githubActionRequired && item.actionRequired) return true
      if (rules.githubNewComments && item.hasNewComments) return true
      if (rules.githubNewCommits && item.hasNewCommits) return true
      if (rules.githubChangesRequested && item.reviewDecision === 'changes_requested') return true
      return false
    case 'gitlab_mr':
      if (rules.gitlabMrNewComments && item.hasNewComments) return true
      if (rules.gitlabMrNewCommits && item.hasNewCommits) return true
      return false
    case 'gitlab_pipeline':
      if (rules.gitlabPipelineFailed && item.status === 'failed') return true
      return false
    case 'jira_ticket':
      if (rules.jiraHighPriority) {
        const p = item.priority?.toLowerCase()
        if (p === 'highest' || p === 'high' || p === 'critical') return true
      }
      return false
    case 'calendar_event':
      if (rules.calendarStartingSoon) {
        const start = new Date(item.startTime).getTime()
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
  return pinnedItems.some((p) => p.itemType === itemType && p.itemId === itemId)
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
  pinnedItems: Ref<PinnedItem[]>,
  now: Ref<number>
) {
  return computed<ImportantSplit>(() => {
    const important: any[] = []
    const rest: any[] = []

    for (const item of items.value) {
      if (matchesRules(itemType, item, rules.value, now.value) || isPinned(itemType, item, pinnedItems.value)) {
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
  pinnedItems: Ref<PinnedItem[]>,
  now: Ref<number>
) {
  return computed(() => {
    const important: Array<{ type: string; data: any }> = []
    const rest: Array<{ type: string; data: any }> = []

    for (const item of items.value) {
      if (
        matchesRules(item.type, item.data, rules.value, now.value) ||
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
