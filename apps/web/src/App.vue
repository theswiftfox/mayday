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
