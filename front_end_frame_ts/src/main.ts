import App from './App.vue';
import router from './router';
import pinia from './stores';
import * as ElementPlusIconsVue from '@element-plus/icons-vue';

// global css
import 'normalize.css';
import 'animate.css';
import 'hover.css';

const app = createApp(App);

// element plus icon
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component);
}

app.use(pinia);
app.use(router);

app.mount('#app');
