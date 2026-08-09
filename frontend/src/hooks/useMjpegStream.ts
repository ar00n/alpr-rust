import { useMjpegStreamHandler } from '@/lib/rust_api/schema';
import { useEffect, useRef, useState, useCallback } from 'react';

interface UseMjpegStreamProps {
  queryParams?: Record<string, any>;
  autoReconnect?: boolean; // Default: true
  reconnectInterval?: number; // Delay in ms before retry, default: 2000
  maxRetries?: number; // Max retries before entering 'failed' state, default: Infinity
}

export type StreamStatus = 'idle' | 'connected' | 'reconnecting' | 'failed';

export function useMjpegStream(props?: UseMjpegStreamProps) {
  const {
    queryParams,
    autoReconnect = true,
    reconnectInterval = 2000,
    maxRetries = Infinity,
  } = props || {};

  const imgRef = useRef<HTMLImageElement>(null);
  const [streamStatus, setStreamStatus] = useState<StreamStatus>('idle');
  
  const retryCountRef = useRef(0);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const {
    data: response,
    status: queryStatus,
    error,
    refetch,
  } = useMjpegStreamHandler({
    query: {
      gcTime: 0,
      staleTime: Infinity,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
    },
    request: {
      adapter: 'fetch',
      responseType: 'stream',
      params: queryParams,
    },
  });

  const clearReconnectTimeout = () => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }
  };

  const scheduleReconnect = useCallback(() => {
    clearReconnectTimeout();

    if (!autoReconnect) {
      setStreamStatus('failed');
      return;
    }

    if (retryCountRef.current < maxRetries) {
      retryCountRef.current += 1;
      setStreamStatus('reconnecting');

      reconnectTimeoutRef.current = setTimeout(() => {
        refetch();
      }, reconnectInterval);
    } else {
      setStreamStatus('failed');
    }
  }, [autoReconnect, maxRetries, reconnectInterval, refetch]);

  useEffect(() => {
    if (!response) return;

    let isCancelled = false;
    let reader: ReadableStreamDefaultReader<Uint8Array> | undefined;

    async function startReading() {
      try {
        // @ts-expect-error
        let streamBody: ReadableStream<Uint8Array> | null = response;

        if (!streamBody) {
          throw new Error('Response body is null or not a ReadableStream');
        }

        reader = streamBody.getReader();
        let buffer = new Uint8Array();
        let isConnectedSet = false;

        while (!isCancelled) {
          const { value, done } = await reader.read();

          if (done) {
            if (!isCancelled) {
              scheduleReconnect();
            }
            break;
          }

          if (!isConnectedSet) {
            retryCountRef.current = 0;
            setStreamStatus('connected');
            isConnectedSet = true;
          }

          // Append incoming chunk to buffer
          const newBuffer = new Uint8Array(buffer.length + value.length);
          newBuffer.set(buffer);
          newBuffer.set(value, buffer.length);
          buffer = newBuffer;

          let searching = true;
          while (searching) {
            let start = -1;

            // Search for JPEG SOI (Start of Image) marker: 0xFF 0xD8
            for (let i = 0; i < buffer.length - 1; i++) {
              if (buffer[i] === 0xff && buffer[i + 1] === 0xd8) {
                start = i;
                break;
              }
            }

            if (start !== -1) {
              // Search for JPEG EOI (End of Image) marker: 0xFF 0xD9 after SOI
              let end = -1;
              for (let i = start + 2; i < buffer.length - 1; i++) {
                if (buffer[i] === 0xff && buffer[i + 1] === 0xd9) {
                  end = i;
                  break;
                }
              }

              if (end !== -1) {
                // JPEG frame ends after 0xD9 (hence end + 2)
                const frameEnd = end + 2;
                const jpegData = buffer.slice(start, frameEnd);
                const blob = new Blob([jpegData], { type: 'image/jpeg' });
                const objectUrl = URL.createObjectURL(blob);

                if (imgRef.current) {
                  if (imgRef.current.src) {
                    URL.revokeObjectURL(imgRef.current.src);
                  }
                  imgRef.current.src = objectUrl;
                }

                // Advance buffer past this complete frame
                buffer = buffer.slice(frameEnd);
              } else {
                // Incomplete frame; drop leading boundary/header data up to SOI
                if (start > 0) {
                  buffer = buffer.slice(start);
                }
                searching = false;
              }
            } else {
              // No SOI marker found in buffer.
              // Retain last byte in case 0xFF is split across chunks.
              if (buffer.length > 1) {
                buffer = buffer.slice(buffer.length - 1);
              }
              searching = false;
            }
          }
        }
      } catch (err) {
        if (!isCancelled) {
          scheduleReconnect();
        }
      }
    }

    startReading();

    return () => {
      isCancelled = true;
      if (reader) {
        reader.cancel().catch(() => {});
      }
      if (imgRef.current?.src) {
        URL.revokeObjectURL(imgRef.current.src);
      }
    };
  }, [response, scheduleReconnect]);

  useEffect(() => {
    if (queryStatus === 'error') {
      scheduleReconnect();
    }
  }, [queryStatus, scheduleReconnect]);

  useEffect(() => {
    return () => {
      clearReconnectTimeout();
    };
  }, []);

  const manualReconnect = () => {
    clearReconnectTimeout();
    retryCountRef.current = 0;
    setStreamStatus('reconnecting');
    refetch();
  };

  return {
    imgRef,
    status: streamStatus,
    error,
    reconnect: manualReconnect,
  };
}