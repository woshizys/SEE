import httpRequest from '.';
import type { StandardResponse } from '@/utils/httpRequest';

export default class ExampleApi {
  static exampleRequest(param: any) {
    return httpRequest.request<StandardResponse<string>>({
      url: '/api/exmaple',
      method: 'GET',
      params: param,
    });
  }
}
