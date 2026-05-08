<script setup lang="ts">
import { ref } from 'vue'
import MarkdownContent from '@/components/MarkdownContent.vue'

export interface ThreadComment {
  id: string | number
  author: string
  body: string
  created_at: string
  /** Optional tag like "system" for GitLab system notes */
  tag?: string
  /** Optional tag color class */
  tag_class?: string
}

const props = withDefaults(defineProps<{
  /** Comments in the thread — first is the root, rest are replies */
  comments: ThreadComment[]
  /** Whether this thread is resolved */
  resolved?: boolean
  /** File path (for code review threads) */
  path?: string
  /** Line number (for code review threads) */
  line?: number
  /** Whether the thread is outdated (e.g. code has changed) */
  outdated?: boolean
  /** Start collapsed when resolved */
  collapseResolved?: boolean
}>(), {
  resolved: false,
  outdated: false,
  collapseResolved: true,
})

const collapsed = ref(props.resolved && props.collapseResolved)

function formatDate(iso: string): string {
  if (!iso) return ''
  return new Date(iso).toLocaleString()
}

function toggle() {
  collapsed.value = !collapsed.value
}
</script>

<template>
  <div
    class="rounded-lg border overflow-hidden"
    :style="{
      borderColor: resolved ? 'var(--color-border)' : 'var(--color-border)',
      background: 'var(--color-surface)',
    }"
  >
    <!-- Thread header (clickable when resolved for collapse/expand) -->
    <div
      class="flex items-center gap-2 px-4 py-2 text-sm"
      :class="{ 'cursor-pointer hover:bg-[var(--color-surface-hover)]': resolved }"
      :style="{ background: 'var(--color-surface-hover)', borderBottom: collapsed ? 'none' : '1px solid var(--color-border)' }"
      @click="resolved && toggle()"
    >
      <!-- File path badge -->
      <span v-if="path" class="font-mono text-xs truncate" style="color: var(--color-text-muted)">
        {{ path }}<span v-if="line">:{{ line }}</span>
      </span>

      <span class="flex-1" />

      <!-- Resolved badge -->
      <span
        v-if="resolved"
        class="text-xs px-2 py-0.5 rounded-full font-medium bg-purple-500/15 text-purple-400"
      >Resolved</span>

      <!-- Outdated badge -->
      <span
        v-if="outdated"
        class="text-xs px-2 py-0.5 rounded-full font-medium"
        style="background: var(--color-surface-hover); color: var(--color-text-muted)"
      >Outdated</span>

      <!-- Collapse indicator -->
      <svg
        v-if="resolved"
        xmlns="http://www.w3.org/2000/svg"
        class="w-4 h-4 transition-transform"
        :class="{ '-rotate-90': collapsed }"
        viewBox="0 0 20 20"
        fill="currentColor"
        style="color: var(--color-text-muted)"
      >
        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
      </svg>
    </div>

    <!-- Thread body -->
    <div v-show="!collapsed">
      <!-- Root comment -->
      <div v-if="comments.length" class="px-4 py-3">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm font-medium" style="color: var(--color-text)">{{ comments[0].author }}</span>
          <span
            v-if="comments[0].tag"
            class="text-xs px-2 py-0.5 rounded"
            :class="comments[0].tag_class || 'bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]'"
          >{{ comments[0].tag }}</span>
          <span class="text-xs ml-auto" style="color: var(--color-text-muted)">{{ formatDate(comments[0].created_at) }}</span>
        </div>
        <div class="text-sm" style="color: var(--color-text)">
          <MarkdownContent :content="comments[0].body" />
        </div>
      </div>

      <!-- Replies -->
      <template v-if="comments.length > 1">
        <div
          v-for="reply in comments.slice(1)"
          :key="reply.id"
          class="px-4 py-3 ml-4 border-l-2"
          style="border-color: var(--color-border)"
        >
          <div class="flex items-center gap-2 mb-2">
            <span class="text-sm font-medium" style="color: var(--color-text)">{{ reply.author }}</span>
            <span
              v-if="reply.tag"
              class="text-xs px-2 py-0.5 rounded"
              :class="reply.tag_class || 'bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]'"
            >{{ reply.tag }}</span>
            <span class="text-xs ml-auto" style="color: var(--color-text-muted)">{{ formatDate(reply.created_at) }}</span>
          </div>
          <div class="text-sm" style="color: var(--color-text)">
            <MarkdownContent :content="reply.body" />
          </div>
        </div>
      </template>
    </div>

    <!-- Collapsed summary -->
    <div v-if="collapsed" class="px-4 py-2 text-xs" style="color: var(--color-text-muted)">
      {{ comments.length }} comment{{ comments.length !== 1 ? 's' : '' }}
      <span v-if="comments.length"> &mdash; {{ comments[0].author }}: {{ comments[0].body.slice(0, 80) }}{{ comments[0].body.length > 80 ? '...' : '' }}</span>
    </div>
  </div>
</template>
