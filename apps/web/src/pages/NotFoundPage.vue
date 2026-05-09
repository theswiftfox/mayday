<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <div class="p-6 max-w-md mx-auto text-center mt-20">
    <h1 class="text-4xl font-bold mb-4">404</h1>
    <p class="text-lg text-gray-600 dark:text-gray-400 mb-6">Page not found</p>
    <button
      class="btn btn-primary"
      @click="router.push('/')"
    >
      Back to Dashboard
    </button>
  </div>
</template>
