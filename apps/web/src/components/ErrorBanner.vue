<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
defineProps<{
  errors: { source: string; message: string }[]
}>()
</script>

<template>
  <div
    v-if="errors.length"
    class="rounded-md border px-3 py-2 mb-4 space-y-1"
    style="background: color-mix(in srgb, var(--color-error) 10%, transparent); border-color: var(--color-error)"
  >
    <div
      v-for="(error, i) in errors"
      :key="i"
      class="text-xs"
      style="color: var(--color-error)"
    >
      <span class="font-medium">{{ error.source }}:</span> {{ error.message }}
    </div>
  </div>
</template>
