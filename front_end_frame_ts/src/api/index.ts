import { HttpRequest } from '../utils/httpRequest';

const configApiUrl: string | null = window.APP_CONFIG?.API_URL;

const httpRequest = new HttpRequest({
  baseURL: configApiUrl || import.meta.env.VITE_API_DOMAIN,
  timeout: import.meta.env.VITE_API_TIME_OUT,
  interceptors: {
    requestInterceptors(config) {
      // console.log('add token to head');
      const token = 'Bearer ' + window.localStorage.getItem('token');
      if (token) {
        config.headers.Authorization = String(token);
      }
      return config;
    },
  },
});

export default httpRequest;
