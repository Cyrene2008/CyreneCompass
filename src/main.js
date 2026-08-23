import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import CompassOverlay from './CompassOverlay.vue'
import 'vue-fluent-widgets/style.css'
import './styles.css'
import { getCurrentWindow } from '@tauri-apps/api/window'

// 桌面罗盘模式使用独立的全屏 overlay 窗口，按窗口标签分流入口
const isOverlay = getCurrentWindow().label === 'compass-overlay'

createApp(isOverlay ? CompassOverlay : App).use(createPinia()).mount('#app')
