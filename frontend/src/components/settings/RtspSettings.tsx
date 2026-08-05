import { useState, useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { Video, Loader2, AlertCircle, CheckCircle2 } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useGetRtspUrl, useUpdateRtspUrl, getGetRtspUrlQueryKey } from '@/lib/rust_api/schema';

export default function RtspSettings() {
  const queryClient = useQueryClient();
  const { data: rtspData, isLoading: isLoadingRtsp } = useGetRtspUrl();
  const updateRtspUrlMutation = useUpdateRtspUrl();
  
  const [rtspUrl, setRtspUrl] = useState<string | null>(null);
  const [rtspError, setRtspError] = useState<string | null>(null);
  const [rtspSuccess, setRtspSuccess] = useState<string | null>(null);

  useEffect(() => {
    if (rtspData?.rtsp_url !== undefined) {
      setRtspUrl(rtspData.rtsp_url);
    }
  }, [rtspData?.rtsp_url]);

  const handleSaveRtsp = async (e: React.FormEvent) => {
    e.preventDefault();
    setRtspError(null);
    setRtspSuccess(null);

    if (!rtspUrl?.trim()) {
      setRtspError("RTSP URL is required");
      return;
    }

    if (!rtspUrl.startsWith("rtsp://") && !rtspUrl.startsWith("rtsps://")) {
      setRtspError("URL must start with rtsp:// or rtsps://");
      return;
    }

    try {
      await updateRtspUrlMutation.mutateAsync({
        data: { rtsp_url: rtspUrl },
      });
      queryClient.invalidateQueries({ queryKey: getGetRtspUrlQueryKey() });
      setRtspSuccess("RTSP stream URL updated and verified successfully!");
    } catch (error: any) {
      setRtspError(error?.response?.data || error?.message || "Failed to validate RTSP connection.");
    }
  };

  return (
    <Card className="w-full min-w-full h-fit">
      <CardHeader>
        <CardTitle>RTSP Video Stream</CardTitle>
        <CardDescription>
          Configure the camera RTSP stream URL. The connection will be tested before saving.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSaveRtsp} className="space-y-4">
          {rtspError && (
            <div className="flex items-start gap-2.5 rounded-md border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
              <div className="space-y-1">
                <p className="font-semibold">Validation Failed</p>
                <p className="opacity-90">{rtspError}</p>
              </div>
            </div>
          )}
          {rtspSuccess && (
            <div className="flex items-start gap-2.5 rounded-md border border-emerald-500/50 bg-emerald-500/10 p-3 text-xs text-emerald-600 dark:text-emerald-400">
              <CheckCircle2 className="h-4 w-4 shrink-0 mt-0.5" />
              <div className="space-y-1">
                <p className="font-semibold">Connection Successful</p>
                <p className="opacity-90">{rtspSuccess}</p>
              </div>
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="rtspUrl">RTSP Stream URL</Label>
            <div className="relative">
              <Video className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                id="rtspUrl"
                type="text"
                placeholder="rtsp://192.168.1.100:554/stream"
                className="pl-9"
                value={rtspUrl || ''}
                disabled={isLoadingRtsp || updateRtspUrlMutation.isPending}
                onChange={(e) => {
                  setRtspError(null);
                  setRtspSuccess(null);
                  setRtspUrl(e.target.value);
                }}
                autoComplete="off"
              />
            </div>
          </div>
          <Button type="submit" className="w-full" disabled={isLoadingRtsp || updateRtspUrlMutation.isPending}>
            {updateRtspUrlMutation.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Testing RTSP Connection...
              </>
            ) : (
              "Save & Connect"
            )}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}