import { createApp, h } from 'vue'
import { createPinia } from 'pinia'
import { NConfigProvider, darkTheme } from 'naive-ui'
import App from './App.vue'
import './style.css'

const app = createApp({
  render() {
    return h(NConfigProvider, { theme: darkTheme }, () => h(App))
  }
})

app.use(createPinia())
app.mount('#app')
