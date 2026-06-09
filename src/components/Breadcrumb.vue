<template>
  <nav class="breadcrumb">
    <template v-for="(segment, index) in pathSegments" :key="index">
      <span v-if="index > 0" class="separator">/</span>
      <button 
        class="segment"
        :class="{ current: index === pathSegments.length - 1 }"
        @click="navigateTo(index)"
      >
        {{ segment.name }}
      </button>
    </template>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '../stores/app'

const store = useAppStore()

interface PathSegment {
  name: string
  path: string
}

const pathSegments = computed<PathSegment[]>(() => {
  if (!store.currentDirectory) return []
  
  const path = store.currentDirectory
  // Normalize separators and split
  const normalized = path.replace(/\\/g, '/')
  const isWindowsDrive = /^[A-Za-z]:/.test(normalized)
  const parts = normalized.split('/').filter(p => p)
  const segments: PathSegment[] = []
  
  let currentPath = ''
  for (let i = 0; i < parts.length; i++) {
    const part = parts[i]
    if (i === 0 && isWindowsDrive) {
      // First part is drive letter like "C:"
      currentPath = part + '/'
    } else if (currentPath === '') {
      // Unix root
      currentPath = '/' + part
    } else {
      currentPath = currentPath.endsWith('/') ? currentPath + part : currentPath + '/' + part
    }
    segments.push({
      name: part,
      path: currentPath
    })
  }
  
  return segments
})

function navigateTo(index: number) {
  if (index < pathSegments.value.length) {
    const segment = pathSegments.value[index]
    store.loadDirectory(segment.path)
  }
}
</script>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 6px 16px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  overflow-x: auto;
  white-space: nowrap;
}

.separator {
  color: var(--text-tertiary);
  font-size: 11px;
  user-select: none;
}

.segment {
  background: none;
  border: none;
  color: var(--accent);
  font-size: 11px;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.segment:hover {
  background: var(--accent-muted);
  color: var(--accent-hover);
}

.segment.current {
  color: var(--text-secondary);
  cursor: default;
}

.segment.current:hover {
  background: transparent;
  color: var(--text-secondary);
}
</style>
