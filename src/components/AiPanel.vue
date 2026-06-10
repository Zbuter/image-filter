<template>
  <div class="ai-panel">
    <div class="ai-header">
      <h3 class="ai-title">AI 废片检测</h3>
      <button v-if="store.images.length > 0" class="btn-reset" :class="{ spinning: store.dedupDetecting }" @click="$emit('detectDuplicates')" title="检测重复" :disabled="store.dedupDetecting">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="16" y="16" width="6" height="6" rx="1"/><rect x="2" y="2" width="6" height="6" rx="1"/><path d="M8 2v4a2 2 0 0 0 2 2h4"/><path d="M16 22v-4a2 2 0 0 0-2-2h-4"/></svg>
      </button>
      <button v-if="!store.aiAnalyzing && store.aiResults.length > 0" class="btn-reset" @click="$emit('startAnalysis')" title="重新分析">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/></svg>
      </button>
      <button v-if="!store.aiAnalyzing && store.aiResults.length > 0" class="btn-reset" @click="store.resetAiResults()" title="清除结果">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
      </button>
    </div>

    <!-- Not loaded state -->
    <div v-if="!store.aiModelLoaded" class="ai-empty">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="ai-icon"><path d="M12 2a4 4 0 0 1 4 4c0 1.95-1.4 3.58-3.25 3.93L12 22"/><path d="M8 6a4 4 0 0 1 .65-2.18"/><path d="M17 12.5c1.77.64 3 2.34 3 4.28A4.5 4.5 0 0 1 15.5 21h-7A4.5 4.5 0 0 1 4 16.78c0-1.94 1.23-3.64 3-4.28"/></svg>
      <p v-if="downloading">下载中... {{ downloadProgress }}</p>
      <p v-else>模型未加载</p>
      <button v-if="!downloading" class="btn btn-primary btn-sm" @click="downloadModel">获取 AI 模型</button>
      <button v-if="!downloading" class="btn btn-ghost btn-sm" @click="$emit('loadModel')">从 ZIP 加载</button>
    </div>

    <!-- Analyzing state -->
    <div v-else-if="store.aiAnalyzing" class="ai-progress">
      <div class="progress-info">
        <span>分析中...</span>
        <span>{{ store.aiProgress }} / {{ store.aiTotal }}</span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: (store.aiProgress / Math.max(store.aiTotal, 1) * 100) + '%' }"></div>
      </div>
    </div>

    <!-- Results state -->
    <div v-else-if="store.aiResults.length > 0" class="ai-results">
      <div class="feedback-info">
        <span class="feedback-count">已收集 {{ store.feedbackCount }} 条标记</span>
        <span v-if="store.feedbackCount > 0" class="feedback-ready">分类头已训练 ({{ store.feedbackCount }} 条数据)</span>
        <span v-else class="feedback-hint">标记图片后自动训练</span>
      </div>
      <div class="results-summary">
        <span class="summary-total">已分析 {{ store.aiResults.length }} 张</span>
        <span class="summary-waste">发现 {{ wasteImages.length }} 张废片</span>
      </div>

      <div v-if="wasteImages.length > 0" class="waste-actions">
        <button class="btn btn-sm btn-danger" @click="store.selectWasteImages()">选中废片</button>
        <button class="btn btn-sm btn-ghost" @click="store.excludeWasteImages()">排除废片</button>
      </div>

      <div class="waste-list">
        <div v-for="item in wasteImages" :key="item.path" class="waste-card"
        @click="$emit('preview', item.path)"
        @mouseenter="$emit('hoverWaste', item.path)"
        @mouseleave="$emit('hoverWaste', null)"
        @contextmenu.prevent="$emit('wasteContextMenu', $event, item.path)">
          <div class="waste-name">{{ getFileName(item.path) }}</div>
          <div class="waste-labels">
            <span v-for="label in item.labels" :key="label" class="label-tag">{{ labelMap[label] || label }}</span>
          </div>
        </div>
      </div>

      <div v-if="wasteImages.length === 0" class="no-waste">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        <span>未发现废片</span>
      </div>
    </div>

    <!-- Ready state -->
    <div v-else class="ai-empty">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="ai-icon"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
      <p v-if="store.images.length === 0">当前没有图片，请先打开目录</p>
      <p v-else>模型已就绪，点击开始分析</p>
      <p class="feedback-hint-inline">已收集 {{ store.feedbackCount }} 条标记{{ store.feedbackCount > 0 ? '，分类头已训练' : '' }}</p>
      <button v-if="store.images.length > 0" class="btn btn-primary btn-sm" @click="$emit('startAnalysis')">开始分析 ({{ store.images.length }} 张)</button>
    </div>
  
    <!-- Duplicate Detection Results -->
    <div v-if="store.duplicateGroups.length > 0" class="dedup-section">
      <div class="dedup-header">
        <span class="dedup-title">发现 {{ store.duplicateGroups.length }} 组重复</span>
        <button class="btn btn-sm btn-danger" @click="$emit('markAllDuplicates')">全部保留最佳</button>
      </div>
      <div class="dedup-list">
        <div v-for="(group, gi) in store.duplicateGroups" :key="gi" class="dedup-group">
          <div class="dedup-best" 
               @click="$emit('preview', group.best_path)"
               @mouseenter="$emit('hoverWaste', group.best_path)"
               @mouseleave="$emit('hoverWaste', null)">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>
            <span class="dedup-best-name">{{ getFileName(group.best_path) }}</span>
            <span class="dedup-score">质量 {{ group.best_score.toFixed(0) }}</span>
          </div>
          <div v-for="dup in group.duplicates" :key="dup.path" class="dedup-dup"
               @click="$emit('preview', dup.path)"
               @mouseenter="$emit('hoverWaste', dup.path)"
               @mouseleave="$emit('hoverWaste', null)"
               @contextmenu.prevent="$emit('wasteContextMenu', $event, dup.path)">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="var(--danger)" stroke-width="2.5"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            <span class="dedup-dup-name">{{ getFileName(dup.path) }}</span>
            <span class="dedup-sim">{{ (dup.similarity * 100).toFixed(1) }}%</span>
            <span class="dedup-score">{{ dup.score.toFixed(0) }}</span>
          </div>
          <div class="dedup-actions">
            <button class="btn-dedup-mark" @click="$emit('markGroupDuplicates', group)">保留最佳，其余标废</button>
            <button class="btn-dedup-ignore" @click="$emit('ignoreGroup', gi)">忽略此组</button>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '../stores/app'

defineEmits<{
  loadModel: []
  preview: [path: string]
  startAnalysis: []
  hoverWaste: [path: string | null]
  wasteContextMenu: [event: MouseEvent, path: string]
  detectDuplicates: []
  markAllDuplicates: []
  markGroupDuplicates: [group: any]
  ignoreGroup: [index: number]
}>()

import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const store = useAppStore()
const downloading = ref(false)
const downloadProgress = ref('')

const labelMap: Record<string, string> = {
  meme_emoji: '表情包',
  bad_expression_blur: '表情崩坏/模糊',
  backlit: '严重背光',
  lens_distortion: '镜头畸变',
}

// Load feedback count when panel mounts
import { onMounted } from 'vue'
onMounted(() => { store.loadFeedbackCount() })

const wasteImages = computed(() => store.getWasteImages())

async function downloadModel() {
  downloading.value = true;
  downloadProgress.value = '准备下载...';
  try {
    const modelDir = await invoke<string>('download_ai_model');
    await store.initAiModel(modelDir);
    downloadProgress.value = '';
  } catch (e) {
    console.error('Download failed:', e);
    downloadProgress.value = '下载失败: ' + String(e);
  } finally {
    downloading.value = false;
  }
}

function getFileName(path: string): string {
  return path.split(/[/\\]/).pop() || path
}
</script>

<style scoped>
.ai-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 12px;
  gap: 12px;
}

.ai-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.ai-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.btn-reset {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-reset:hover {
  background: var(--bg-hover);
  color: var(--danger);
}

.ai-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 32px 16px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 12px;
}

.ai-icon {
  color: var(--text-tertiary);
}

.ai-progress {
  padding: 16px 0;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.progress-bar {
  height: 4px;
  background: var(--bg-elevated);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.results-summary {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
}

.summary-total {
  color: var(--text-secondary);
}

.summary-waste {
  color: var(--danger);
  font-weight: 600;
}

.waste-actions {
  display: flex;
  gap: 6px;
}

.btn-sm {
  padding: 4px 10px;
  font-size: 11px;
}

.btn-danger {
  background: rgba(212, 83, 83, 0.12);
  border-color: rgba(212, 83, 83, 0.3);
  color: var(--danger);
}

.btn-danger:hover {
  background: rgba(212, 83, 83, 0.2);
  border-color: var(--danger);
}

.waste-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.waste-card {
  padding: 8px;
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.waste-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
}

.waste-name {
  font-size: 11px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.waste-labels {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}

.label-tag {
  display: inline-block;
  padding: 1px 5px;
  background: rgba(212, 83, 83, 0.1);
  border-radius: 3px;
  font-size: 10px;
  color: var(--danger);
}

.no-waste {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 24px 0;
  color: var(--success);
  font-size: 12px;
}

.feedback-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  padding: 6px 0;
  border-bottom: 1px solid var(--border-subtle);
  margin-bottom: 4px;
}

.feedback-count {
  color: var(--text-secondary);
}

.feedback-ready {
  color: var(--success);
  font-weight: 600;
}

.feedback-hint {
  color: var(--text-tertiary);
}

.feedback-hint-inline {
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 4px;
}


/* Dedup Section */
.dedup-section {
  border-top: 1px solid var(--border-subtle);
  padding-top: 8px;
  margin-top: 8px;
}

.dedup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.dedup-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.dedup-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 300px;
  overflow-y: auto;
}

.dedup-group {
  background: var(--bg-elevated);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  padding: 4px;
}

.dedup-best, .dedup-dup {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 4px;
  border-radius: 3px;
  cursor: pointer;
  font-size: 11px;
  transition: background var(--transition-fast);
}

.dedup-best:hover, .dedup-dup:hover {
  background: var(--bg-hover);
}

.dedup-best-name, .dedup-dup-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--text-primary);
}

.dedup-score {
  font-size: 10px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}


.btn-reset.spinning svg {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.btn-reset:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.dedup-actions {
  display: flex;
  gap: 4px;
  padding: 4px 0 0 0;
  border-top: 1px solid var(--border-subtle);
  margin-top: 4px;
}

.btn-dedup-mark {
  flex: 1;
  padding: 3px 6px;
  background: rgba(212, 83, 83, 0.1);
  border: 1px solid rgba(212, 83, 83, 0.3);
  border-radius: 3px;
  color: var(--danger);
  font-size: 10px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-dedup-mark:hover {
  background: rgba(212, 83, 83, 0.2);
}

.btn-dedup-ignore {
  padding: 3px 6px;
  background: transparent;
  border: 1px solid var(--border-subtle);
  border-radius: 3px;
  color: var(--text-tertiary);
  font-size: 10px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-dedup-ignore:hover {
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.dedup-sim {
  font-size: 10px;
  color: var(--accent);
  flex-shrink: 0;
  font-weight: 600;
}
</style>
