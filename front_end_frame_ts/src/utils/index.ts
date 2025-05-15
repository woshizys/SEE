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
  func: Function = () => {},
  callback: Function = () => {}
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
