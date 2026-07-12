import { createRouter, createWebHistory } from 'vue-router'
import MainPanel from '../views/MainPanelRedesign.vue'
import WidgetIsland from '../views/WidgetIsland.vue'

const router = createRouter({
    history: createWebHistory(),
    routes: [
        { path: '/', component: MainPanel },
        { path: '/widget', component: WidgetIsland }
    ]
})

export default router
