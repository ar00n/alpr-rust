import { useState, useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { 
  useGetFramerate, 
  useUpdateFramerate, 
  getGetFramerateQueryKey,
  useGetMinConfidence,
  useUpdateMinConfidence,
  getGetMinConfidenceQueryKey
} from '@/lib/rust_api/schema';

const getNumericValue = (val: number | readonly number[]): number => {
  return typeof val === 'number' ? val : val[0];
};

export default function EngineSettings() {
  const queryClient = useQueryClient();
  
  // Framerate State & Hooks
  const { data: fpsData, isLoading: isLoadingFps } = useGetFramerate();
  const updateFramerateMutation = useUpdateFramerate();
  const [framerate, setFramerate] = useState<number>(30);

  // Min Confidence State & Hooks
  const { data: confidenceData, isLoading: isLoadingConfidence } = useGetMinConfidence();
  const updateConfidenceMutation = useUpdateMinConfidence();
  const [minConfidence, setMinConfidence] = useState<number>(0.5);

  useEffect(() => {
    if (fpsData?.framerate !== undefined) {
      setFramerate(fpsData.framerate);
    }
  }, [fpsData?.framerate]);

  useEffect(() => {
    // Note: adjust 'min_confidence' if your generated API uses camelCase (e.g. 'minConfidence')
    if (confidenceData?.min_confidence !== undefined) {
      setMinConfidence(confidenceData.min_confidence);
    }
  }, [confidenceData?.min_confidence]);

  const handleFramerateCommit = (val: number | readonly number[]) => {
    const newFps = getNumericValue(val);
    updateFramerateMutation.mutate(
      { data: { framerate: newFps } },
      {
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: getGetFramerateQueryKey() });
        },
      }
    );
  };

  const handleConfidenceCommit = (val: number | readonly number[]) => {
    // Slider gives us 0-100, we convert it back to 0-1 for the API mutation
    const newConfidence = getNumericValue(val) / 100;
    updateConfidenceMutation.mutate(
      { data: { min_confidence: newConfidence } }, 
      {
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: getGetMinConfidenceQueryKey() });
        },
      }
    );
  };

  return (
    <Card className="w-full min-w-full h-fit">
      <CardHeader>
        <CardTitle>Processing Engine</CardTitle>
        <CardDescription>Configure computational resources.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-8">
        
        {/* Framerate Setting */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>Processing Framerate (FPS)</Label>
            <span className="font-mono bg-slate-100 px-2 py-1 rounded text-sm">
              {isLoadingFps ? '...' : `${framerate} FPS`}
            </span>
          </div>
          <Slider 
            value={[framerate]} 
            onValueChange={(val) => setFramerate(getNumericValue(val))}
            onValueCommitted={handleFramerateCommit}
            max={60} 
            min={1}
            step={1} 
            disabled={isLoadingFps || updateFramerateMutation.isPending}
            className="w-full" 
          />
          <p className="text-xs text-muted-foreground">
            Higher FPS increases detection accuracy but requires more computational power.
          </p>
        </div>

        {/* Min Confidence Setting */}
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <Label>Minimum Confidence</Label>
            <span className="font-mono bg-slate-100 px-2 py-1 rounded text-sm">
              {isLoadingConfidence ? '...' : `${Math.round(minConfidence * 100)}%`}
            </span>
          </div>
          <Slider 
            value={[minConfidence * 100]} 
            onValueChange={(val) => setMinConfidence(getNumericValue(val) / 100)}
            onValueCommitted={handleConfidenceCommit}
            max={100} 
            min={0}
            step={1} 
            disabled={isLoadingConfidence || updateConfidenceMutation.isPending}
            className="w-full" 
          />
          <p className="text-xs text-muted-foreground">
            Higher confidence reduces false positives but might miss some harder-to-spot detections.
          </p>
        </div>

      </CardContent>
    </Card>
  );
}