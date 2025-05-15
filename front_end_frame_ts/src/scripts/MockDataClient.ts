import LruCacheApi from "@/api/lru_cache";
import { generateRandomString } from "@/utils";

export default class MockDataClient {
    private isCacheOn: number = 0;
    private db: { [key: string]: string } = {}; // 模拟数据库存储

    turnOffCache() {
        this.isCacheOn = 0;
    }

    turnOnCache() {
        this.isCacheOn = 1;
    }

    // 缓存访问接口
    private async cacheAccess(key: string): Promise<string | null> {
        return await LruCacheApi.downloadData({ key }).then((res) => {
            return res.data as unknown as string
        }).catch((err) => {
            // cache miss
            return null
        });
    }

    // 缓存上传接口
    private async cacheUpload(data: string): Promise<string | null> {
        return await LruCacheApi.uploadData(data).then((res) => {
            console.log('上传到cache', data, res);
            return res.data.data.key
        }).catch((err) => {
            console.error('err', err);
            return null
        });
    }

    // 模拟数据库访问（带随机延迟）
    private async fetchFromDB(key: string): Promise<string | null> {
        // 生成随机延迟
        const delay = Math.floor(Math.random() * 200) + 200;
        await new Promise(resolve => setTimeout(resolve, delay));

        return this.db[key] ?? null;
    }

    // 核心下载接口
    async downloadData(key: string): Promise<string | null> {
        if (this.isCacheOn === 0) {
            console.log("无cache直接下载数据", key);
            return await this.fetchFromDB(key);
        }

        // 缓存优先策略
        const cacheData = await this.cacheAccess(key);
        if (cacheData !== null) {
            console.log("Cache命中", key);
            return cacheData;
        }

        // 缓存未命中时访问数据库
        const dbData = await this.fetchFromDB(key);
        if (dbData !== null) {
            console.log("Cache未命中", key);
            await this.cacheUpload(dbData); // 数据库数据同步到缓存
            return dbData;
        }
        return null;
    }

    // =======测试用========

    // 向数据库添加测试数据
    private addDbData(key: string, value: string): void {
        this.db[key] = value;
    }

    // 初始化测试数据，生成指定数量的指定大小的数据。
    async initTestData(dataNum: number, dataSize: number): Promise<void> {
        console.log("初始化测试数据，个数：", dataNum, "大小：", dataSize);
        for (let i = 0; i < dataNum; i++) {
            const value = generateRandomString(dataSize);
            const key = await this.cacheUpload(value) as string;
            this.addDbData(key, value);
        }
    }

    // 随机下载一个数据库里的数据
    async randomDownload(): Promise<string | null> {
        const dbKeys = Object.keys(this.db);
        if (dbKeys.length === 0) {
            console.log("数据库为空，无法随机下载");
            return null;
        }

        const randomIndex = Math.floor(Math.random() * dbKeys.length);
        const randomKey = dbKeys[randomIndex];
        return await this.downloadData(randomKey);
    }
}

