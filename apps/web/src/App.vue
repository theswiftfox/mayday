<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import AppLayout from './components/AppLayout.vue'
import { useTheme } from './composables/useTheme'

// Initialize theme on app load
useTheme()
</script>

<template>
  <AppLayout>
    <router-view />
  </AppLayout>
</template>
