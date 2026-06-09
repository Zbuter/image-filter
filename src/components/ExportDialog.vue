<template>
  <n-modal
    v-model:show="showDialog"
    preset="dialog"
    title="导出选中图片"
    positive-text="导出"
    negative-text="取消"
    :positive-button-props="{ disabled: !targetPath || exporting }"
    @positive-click="handleExport"
    @negative-click="handleCancel"
  >
    <div class="export-dialog">
      <div class="export-info">
        <p>将导出 <strong>{{ store.selectedCount }}</strong> 张图片</p>
      </div>

      <div class="target-path">
        <label>目标目录:</label>
        <div class="path-input">
          <input
            type="text"
            :value="targetPath"
            readonly
            placeholder="请选择目标目录"
          />
          <button @click="selectDirectory" :disabled="exporting">选择</button>
        </div>
      </div>

      <div v-if="exporting" class="export-progress">
        <n-progress
          type="line"
          :percentage="progress"
          :status="progressStatus"
        />
        <p class="progress-text">
          已导出 {{ completedCount }} / {{ store.selectedCount }}
        </p>
      </div>

      <div v-if="exportError" class="export-error">
        <p>导出失败: {{ exportError }}</p>
      </div>
    </div>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { NModal, NProgress } from 'naive-ui'
import { useAppStore } from '../stores/app'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{
  show: boolean
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
  (e: 'exported'): void
}>()

const store = useAppStore()

const showDialog = computed({
  get: () => props.show,
  set: (value) => emit('update:show', value)
})

const targetPath = ref('')
const exporting = ref(false)
const completedCount = ref(0)
const exportError = ref('')

const progress = computed(() => {
  if (store.selectedCount === 0) return 0
  return Math.round((completedCount.value / store.selectedCount) * 100)
})

const progressStatus = computed(() => {
  if (exportError.value) return 'error'
  if (progress.value === 100) return 'success'
  return 'default'
})

async function selectDirectory() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择导出目录'
    })
    if (selected) {
      targetPath.value = selected as string
    }
  } catch (e) {
    console.error('Failed to select directory:', e)
  }
}

async function handleExport() {
  if (!targetPath.value) return

  try {
    exporting.value = true
    exportError.value = ''
    completedCount.value = 0

    const result = await store.exportImages(targetPath.value)

    completedCount.value = result.completed

    if (result.failed.length > 0) {
      exportError.value = `${result.failed.length} 个文件导出失败`
    }

    if (result.completed > 0) {
      emit('exported')
      setTimeout(() => {
        showDialog.value = false
        reset()
      }, 1500)
    }
  } catch (e: any) {
    exportError.value = e.message || '导出失败'
  } finally {
    exporting.value = false
  }

  return false
}

function handleCancel() {
  reset()
}

function reset() {
  targetPath.value = ''
  completedCount.value = 0
  exportError.value = ''
  exporting.value = false
}
</script>

<style scoped>
.export-dialog {
  padding: 16px 0;
}

.export-info {
  margin-bottom: 16px;
}

.export-info p {
  margin: 0;
  color: var(--text-primary);
}

.target-path {
  margin-bottom: 16px;
}

.target-path label {
  display: block;
  margin-bottom: 8px;
  color: var(--text-secondary);
  font-size: 12px;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: 6px 10px;
  color: var(--text-primary);
  font-size: 12px;
}

.path-input button {
  background: var(--bg-elevated);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: 6px 12px;
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.path-input button:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--border-strong);
}

.path-input button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.export-progress {
  margin-top: 16px;
}

.progress-text {
  margin-top: 8px;
  color: var(--text-tertiary);
  font-size: 11px;
  text-align: center;
}

.export-error {
  margin-top: 16px;
  padding: 10px;
  background: rgba(212, 83, 83, 0.08);
  border: 1px solid rgba(212, 83, 83, 0.2);
  border-radius: var(--radius-sm);
}

.export-error p {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
}
</style>
