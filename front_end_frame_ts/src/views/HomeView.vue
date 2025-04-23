<script setup lang="ts">
import LruCacheApi from '@/api/lru_cache';

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
  LruCacheApi.downloadData({ key: inputKey.value })
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
</template>

<style scoped>
.image-example {
  border: 0.2rem solid #a459c7;
  width: 15rem;
  height: 10rem;
  display: flex;
  flex-wrap: wrap;
}
</style>
