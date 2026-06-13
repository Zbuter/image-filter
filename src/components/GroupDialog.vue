<template>
  <Teleport to="body">
    <div v-if="show" class="dialog-overlay" @click.self="$emit('update:show', false)">
      <div class="dialog">
        <div class="dialog-header">
          <h3>{{ editId ? '编辑分组' : '新建分组' }}</h3>
          <button class="dialog-close" @click="$emit('update:show', false)">×</button>
        </div>
        <div class="dialog-body">
          <div class="form-row">
            <label>分组名称</label>
            <input v-model="name" type="text" placeholder="输入分组名称" class="form-input" maxlength="5" autofocus />
          </div>
          <div class="form-row">
            <label>快捷键</label>
            <input
              v-model="shortcut"
              type="text"
              placeholder="点击后按一个键"
              class="form-input shortcut-input"
              readonly
              @focus="listeningShortcut = true"
              @blur="listeningShortcut = false"
              @keydown.prevent="captureShortcut"
            />
            <span class="form-hint" v-if="listeningShortcut">请按下快捷键...</span>
            <span class="form-hint" v-else>数字、字母均可</span>
          </div>
        </div>
        <div class="dialog-footer">
          <button class="btn btn-ghost" @click="$emit('update:show', false)">取消</button>
          <button class="btn btn-primary" @click="confirm" :disabled="!name.trim()">确定</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  show: boolean
  editId?: string
  editName?: string
  editShortcut?: string
}>()

const emit = defineEmits<{
  'update:show': [val: boolean]
  confirm: [name: string, shortcut: string]
}>()

const name = ref('')
const shortcut = ref('')
const listeningShortcut = ref(false)

watch(() => props.show, (val) => {
  if (val) {
    name.value = props.editName || ''
    shortcut.value = props.editShortcut || ''
  }
})

function captureShortcut(e: KeyboardEvent) {
  if (['Shift', 'Control', 'Alt', 'Meta', 'Tab', 'Escape', 'CapsLock'].includes(e.key)) return
  shortcut.value = e.key.length === 1 ? e.key.toLowerCase() : e.key
  listeningShortcut.value = false
}

function confirm() {
  if (!name.value.trim()) return
  emit('confirm', name.value.trim(), shortcut.value)
  emit('update:show', false)
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3100;
}

.dialog {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  width: 360px;
  box-shadow: var(--shadow-lg);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-subtle);
}

.dialog-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.dialog-close {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 0 4px;
}

.dialog-close:hover { color: var(--text-primary); }

.dialog-body {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-row label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-input {
  padding: 6px 10px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
}

.form-input:focus {
  border-color: var(--accent);
}

.shortcut-input {
  cursor: pointer;
  text-align: center;
  font-weight: 600;
  letter-spacing: 2px;
}

.form-hint {
  font-size: 11px;
  color: var(--text-tertiary);
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-subtle);
}

.btn {
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  cursor: pointer;
  border: 1px solid var(--border-subtle);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
}

.btn-ghost:hover { background: var(--bg-hover); }

.btn-primary {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}

.btn-primary:hover { opacity: 0.9; }
.btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
</style>
