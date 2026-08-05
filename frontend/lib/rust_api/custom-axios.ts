import { useAuthStore } from '@/src/store/useAuthStore';
import Axios from 'axios';
import type { AxiosRequestConfig } from 'axios';

export const AXIOS_INSTANCE = Axios.create({
  baseURL: '/', 
});

AXIOS_INSTANCE.interceptors.request.use((config) => {
  const token = useAuthStore.getState().token
  
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  
  return config
})

export const customAxios = <T>(
  config: AxiosRequestConfig,
  options?: AxiosRequestConfig
): Promise<T> => {
  const source = Axios.CancelToken.source();
  const promise = AXIOS_INSTANCE({
    ...config,
    ...options,
    cancelToken: source.token,
  }).then(({ data }) => data);

  // @ts-expect-error cancel attached by Orval
  promise.cancel = () => {
    source.cancel('Query was cancelled');
  };

  return promise;
};