<template>
  <div class="ai-panel">
    <div class="ai-header">
      <h3 class="ai-title">废片检测</h3>
      <div class="header-actions">
        <button v-if="!store.wasteAnalyzing && store.wasteResults.length > 0" class="btn-reset" @click="store.resetWasteResults()" title="清除结果">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        </button>
      </div>
    </div>

    <!-- 分析中进度条 -->
    <div v-if="store.wasteAnalyzing" class="ai-progress">
      <div class="progress-info">
        <span>分析中...</span>
        <span>{{ store.wasteProgress }} / {{ store.wasteTotal }}</span>
      </div>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: (store.wasteProgress / Math.max(store.wasteTotal, 1) * 100) + '%' }"></div>
      </div>
    </div>

    <!-- 废片列表（分析中和完成后都显示） -->
    <div v-if="store.wasteResults.length > 0" class="ai-results">
      <div class="feedback-info">
        <span class="feedback-count">已收集 {{ store.wasteFeedbackCount }} 条标记</span>
        <span v-if="store.wasteFeedbackCount > 0" class="feedback-ready">分类器已训练</span>
        <span v-else class="feedback-hint">标记图片后自动训练</span>
      </div>

      <div class="results-summary">
        <span class="summary-total">已分析 {{ store.wasteResults.length }} 张</span>
        <span class="summary-waste">发现 {{ wasteCount }} 张废片</span>
      </div>

      <div v-if="wasteCount > 0" class="waste-actions">
        <button class="btn btn-sm btn-danger" @click="store.selectWasteImages()">选中废片</button>
        <button class="btn btn-sm btn-ghost" @click="store.excludeWasteImages()">排除废片</button>
      </div>

      <!-- 废片列表 -->
      <div class="waste-list">
        <div v-for="item in wasteImages" :key="item.path" class="waste-card"
             @click="$emit('preview', item.path)"
             @mouseenter="$emit('hoverWaste', item.path)"
             @mouseleave="$emit('hoverWaste', null)">
          <img class="waste-thumb" :src="getThumbUrl(item.path)" loading="lazy" />
          <div class="waste-info">
            <div class="waste-name">{{ getFileName(item.path) }}</div>
            <div class="waste-reasons">
              <span v-for="reason in item.reasons" :key="reason" class="reason-tag">{{ reason }}</span>
            </div>
            <div class="waste-meta">
              <span class="waste-score">废片: {{ (item.waste_score * 100).toFixed(0) }}%</span>
              <span class="quality-score-small" :class="getQualityClass(item.quality_score)">
                质量: {{ item.quality_score.toFixed(1) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="wasteCount === 0" class="no-waste">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--success)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
        <span>未发现废片</span>
      </div>
    </div>

    <!-- 就绪状态 -->
    <div v-else class="ai-empty">
      <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class="ai-icon"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
      <p v-if="store.images.length === 0">当前没有图片，请先打开目录</p>
      <p v-else>点击开始分析</p>
      <p class="feedback-hint-inline">已收集 {{ store.wasteFeedbackCount }} 条标记{{ store.wasteFeedbackCount > 0 ? '，分类器已训练' : '' }}</p>
      <button v-if="store.images.length > 0" class="btn btn-primary btn-sm" @click="startAnalysis">
        开始分析 ({{ store.images.length }} 张)
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAppStore } from '../stores/app'
import { convertFileSrc } from '@tauri-apps/api/core'

const RAW_EXTENSIONS = ['cr2', 'cr3', 'nef', 'arw', 'dng', 'orf', 'rw2', 'pef', 'srw', 'raf']

defineEmits<{
  preview: [path: string]
  hoverWaste: [path: string | null]
}>()

const store = useAppStore()

const wasteImages = computed(() => store.wasteResults.filter(r => r.is_waste))
const wasteCount = computed(() => wasteImages.value.length)



function getFileName(path: string): string {
  return path.split(/[/\\]/).pop() || path
}

function getQualityClass(score: number): string {
  if (score >= 7) return 'quality-good'
  if (score >= 4) return 'quality-medium'
  return 'quality-bad'
}

const thumbCache = ref<Map<string, string>>(new Map())

function getThumbUrl(path: string): string {
  if (thumbCache.value.has(path)) {
    return thumbCache.value.get(path)!
  }
  const ext = path.split('.').pop()?.toLowerCase() || ''
  if (RAW_EXTENSIONS.includes(ext)) {
    // RAW 文件异步加载预览
    store.getRawPreview(path).then(url => {
      if (url) thumbCache.value.set(path, url)
    })
    return ''
  }
  return convertFileSrc(path)
}

async function startAnalysis() {
  if (store.images.length === 0) return
  try {
    const paths = store.images.map(img => img.path)
    await store.batchAnalyzeWaste(paths)
    store.showToast("废片分析完成")
  } catch (e) {
    console.error("Analysis failed:", e)
    store.showToast("分析失败: " + String(e), "error")
  }
}
</script>

<style scoped>
.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--border-subtle);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 12px 0;
}

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

.header-actions {
  display: flex;
  gap: 4px;
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
  gap: 6px;
}

.waste-card {
  display: flex;
  gap: 8px;
  padding: 6px 8px;
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.waste-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
}

.waste-card.waste-selected {
  border-color: var(--accent);
  background: var(--accent-muted);
}

.waste-thumb {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: 4px;
  flex-shrink: 0;
  background: var(--bg-elevated);
}

.waste-info {
  flex: 1;
  min-width: 0;
}

.waste-name {
  font-size: 11px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.waste-reasons {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 4px;
}

.reason-tag {
  display: inline-block;
  padding: 1px 6px;
  background: rgba(212, 83, 83, 0.15);
  border-radius: 3px;
  font-size: 10px;
  color: var(--danger);
}

.waste-meta {
  display: flex;
  gap: 8px;
  font-size: 10px;
  color: var(--text-tertiary);
}

.waste-score {
  color: var(--danger);
}

.quality-score-small {
  color: var(--accent);
}

.quality-good {
  color: var(--success) !important;
}

.quality-medium {
  color: var(--accent) !important;
}

.quality-bad {
  color: var(--danger) !important;
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

.quality-stats {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-subtle);
  margin-bottom: 8px;
}

.quality-score {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 80px;
}

.quality-label {
  font-size: 11px;
  color: var(--text-secondary);
}

.quality-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--accent);
}

.quality-bar {
  flex: 1;
  height: 6px;
  background: var(--bg-elevated);
  border-radius: 3px;
  overflow: hidden;
}

.quality-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--danger) 0%, var(--accent) 50%, var(--success) 100%);
  border-radius: 3px;
  transition: width 0.3s ease;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.waste-ctx-menu {
  position: fixed;
  z-index: 2000;
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  padding: 4px 0;
  min-width: 140px;
}

.ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 12px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: background var(--transition-fast);
  text-align: left;
}

.ctx-item:hover {
  background: var(--bg-hover);
}
</style>
