import httpRequest from '.';
import type { StandardResponse } from '@/utils/httpRequest';

export default class LruCacheApi {
  static downloadData(params: { key: string }) {
    return httpRequest.request<StandardResponse<any>>({
      url: '/api/lru',
      method: 'GET',
      params: params,
    });
  }

  static uploadData(params: { content: any }) {
    return httpRequest.request<StandardResponse<{
      key: string;
      size: number;
    }>>({
      url: '/api/lru',
      method: 'POST',
      data: {
        content: params.content,
      },
    });
  }
}
