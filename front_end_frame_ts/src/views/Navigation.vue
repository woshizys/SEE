<script setup lang="ts">
const route = useRoute()
// const router = useRouter();

const activeIndex = computed(() => route.path)

const handleSelect = (key: any, keyPath: any) => {
  console.log('selectRouter', key, keyPath);
};
</script>

<template>
   <el-menu
    mode="horizontal"
    :ellipsis="false"
    background-color="#545c64"
    text-color="#fff"
    active-text-color="#ffd04b"
    :default-active="activeIndex"
    router
    @select="handleSelect"
  >
    <el-menu-item index="home">主页</el-menu-item>
    <el-menu-item index="binary">页面1</el-menu-item>
    <el-menu-item index="numeric">页面2</el-menu-item>

    <!-- 分左右 -->
    <div style="flex-grow: 1;" />
    
    <div>右边的东西</div>
  </el-menu>
</template>

<style scoped>

</style>