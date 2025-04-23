<script setup lang="ts">
const props = defineProps(['src', 'fit']);
</script>

<template>
  <el-image :src="props.src" :fit="props.fit" class="image">
    <!-- Default Display -->
    <template #error>
      <div class="image_slot">
        <el-icon><Picture /></el-icon>
      </div>
    </template>
  </el-image>
</template>

<style scoped>
.image {
  width: 100%;
  height: 100%;
  /* min-height: 4rem; */
}

.image_slot {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
  background: var(--el-fill-color-dark);
  color: var(--el-text-color-secondary);
  font-size: 30px;
}
</style>
