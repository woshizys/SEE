<script setup lang="ts">
import LruCacheApi from '@/api/lru_cache';
import DynamicChart from '@/components/DynamicChart.vue';
import { useHttpLatencyTracker } from '@/scripts/LatencyCalculator';

const test_env = () => {
  console.log(
    'import.meta.env.VITE_API_DOMAIN',
    import.meta.env.VITE_API_DOMAIN
  );
  console.debug('import.meta.env.BASE_URL', import.meta.env.BASE_URL);
};

const downloadKey = ref('');
const downloadData = () => {
  console.log('start download data from', downloadKey.value);
  LruCacheApi.downloadData({ key: downloadKey.value })
    .then((res) => {
      console.log('res', res);
    })
    .catch((err) => {
      console.error('err', err);
    });
}

const uploadContentText = ref('');
const uploadKey = ref('');
const uploadData = () => {
  console.log('start upload text data', uploadContentText.value);
  LruCacheApi.uploadData({ content: uploadContentText.value })
    .then((res) => {
      console.log('res', res);
    })
    .catch((err) => {
      console.error('err', err);
    });
}

const isTurnOnCache = ref(false);

const minFrequency = 1;
const maxFrequency = 100;
const reqFrequency = ref(minFrequency);
const isRunning = ref(false);
let intervalId: number | null = null;

const handleValueChange = (newValue: number) => {
  // reqFrequency.value = Math.min(Math.max(newValue, minFrequency), maxFrequency);
  reqFrequency.value = newValue;
  if (isRunning.value) {
    clearInterval(intervalId as number);
    startInterval();
  }
};

const handleInputChange = (inputValue: string) => {
  const numValue = Number(inputValue);
  if (!isNaN(numValue)) {
    handleValueChange(numValue);
  }
};

const startInterval = () => {
  intervalId = setInterval(() => {
    for (let i = 0; i < reqFrequency.value; i++) {
      // 直接调用trackRequest，不使用await
      trackRequest(mockRequest())
        .catch(error => {
          console.error(`第${i + 1}次请求失败:`, error);
        });
    }
  }, 1000);
};

const stopInterval = () => {
  if (intervalId) {
    clearInterval(intervalId);
  }
};

const toggleRunning = () => {
  if (isRunning.value) {
    startInterval();
  } else {
    stopInterval();
  }
};

// 配置5秒窗口（可修改为其他值如10000=10秒）
const { trackRequest, avgLatency, windowMs } = useHttpLatencyTracker({
  windowMs: 5000,       // 核心配置：统计窗口时间
  cleanupInterval: 500  // 可选配置：清理间隔（默认1000ms）
});

// 记录延迟，转化为数字
const displayLatency = computed<number>({
  get() {
    return avgLatency.value ?? 0; // 保持null时显示'-'
  },
  set(newValue) {
  }
});

function mockRequest(): Promise<void> {
  return new Promise((resolve) => {
    // 生成100-1500ms的随机延迟
    const delay = Math.floor(Math.random() * reqFrequency.value * 14) + 100;
    setTimeout(() => resolve(), delay);
  });
}

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});
</script>

<template>
  <div class="image-example">
    <WrappedImage></WrappedImage>
  </div>
  <br />
  <el-input v-model="downloadKey" placeholder="input key"></el-input>
  <el-button type="info" size="default" @click="downloadData">Download Data From LRU Cache</el-button>
  <br />
  <el-input v-model="uploadContentText" placeholder="text to upload"></el-input>
  <el-text>Key of the last uploaded data: {{ uploadKey }}</el-text>
  <br />
  <el-button type="info" size="default" @click="uploadData">Upload Text Data to LRU Cache</el-button>

  <div>
    <span>Cache Switch: </span><el-switch v-model="isTurnOnCache" size="large" />
    <br />
    <span>Request Switch: </span><el-switch v-model="isRunning" @change="toggleRunning" />
    <div class="slider-demo-block">
      <el-input v-model="reqFrequency" type="number" @input="handleInputChange" />
      <el-slider v-model="reqFrequency" />
    </div>
    <DynamicChart v-model:x="reqFrequency" v-model:y="displayLatency" :max-points="200" />
  </div>
</template>

<style scoped>
.image-example {
  border: 0.2rem solid #a459c7;
  width: 15rem;
  height: 10rem;
  display: flex;
  flex-wrap: wrap;
}

.slider-demo-block {
  max-width: 600px;
  display: flex;
  align-items: center;
}

.slider-demo-block .el-slider {
  margin-top: 0;
  margin-left: 12px;
}

.slider-demo-block .demonstration {
  font-size: 14px;
  color: var(--el-text-color-secondary);
  line-height: 44px;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-bottom: 0;
}

.slider-demo-block .demonstration+.el-slider {
  flex: 0 0 70%;
}

.demonstration {
  background-color: #111111;
  color: white;
}
</style>
