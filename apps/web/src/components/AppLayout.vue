<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

const navItems = [
  { path: '/', label: 'Dashboard', icon: '◉' },
  { path: '/github', label: 'GitHub', icon: '⌥' },
  { path: '/gitlab', label: 'GitLab', icon: '⎔' },
  { path: '/jira', label: 'JIRA', icon: '▦' },
  { path: '/calendar', label: 'Calendar', icon: '▣' },
  { path: '/settings', label: 'Settings', icon: '⚙' },
]

function isActive(path: string): boolean {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}
</script>

<template>
  <div class="flex h-screen">
    <!-- Sidebar -->
    <aside class="w-56 border-r border-[var(--color-border)] bg-[var(--color-surface)] flex flex-col">
      <div class="p-4 border-b border-[var(--color-border)]">
        <h1 class="text-lg font-semibold">My Day</h1>
        <p class="text-xs text-[var(--color-text-muted)]">{{ new Date().toLocaleDateString('en-US', { weekday: 'long', month: 'short', day: 'numeric' }) }}</p>
      </div>
      <nav class="flex-1 p-2">
        <RouterLink
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors"
          :class="isActive(item.path) ? 'bg-[var(--color-primary)] text-white' : 'text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]'"
        >
          <span class="text-base">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </RouterLink>
      </nav>
    </aside>

    <!-- Main content -->
    <main class="flex-1 overflow-auto">
      <slot />
    </main>
  </div>
</template>
