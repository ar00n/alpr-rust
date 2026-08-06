import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { 
  Zap, Trash2, AlertCircle, CheckCircle2, Play, Plus, Loader2
} from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

import { 
  useGetCustomActions,
  getGetCustomActionsQueryKey,
  useAddCustomAction,
  useTestCustomAction,
  useDeleteCustomAction
} from '@/lib/rust_api/schema';
import { getErrorMessage } from '../main';

export default function CustomActions() {
  const queryClient = useQueryClient();

  const { data: actions, isLoading: isLoadingActions } = useGetCustomActions();
  const addActionMutation = useAddCustomAction();
  const testActionMutation = useTestCustomAction();
  const deleteActionMutation = useDeleteCustomAction();

  const [name, setName] = useState('');
  const [url, setUrl] = useState('');
  const [method, setMethod] = useState('POST');
  const [authType, setAuthType] = useState('NONE');
  const [authToken, setAuthToken] = useState('');
  const [authUsername, setAuthUsername] = useState('');
  const [authPassword, setAuthPassword] = useState('');
  
  const [apiKeyName, setApiKeyName] = useState('X-API-Key');
  const [apiKeyValue, setApiKeyValue] = useState('');
  const [apiKeyPlacement, setApiKeyPlacement] = useState<'header' | 'query'>('header');

  const [headersStr, setHeadersStr] = useState('{\n  "Content-Type": "application/json"\n}');
  const [bodyTemplate, setBodyTemplate] = useState('{\n  "plate": "${LICENCE_PLATE}"\n}');

  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<{ status: number; body: string } | null>(null);

  const buildPayload = () => {
    let headersObj = null;
    if (headersStr.trim()) {
      try {
        headersObj = JSON.parse(headersStr);
      } catch (e) {
        throw new Error("Headers must be valid JSON.");
      }
    }

    let authData = null;
    if (authType === 'BEARER') {
      authData = { token: authToken };
    } else if (authType === 'BASIC') {
      authData = { username: authUsername, password: authPassword };
    } else if (authType === 'API_KEY') {
      authData = { 
        key: apiKeyValue, 
        header_name: apiKeyName.trim() || 'X-API-Key', 
        placement: apiKeyPlacement 
      };
    }

    return {
      name: name.trim() || 'Unnamed Action',
      url,
      method,
      auth_type: authType,
      auth_data: authData,
      headers: headersObj,
      body_template: bodyTemplate.trim() || null,
    };
  };

  const handleTest = async () => {
    setError(null);
    setSuccess(null);
    setTestResult(null);

    try {
      if (!url) throw new Error("URL is required");
      const payload = buildPayload();
      
      const res = await testActionMutation.mutateAsync({ data: payload });
      setTestResult({
        status: res.status,
        body: res.body
      });
      setSuccess("Test executed successfully. Check the response below.");
    } catch (err: any) {
      setError(getErrorMessage(err) || "Failed to test action.");
    }
  };

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setTestResult(null);

    try {
      if (!url || !name) throw new Error("Name and URL are required");
      const payload = buildPayload();

      await addActionMutation.mutateAsync({ data: payload });
      
      queryClient.invalidateQueries({ queryKey: getGetCustomActionsQueryKey() });
      setSuccess("Custom action saved successfully!");
      
      setName('');
      setUrl('');
      setAuthType('NONE');
      setAuthToken('');
      setAuthUsername('');
      setAuthPassword('');
      setApiKeyValue('');
      setApiKeyName('X-API-Key');
      setApiKeyPlacement('header');
    } catch (err: any) {
      setError(err?.message || err?.response?.data || "Failed to save action.");
    }
  };

  const handleDelete = async (id: number) => {
    if (!confirm("Are you sure you want to delete this action?")) return;
    try {
      // @ts-ignore
      await deleteActionMutation.mutateAsync({ id });
      queryClient.invalidateQueries({ queryKey: getGetCustomActionsQueryKey() });
    } catch (err: any) {
      alert("Failed to delete action");
    }
  };

  return (
    <div className="space-y-6">
      {/* Creation / Test Form */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Zap className="w-5 h-5 text-primary" />
            Create Webhook / Action
          </CardTitle>
          <CardDescription>Configure and test HTTP requests triggered by ANPR events.</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="space-y-4" onSubmit={handleSave}>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Action Name</Label>
                <Input value={name} onChange={e => setName(e.target.value)} placeholder="e.g. Open Gate 1" required />
              </div>
              <div className="space-y-2">
                <Label>HTTP Method</Label>
                <select 
                  value={method} 
                  onChange={e => setMethod(e.target.value)}
                  className="flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                >
                  <option value="GET">GET</option>
                  <option value="POST">POST</option>
                  <option value="PUT">PUT</option>
                  <option value="DELETE">DELETE</option>
                </select>
              </div>
            </div>

            <div className="space-y-2">
              <Label>Target URL</Label>
              <Input value={url} onChange={e => setUrl(e.target.value)} placeholder="https://api.example.com/webhook" required type="url" />
            </div>

            {/* Auth Section */}
            <div className="space-y-4 p-4 border rounded-lg bg-muted/20">
              <div className="space-y-2">
                <Label>Authentication Type</Label>
                <select 
                  value={authType} 
                  onChange={e => setAuthType(e.target.value)}
                  className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                >
                  <option value="NONE">None</option>
                  <option value="BEARER">Bearer Token</option>
                  <option value="BASIC">Basic Auth</option>
                  <option value="API_KEY">API Key</option>
                </select>
              </div>

              {authType === 'BEARER' && (
                <div className="space-y-2">
                  <Label>Token</Label>
                  <Input value={authToken} onChange={e => setAuthToken(e.target.value)} placeholder="eyJh..." />
                </div>
              )}

              {authType === 'BASIC' && (
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label>Username</Label>
                    <Input value={authUsername} onChange={e => setAuthUsername(e.target.value)} />
                  </div>
                  <div className="space-y-2">
                    <Label>Password</Label>
                    <Input type="password" value={authPassword} onChange={e => setAuthPassword(e.target.value)} />
                  </div>
                </div>
              )}

              {authType === 'API_KEY' && (
                <div className="space-y-3">
                  <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                    <div className="space-y-2">
                      <Label>Placement</Label>
                      <select
                        value={apiKeyPlacement}
                        onChange={e => setApiKeyPlacement(e.target.value as 'header' | 'query')}
                        className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                      >
                        <option value="header">Header</option>
                        <option value="query">URL Query Parameter</option>
                      </select>
                    </div>
                    <div className="space-y-2 sm:col-span-2">
                      <Label>{apiKeyPlacement === 'header' ? 'Header Name' : 'Query Parameter Name'}</Label>
                      <Input 
                        value={apiKeyName} 
                        onChange={e => setApiKeyName(e.target.value)} 
                        placeholder={apiKeyPlacement === 'header' ? 'X-API-Key' : 'api_key'} 
                      />
                    </div>
                  </div>
                  <div className="space-y-2">
                    <Label>Key Value</Label>
                    <Input 
                      type="password" 
                      value={apiKeyValue} 
                      onChange={e => setApiKeyValue(e.target.value)} 
                      placeholder="e.g. secret_123456789" 
                    />
                  </div>
                </div>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>Headers (JSON)</Label>
                <textarea
                  value={headersStr}
                  onChange={e => setHeadersStr(e.target.value)}
                  className="flex min-h-[120px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
                  placeholder='{"Content-Type": "application/json"}'
                />
              </div>
              <div className="space-y-2">
                <Label>Body Template</Label>
                <textarea
                  value={bodyTemplate}
                  onChange={e => setBodyTemplate(e.target.value)}
                  className="flex min-h-[120px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono"
                  placeholder='{"plate": "${LICENCE_PLATE}"}'
                />
                <p className="text-xs text-muted-foreground">
                  Use <code>{`\${LICENCE_PLATE}`}</code> to inject the detected plate.
                </p>
              </div>
            </div>

            {/* Status Messages */}
            {error && (
              <div className="flex items-center gap-2 text-sm text-destructive bg-destructive/10 p-3 rounded-md">
                <AlertCircle className="w-4 h-4 shrink-0" />
                <p>{error}</p>
              </div>
            )}
            {success && (
              <div className="flex items-center gap-2 text-sm text-green-600 bg-green-50 p-3 rounded-md">
                <CheckCircle2 className="w-4 h-4 shrink-0" />
                <p>{success}</p>
              </div>
            )}
            {testResult && (
              <div className="mt-4 p-4 border rounded-md bg-muted/30">
                <div className="flex items-center gap-2 mb-2">
                  <Badge variant={testResult.status < 300 ? "default" : "destructive"}>
                    HTTP {testResult.status}
                  </Badge>
                  <span className="text-sm font-medium">Response Body:</span>
                </div>
                <pre className="text-xs p-2 bg-background border rounded overflow-auto max-h-40">
                  {testResult.body || "(Empty Response)"}
                </pre>
              </div>
            )}

            {/* Actions */}
            <div className="flex items-center gap-3 pt-4 border-t">
              <Button type="button" variant="secondary" onClick={handleTest} disabled={testActionMutation.isPending}>
                {testActionMutation.isPending ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : <Play className="w-4 h-4 mr-2" />}
                Test Action
              </Button>
              <Button type="submit" disabled={addActionMutation.isPending}>
                {addActionMutation.isPending ? <Loader2 className="w-4 h-4 mr-2 animate-spin" /> : <Plus className="w-4 h-4 mr-2" />}
                Save Action
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      {/* Existing Actions List */}
      <Card>
        <CardHeader>
          <CardTitle>Saved Actions</CardTitle>
          <CardDescription>Manage active webhooks and hooks.</CardDescription>
        </CardHeader>
        <CardContent>
          {isLoadingActions ? (
            <div className="flex items-center justify-center p-6 text-muted-foreground">
              <Loader2 className="w-6 h-6 animate-spin mr-2" />
              Loading actions...
            </div>
          ) : !actions || actions.length === 0 ? (
            <div className="text-center p-6 text-sm text-muted-foreground border rounded-lg border-dashed">
              No custom actions configured yet.
            </div>
          ) : (
            <div className="space-y-3">
              {actions.map((action: any) => (
                <div key={action.id} className="flex flex-col sm:flex-row sm:items-center justify-between p-4 border rounded-lg bg-card gap-4">
                  <div className="space-y-1 overflow-hidden">
                    <h4 className="font-semibold text-sm flex items-center gap-2">
                      {action.name}
                      <span className="text-xs px-2 py-0.5 rounded bg-primary/10 text-primary font-mono font-normal">
                        {action.method}
                      </span>
                    </h4>
                    <p className="text-xs text-muted-foreground truncate" title={action.url}>
                      {action.url}
                    </p>
                  </div>
                  <Button 
                    variant="destructive" 
                    size="sm"
                    onClick={() => handleDelete(action.id)}
                    disabled={deleteActionMutation.isPending}
                    className="shrink-0"
                  >
                    <Trash2 className="w-4 h-4 sm:mr-2" />
                    <span className="hidden sm:inline">Delete</span>
                  </Button>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function Badge({ children, variant, className }: any) {
  const base = "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2";
  const variants: any = {
    default: "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
    destructive: "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
  };
  return (
    <div className={`${base} ${variants[variant]} ${className || ''}`}>
      {children}
    </div>
  );
}