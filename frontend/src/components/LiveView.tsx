import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Camera, AlertCircle, Loader2, RefreshCw, Clock, RotateCw, CheckCircle2, XCircle } from 'lucide-react';
import type { PlateRead } from '@/lib/rust_api/schema';
import { useMjpegStream } from '@/src/hooks/useMjpegStream';
import { cn } from '@/lib/utils';

interface LiveViewProps {
  recentPlates: PlateRead[];
  isLoading?: boolean;
  isError?: boolean;
  onReload?: () => void;
}

export default function LiveView({ recentPlates, isLoading, isError, onReload }: LiveViewProps) {
  const { imgRef, status, reconnect } = useMjpegStream();

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Camera size={20} />
            Live Camera Feed
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="relative w-full aspect-video bg-slate-900 rounded-lg overflow-hidden flex items-center justify-center border-4 border-slate-800">
            {/* Camera Stream Image Frame */}
            <img
              ref={imgRef}
              className={`w-full h-full object-cover ${
                status !== 'connected' ? 'hidden' : 'block'
              }`}
              alt="Live stream"
            />

            {/* Connecting / Reconnecting State */}
            {(status === 'idle' || status === 'reconnecting') && (
              <div className="absolute inset-0 bg-slate-900/80 flex flex-col items-center justify-center gap-2 text-slate-300 backdrop-blur-xs">
                <Loader2 size={36} className="animate-spin text-white" />
                <p className="text-sm font-medium">
                  {status === 'reconnecting'
                    ? 'Reconnecting to camera stream...'
                    : 'Connecting to camera stream...'}
                </p>
              </div>
            )}

            {/* Connection Failed State */}
            {status === 'failed' && (
              <div className="absolute inset-0 bg-slate-900/90 flex flex-col items-center justify-center gap-3 text-red-400">
                <AlertCircle size={36} />
                <div className="text-center">
                  <p className="text-sm font-medium">Failed to establish connection</p>
                  <p className="text-xs text-slate-400 mt-1">
                    Check your network or server availability
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={reconnect}
                  className="mt-1 gap-2 text-slate-200 border-slate-700 bg-slate-800 hover:bg-slate-700 hover:text-white"
                >
                  <RefreshCw size={14} />
                  Try Reconnecting
                </Button>
              </div>
            )}

            {/* Live Indicator */}
            {status === 'connected' && (
              <div className="absolute top-4 right-4 flex items-center gap-2 pointer-events-none">
                <span className="relative flex h-3 w-3">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
                  <span className="relative inline-flex rounded-full h-3 w-3 bg-red-500"></span>
                </span>
                <span className="text-white text-xs font-medium uppercase tracking-wider bg-black/50 px-2 py-1 rounded backdrop-blur-sm">
                  LIVE
                </span>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
          <CardTitle className="flex items-center gap-2 text-lg">
            <Clock className="h-5 w-5 text-primary" />
            Recent Detections
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isError ? (
            <div className="flex flex-col items-center justify-center py-12 text-center text-muted-foreground gap-3">
              <div className="flex items-center gap-2 text-destructive font-medium">
                <AlertCircle className="h-5 w-5" />
                <span>Failed to load recent detections</span>
              </div>
              {onReload && (
                <Button variant="outline" size="sm" onClick={() => onReload()} className="gap-2">
                  <RotateCw className="h-4 w-4" />
                  Reload
                </Button>
              )}
            </div>
          ) : isLoading ? (
            <div className="flex items-center justify-center py-12 text-muted-foreground gap-2">
              <Loader2 className="h-5 w-5 animate-spin text-primary" />
              <span>Loading detections...</span>
            </div>
          ) : recentPlates.length > 0 ? (
            <div className="space-y-3">
              {recentPlates.map((plate) => (
                <div
                  key={plate.id}
                  className={cn(
                    "flex items-center justify-between p-3.5 rounded-lg border bg-card shadow-sm hover:bg-accent/40 transition-all duration-150",
                    plate.was_allowed
                      ? "border-l-4 border-l-emerald-500"
                      : "border-l-4 border-l-destructive"
                  )}
                >
                  <div className="flex items-center gap-3">
                    {/* Status Icon */}
                    {plate.was_allowed ? (
                      <CheckCircle2 className="h-5 w-5 text-emerald-500 shrink-0" />
                    ) : (
                      <XCircle className="h-5 w-5 text-destructive shrink-0" />
                    )}

                    <div className="space-y-1">
                      {/* Plate Badge Display */}
                      <span className={cn("inline-flex items-center rounded-md border border-border/80 bg-muted/50 px-2.5 py-1 font-mono text-xs font-bold tracking-widest text-foreground shadow-xs transition-colors select-none whitespace-nowrap")}>
                        {plate.plate}
                      </span>

                      <p className="text-xs text-muted-foreground">
                        {new Date(plate.timestamp).toLocaleTimeString([], {
                          hour: "2-digit",
                          minute: "2-digit",
                          second: "2-digit",
                        })}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    {/* Status Badge */}
                    <Badge
                      variant="outline"
                      className={cn(
                        "text-xs font-medium",
                        plate.was_allowed
                          ? "border-emerald-500/30 text-emerald-600 bg-emerald-500/10 dark:text-emerald-400"
                          : "border-destructive/30 text-destructive bg-destructive/10"
                      )}
                    >
                      {plate.was_allowed ? "Allowed" : "Denied"}
                    </Badge>

                    {/* Confidence Score */}
                    <Badge
                      variant={plate.confidence >= 0.9 ? "secondary" : "outline"}
                      className="font-mono text-xs text-muted-foreground"
                    >
                      {(plate.confidence * 100).toFixed(0)}%
                    </Badge>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-12 text-muted-foreground text-sm">
              No recent detections.
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}