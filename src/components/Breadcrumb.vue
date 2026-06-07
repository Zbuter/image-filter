<template>
  <div class="breadcrumb">
    <span class="label">路径:</span>
    <template v-for="(segment, index) in pathSegments" :key="index">
      <span 
        v-if="index > 0" 
        class="separator"
      >›</span>
      <span 
        class="segment"
        :class="{ current: index === pathSegments.length - 1 }"
        @click="navigateTo(index)"
      >
        {{ segment.name }}
      </span>
    </template>
  </div>
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
  const parts = path.split('/').filter(p => p)
  const segments: PathSegment[] = []
  
  // Handle macOS root
  if (path.startsWith('/')) {
    segments.push({ name: '/', path: '/' })
  }
  
  // Build path segments
  let currentPath = path.startsWith('/') ? '' : ''
  for (const part of parts) {
    currentPath = currentPath ? `${currentPath}/${part}` : `/${part}`
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
  width: 100%;
  flex-shrink: 0;
  background: #252525;
  padding: 8px 16px;
  border-bottom: 1px solid #3a3a3a;
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.label {
  color: #888;
  font-size: 12px;
  margin-right: 4px;
}

.separator {
  color: #666;
  font-size: 12px;
  margin: 0 2px;
}

.segment {
  color: #007acc;
  font-size: 12px;
  cursor: pointer;
  transition: color 0.2s;
}

.segment:hover {
  color: #0098ff;
}

.segment.current {
  color: #ccc;
  cursor: default;
}

.segment.current:hover {
  color: #ccc;
}
</style>
