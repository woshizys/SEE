/**
 * Execute every specified time, and the number of executions can be specified. 
 * Executed once at the moment of call.
 * @param time Interval time
 * @param num Number of executions
 * @param func Function to be executed
 * @param callback Executed when last `func` done
 */
export function callPerPeriod(
  time: number,
  num: number,
  func: Function = () => { },
  callback: Function = () => { }
) {
  func();
  num--;
  if (num <= 0) {
    callback();
    console.log('complete count');
    return;
  }
  setTimeout(() => callPerPeriod(time, num, func, callback), time);
}

export function generateRandomString(length: number): string {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    const randomIndex = Math.floor(Math.random() * characters.length);
    result += characters.charAt(randomIndex);
  }
  return result;
}

/**
 * 模拟网络拥塞的异步函数
 * 特性：请求频率越高（1秒内调用次数越多），返回延迟越长
 */
export async function simulateNetworkCongestion(base: number, factor: number): Promise<void> {
  return new Promise((resolve) => {
    const now = Date.now();

    // 1. 记录当前请求时间戳（添加到共享数组）
    requestTimestamps.push(now);

    // 2. 清理超过1秒的旧记录（仅保留最近1秒内的请求）
    const oneSecondAgo = now - 1000;
    // 过滤掉早于1秒前的时间戳，保留最近1秒内的记录
    const recentRequests = requestTimestamps.filter(timestamp => timestamp > oneSecondAgo);
    // 用过滤后的结果覆盖原数组（清理旧数据）
    requestTimestamps.length = 0;
    requestTimestamps.push(...recentRequests);

    // 3. 根据当前频率计算延迟（频率=最近1秒内的请求次数）
    const requestFrequency = recentRequests.length;
    const delay = base + requestFrequency * factor; // 基础延迟+频率相关延迟

    // 4. 模拟延迟
    setTimeout(() => {
      // console.log(`当前频率：${requestFrequency}次/秒 → 延迟：${delay}ms`);
      resolve();
    }, delay);
  });
}
const requestTimestamps: number[] = [];

