import { useState, useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { useGetFramerate, useUpdateFramerate, getGetFramerateQueryKey } from '@/lib/rust_api/schema';

const getNumericValue = (val: number | readonly number[]): number => {
  return typeof val === 'number' ? val : val[0];
};

export default function EngineSettings() {
  const queryClient = useQueryClient();
  const { data: fpsData, isLoading: isLoadingFps } = useGetFramerate();
  const updateFramerateMutation = useUpdateFramerate();
  const [framerate, setFramerate] = useState<number>(30);

  useEffect(() => {
    if (fpsData?.framerate !== undefined) {
      setFramerate(fpsData.framerate);
    }
  }, [fpsData?.framerate]);

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

  return (
    <Card className="w-full min-w-full h-fit">
      <CardHeader>
        <CardTitle>Processing Engine</CardTitle>
        <CardDescription>Configure computational resources.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
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
      </CardContent>
    </Card>
  );
}