// 配置选项类型
interface HttpLatencyTrackerOptions {
    windowMs: number; // 统计时间窗口（毫秒）
    cleanupInterval?: number; // 清理间隔（毫秒），默认1000ms
}

// 延迟记录类型
interface LatencyRecord {
    timestamp: number; // 时间戳（毫秒）
    latency: number;   // 延迟时间（毫秒）
}

/**
 * 跟踪HTTP请求延迟的组合式函数（支持时间窗口配置）
 * @param options 配置选项
 * @returns {Object} 包含跟踪方法和统计数据的对象
 */
export function useHttpLatencyTracker(
    options: HttpLatencyTrackerOptions
) {
    const { windowMs, cleanupInterval = 1000 } = options;
    const latencyRecords: Ref<LatencyRecord[]> = ref([]);
    const avgLatency: Ref<number | null> = ref(null);
    let cleanupTimer: ReturnType<typeof setInterval>;

    // 清理旧记录（保留最近windowMs内的数据）
    const cleanupOldRecords = () => {
        const now = Date.now();
        latencyRecords.value = latencyRecords.value.filter(
            record => now - record.timestamp <= windowMs
        );
    };

    // 计算当前平均延迟
    const calculateAvg = () => {
        if (latencyRecords.value.length === 0) {
            avgLatency.value = null;
            return;
        }
        const total = latencyRecords.value.reduce(
            (sum, record) => sum + record.latency, 0
        );
        avgLatency.value = Math.round(total / latencyRecords.value.length);
    };

    // 初始化跟踪器
    const initTracker = () => {
        cleanupTimer = setInterval(() => {
            cleanupOldRecords();
            calculateAvg();
        }, cleanupInterval);
    };

    // 跟踪请求延迟（支持泛型返回类型）
    const trackRequest = async <T>(requestPromise: Promise<T>): Promise<T> => {
        const startTime = Date.now();
        try {
            const result = await requestPromise;
            const latency = Date.now() - startTime;
            latencyRecords.value.push({ timestamp: startTime, latency });
            return result;
        } catch (error) {
            // 记录失败请求的延迟（包含错误处理时间）
            const latency = Date.now() - startTime;
            latencyRecords.value.push({ timestamp: startTime, latency });
            throw error;
        }
    };

    // 组件卸载清理
    onUnmounted(() => {
        clearInterval(cleanupTimer);
    });

    initTracker();

    return {
        trackRequest,
        avgLatency,
        latencyRecords,
        windowMs: ref(windowMs) // 暴露当前窗口配置（响应式）
    };
}
    