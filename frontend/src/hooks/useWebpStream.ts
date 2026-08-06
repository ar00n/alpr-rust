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

            // Search for WebP / RIFF header:
            // "RIFF" [4 bytes size] "WEBP" (requires at least 12 bytes)
            for (let i = 0; i <= buffer.length - 12; i++) {
              if (
                buffer[i] === 0x52 &&     // 'R'
                buffer[i + 1] === 0x49 && // 'I'
                buffer[i + 2] === 0x46 && // 'F'
                buffer[i + 3] === 0x46 && // 'F'
                buffer[i + 8] === 0x57 && // 'W'
                buffer[i + 9] === 0x45 && // 'E'
                buffer[i + 10] === 0x42 &&// 'B'
                buffer[i + 11] === 0x50   // 'P'
              ) {
                start = i;
                break;
              }
            }

            if (start !== -1) {
              // Read 32-bit Little-Endian uint payload size from offset start + 4
              const payloadSize =
                (buffer[start + 4] |
                  (buffer[start + 5] << 8) |
                  (buffer[start + 6] << 16) |
                  (buffer[start + 7] << 24)) >>> 0;

              // Total WebP file size = RIFF payload + 8 bytes header ('RIFF' + 4-byte size uint)
              const totalFrameSize = payloadSize + 8;

              // Check if the complete WebP frame has arrived in the buffer
              if (buffer.length >= start + totalFrameSize) {
                const webpData = buffer.slice(start, start + totalFrameSize);
                const blob = new Blob([webpData], { type: 'image/webp' });
                const objectUrl = URL.createObjectURL(blob);

                if (imgRef.current) {
                  if (imgRef.current.src) {
                    URL.revokeObjectURL(imgRef.current.src);
                  }
                  imgRef.current.src = objectUrl;
                }

                // Advance buffer past this frame (also clears boundary text preceding 'start')
                buffer = buffer.slice(start + totalFrameSize);
              } else {
                // Incomplete frame; wait for next stream chunk from reader
                searching = false;
              }
            } else {
              // No full RIFF/WEBP header found in current buffer.
              // Keep the last 11 bytes in case a header was split across stream chunks.
              if (buffer.length > 11) {
                buffer = buffer.slice(buffer.length - 11);
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