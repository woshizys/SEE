import httpRequest from '.';
import type { StandardResponse } from '@/utils/httpRequest';

export default class LruCacheApi {
  static downloadData(params: { key: string }) {
    return httpRequest.request<Blob>({
      url: '/api/lru',
      method: 'GET',
      params: params,
    });
  }

  static uploadData(content: any) {
    const formData = new FormData()
    if (typeof content === 'string') {
      // 将文本转为 Blob 对象
      const blob = new Blob([content], { type: 'text/plain' })
      formData.append('file', blob, 'text-content.txt')
    } else {
      formData.append('file', content)
    }
    return httpRequest.request<StandardResponse<{
      key: string;
      size: number;
    }>>({
      url: '/api/lru',
      method: 'POST',
      data: formData,
    });
  }
}

import axios from 'axios'

type UploadableData = string | File

export async function uploadFile(data: UploadableData, apiUrl: string) {
  const formData = new FormData()
  
  // 统一用相同字段名（需与后端约定，示例用"file"）
  if (typeof data === 'string') {
    // 将文本转为 Blob 对象
    const blob = new Blob([data], { type: 'text/plain' })
    formData.append('file', blob, 'text-content.txt')
  } else {
    formData.append('file', data)
  }

  try {
    const response = await axios.post(apiUrl, formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })
    return response.data
  } catch (error) {
    throw new Error(`Upload failed: ${error instanceof Error ? error.message : 'Unknown error'}`)
  }
}
