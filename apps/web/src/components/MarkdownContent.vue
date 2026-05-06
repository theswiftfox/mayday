<script setup lang="ts">
import { computed } from 'vue'
import { marked } from 'marked'

const props = defineProps<{
  content: string
}>()

// Configure marked for GitHub-flavored markdown
marked.setOptions({
  gfm: true,
  breaks: true,
})

const renderedHtml = computed(() => {
  if (!props.content) return ''
  return marked.parse(props.content) as string
})
</script>

<template>
  <div class="markdown-content" v-html="renderedHtml"></div>
</template>

<style scoped>
.markdown-content :deep(h1) {
  font-size: 1.5em;
  font-weight: 700;
  margin: 1em 0 0.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--color-border);
}

.markdown-content :deep(h2) {
  font-size: 1.3em;
  font-weight: 600;
  margin: 1em 0 0.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--color-border);
}

.markdown-content :deep(h3) {
  font-size: 1.1em;
  font-weight: 600;
  margin: 0.8em 0 0.4em;
}

.markdown-content :deep(p) {
  margin: 0.5em 0;
  line-height: 1.6;
}

.markdown-content :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
}

.markdown-content :deep(a:hover) {
  text-decoration: underline;
}

.markdown-content :deep(code) {
  background-color: var(--color-surface-hover);
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: 'SF Mono', 'Fira Code', 'Fira Mono', Menlo, monospace;
}

.markdown-content :deep(pre) {
  background-color: var(--color-surface-hover);
  padding: 1em;
  border-radius: 6px;
  overflow-x: auto;
  margin: 0.8em 0;
}

.markdown-content :deep(pre code) {
  background: none;
  padding: 0;
  font-size: 0.85em;
}

.markdown-content :deep(blockquote) {
  border-left: 3px solid var(--color-border);
  padding-left: 1em;
  margin: 0.8em 0;
  color: var(--color-text-muted);
}

.markdown-content :deep(ul),
.markdown-content :deep(ol) {
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.markdown-content :deep(li) {
  margin: 0.25em 0;
  line-height: 1.5;
}

.markdown-content :deep(ul) {
  list-style-type: disc;
}

.markdown-content :deep(ol) {
  list-style-type: decimal;
}

.markdown-content :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.8em 0;
}

.markdown-content :deep(th),
.markdown-content :deep(td) {
  border: 1px solid var(--color-border);
  padding: 0.5em 0.75em;
  text-align: left;
}

.markdown-content :deep(th) {
  background-color: var(--color-surface-hover);
  font-weight: 600;
}

.markdown-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--color-border);
  margin: 1.5em 0;
}

.markdown-content :deep(img) {
  max-width: 100%;
  border-radius: 6px;
}

.markdown-content :deep(input[type="checkbox"]) {
  margin-right: 0.4em;
}
</style>
