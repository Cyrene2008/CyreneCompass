import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import 'vue-fluent-widgets/style.css'
import './styles.css'

createApp(App).use(createPinia()).mount('#app')
