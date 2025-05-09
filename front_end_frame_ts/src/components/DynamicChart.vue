<!-- LatencyChart.vue -->
<template>
    <div ref="chartRef" style="width: 100%; height: 500px"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, defineModel } from 'vue'
import * as echarts from 'echarts'
import type {
    ECharts,
    EChartsOption,
    TooltipComponentOption
} from 'echarts'

interface DataPoint {
    time: number
    x: number
    y: number
}

const props = defineProps({
    maxPoints: {
        type: Number,
        default: 100
    },
    chartOptions: {
        type: Object as () => EChartsOption,
        default: () => ({})
    },
    updateInterval: {
        type: Number,
        default: 1000 // 默认1秒更新一次时间轴
    },
    // 历史数据保留时长（毫秒）
    historyDuration: {
        type: Number,
        default: 60000 // 默认保留1分钟
    },
})

const xModel = defineModel<number>('x', { required: true })
const yModel = defineModel<number>('y', { required: true })

const chartRef = ref<HTMLElement>()
const chart = ref<ECharts>()
const dataList = ref<DataPoint[]>([])
let updateTimer: number | null = null

// 基础图表配置
const baseOptions: EChartsOption = {
    tooltip: {
        trigger: 'axis',
        formatter: (params: any) => {
            const date = new Date(params[0].value[0])
            return `
          ${date.toLocaleTimeString()}<br/>
          X: ${params[0].value[1]}ms<br/>
          Y: ${params[1].value[1]}ms
        `
        }
    } as TooltipComponentOption,
    xAxis: {
        type: 'time',
        splitLine: { show: false }
    },
    yAxis: [
        {
            name: '请求间隔 (ms)',
            min: 0,
            splitLine: { lineStyle: { type: 'dashed' } }
        },
        {
            name: '响应延迟 (ms)',
            min: 0,
            splitLine: { lineStyle: { type: 'dashed' } }
        }
    ],
    series: [
        {
            name: 'X',
            type: 'line',
            yAxisIndex: 0,
            showSymbol: false,
            smooth: true,
            lineStyle: { width: 2 }
        },
        {
            name: 'Y',
            type: 'line',
            yAxisIndex: 1,
            showSymbol: false,
            smooth: true,
            lineStyle: {
                width: 2,
                type: 'dashed'
            }
        }
    ]
}

// 添加数据点（带时间戳）
const addDataPoint = () => {
    const now = Date.now()
    dataList.value.push({
        time: Date.now(),
        x: xModel.value,
        y: yModel.value
    })

    // 保持数据长度
    // if (dataList.value.length > props.maxPoints) {
    //   dataList.value.shift()
    // }
    // 清理过期数据
    dataList.value = dataList.value.filter(
        item => now - item.time <= props.historyDuration
    )

    updateChart()
}

// 更新图表
const updateChart = () => {
    chart.value?.setOption({
        series: [
            { data: dataList.value.map(d => [d.time, d.x]) },
            { data: dataList.value.map(d => [d.time, d.y]) }
        ]
    })
}

// 初始化定时器
const initTimer = () => {
    updateTimer = setInterval(() => {
        addDataPoint() // 定期添加当前值
    }, props.updateInterval)
}

// 初始化图表
const initChart = () => {
    if (!chartRef.value) return
    chart.value = echarts.init(chartRef.value)
    chart.value.setOption({ ...baseOptions, ...props.chartOptions })
}

// 监听窗口变化
const handleResize = () => chart.value?.resize()

// 监听数值变化（立即更新）
watch([xModel, yModel], () => {
    addDataPoint()
})

onMounted(() => {
    initChart()
    initTimer()
    window.addEventListener('resize', handleResize)
})

onBeforeUnmount(() => {
    if (updateTimer) clearInterval(updateTimer)
    window.removeEventListener('resize', handleResize)
    chart.value?.dispose()
})
</script>