<template>
  <Teleport to="body">
    <div v-if="show" class="dialog-overlay" @click.self="$emit('update:show', false)">
      <div class="dialog">
        <div class="dialog-header">
          <h3>分组管理</h3>
          <button class="dialog-close" @click="$emit('update:show', false)">×</button>
        </div>
        <div class="dialog-body">
          <div v-if="store.groups.length === 0" class="empty-hint">暂无分组</div>
          <div v-for="g in store.groups" :key="g.id" class="group-item">
            <span class="group-dot" :style="{ background: g.color }"></span>
            <template v-if="editingId === g.id">
              <input v-model="editingName" class="edit-input" @keydown.enter="saveEdit(g.id)" @blur="saveEdit(g.id)" autofocus />
              <input
                v-model="editingShortcut"
                class="edit-input shortcut"
                readonly
                placeholder="按一个键"
                @focus="listeningShortcut = true"
                @blur="listeningShortcut = false"
                @keydown.prevent="captureShortcut"
              />
            </template>
            <template v-else>
              <span class="group-name">{{ g.name }}</span>
              <span class="group-shortcut">{{ g.shortcut }}</span>
            </template>
            <div class="group-actions">
              <button v-if="editingId !== g.id" class="icon-btn" @click="startEdit(g)" title="编辑">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
              </button>
              <button class="icon-btn danger" @click="store.deleteGroup(g.id)" title="删除">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useAppStore } from '../stores/app'

const props = defineProps<{ show: boolean }>()
defineEmits<{ 'update:show': [val: boolean] }>()

const store = useAppStore()
const editingId = ref<string | null>(null)
const editingName = ref('')
const editingShortcut = ref('')
const listeningShortcut = ref(false)

function startEdit(g: { id: string; name: string; shortcut: string }) {
  editingId.value = g.id
  editingName.value = g.name
  editingShortcut.value = g.shortcut
}

function captureShortcut(e: KeyboardEvent) {
  if (['Shift', 'Control', 'Alt', 'Meta', 'Tab', 'Escape', 'CapsLock'].includes(e.key)) return
  editingShortcut.value = e.key.length === 1 ? e.key.toLowerCase() : e.key
  listeningShortcut.value = false
}

function saveEdit(id: string) {
  if (editingName.value.trim()) {
    store.renameGroup(id, editingName.value.trim())
  }
  if (editingShortcut.value) {
    store.setGroupShortcut(id, editingShortcut.value)
  }
  editingId.value = null
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
  z-index: 3000;
}
.dialog {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  width: 400px;
  max-height: 60vh;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
}
.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-subtle);
}
.dialog-header h3 { margin: 0; font-size: 14px; font-weight: 600; color: var(--text-primary); }
.dialog-close { background: none; border: none; font-size: 18px; color: var(--text-tertiary); cursor: pointer; }
.dialog-close:hover { color: var(--text-primary); }
.dialog-body {
  padding: 12px 16px;
  overflow-y: auto;
}
.empty-hint { text-align: center; color: var(--text-tertiary); font-size: 12px; padding: 20px 0; }
.group-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-subtle);
}
.group-item:last-child { border-bottom: none; }
.group-dot { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
.group-name { flex: 1; font-size: 13px; color: var(--text-primary); }
.group-shortcut { font-size: 11px; color: var(--text-tertiary); font-weight: 600; padding: 2px 6px; background: var(--bg-elevated); border-radius: 3px; }
.group-actions { display: flex; gap: 4px; }
.icon-btn {
  display: flex; align-items: center; justify-content: center;
  width: 24px; height: 24px;
  background: transparent; border: none; border-radius: var(--radius-sm);
  color: var(--text-tertiary); cursor: pointer;
}
.icon-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
.icon-btn.danger:hover { color: var(--danger); }
.edit-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--bg-base);
  border: 1px solid var(--accent);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
}
.edit-input.shortcut { width: 40px; flex: none; text-align: center; font-weight: 600; }
</style>
