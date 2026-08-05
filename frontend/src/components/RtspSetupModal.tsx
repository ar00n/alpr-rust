import { useState } from "react"
import { useForm } from "@tanstack/react-form"
import { Video, Loader2, AlertCircle, CheckCircle2 } from "lucide-react"

import { useAuthStore } from "@/src/store/useAuthStore"
import { Dialog, DialogContent } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "@/components/ui/card"
import { useGetRtspUrl, useUpdateRtspUrl } from "@/lib/rust_api/schema"

export function RtspSetupModal() {
  const { isAuthenticated, isAdmin } = useAuthStore()
  const [isDismissed, setIsDismissed] = useState(false)
  const [serverError, setServerError] = useState<string | null>(null)
  const [successMessage, setSuccessMessage] = useState<string | null>(null)

  // Skip querying if user is not authenticated or not an admin
  const { data: rtspUrl, isPending, isError } = useGetRtspUrl({
    query: {
      enabled: isAuthenticated && isAdmin,
    },
  })

  const updateRtspUrlMutation = useUpdateRtspUrl()

  const form = useForm({
    defaultValues: {
      rtspUrl: "",
    },
    onSubmit: async ({ value }) => {
      setServerError(null)
      setSuccessMessage(null)
      try {
        await updateRtspUrlMutation.mutateAsync({
          data: { rtsp_url: value.rtspUrl },
        })
        setSuccessMessage("RTSP stream URL updated and verified successfully!")

        // Automatically dismiss the modal after 2 seconds
        setTimeout(() => {
          setIsDismissed(true)
        }, 2000)
      } catch (error: any) {
        // Extract error message from Rust backend HTTP 400 response
        const errorMessage =
          error?.response?.data ||
          error?.message ||
          "Failed to validate RTSP connection. Please check the URL and try again."

        setServerError(errorMessage)
      }
    },
  })

  // Keep open while setting RTSP URL or while showing the success message
  const shouldBeOpen = Boolean(
    isAuthenticated && isAdmin && !isPending && !isError && (!rtspUrl?.rtsp_url || successMessage)
  )
  const isOpen = shouldBeOpen && !isDismissed

  return (
    <Dialog
      open={isOpen}
      onOpenChange={(open) => {
        if (!open) {
          setIsDismissed(true)
        }
      }}
    >
      <DialogContent className="p-0 sm:max-w-[425px]">
        <Card className="border-0 shadow-none">
          <CardHeader>
            <CardTitle>Set RTSP Stream URL</CardTitle>
            <CardDescription>
              Please provide a valid RTSP URL for the video stream. We will test
              the connection before saving.
            </CardDescription>
          </CardHeader>

          <form
            onSubmit={(e) => {
              e.preventDefault()
              e.stopPropagation()
              form.handleSubmit()
            }}
          >
            <CardContent className="space-y-4">
              {/* Server Error Banner */}
              {serverError && (
                <div className="flex items-start gap-2.5 rounded-md border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive">
                  <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
                  <div className="space-y-1">
                    <p className="font-semibold">Validation Failed</p>
                    <p className="opacity-90">{serverError}</p>
                  </div>
                </div>
              )}

              {/* Success Banner */}
              {successMessage && (
                <div className="flex items-start gap-2.5 rounded-md border border-emerald-500/50 bg-emerald-500/10 p-3 text-xs text-emerald-600 dark:text-emerald-400">
                  <CheckCircle2 className="h-4 w-4 shrink-0 mt-0.5" />
                  <div className="space-y-1">
                    <p className="font-semibold">Connection Successful</p>
                    <p className="opacity-90">{successMessage}</p>
                  </div>
                </div>
              )}

              <form.Field
                name="rtspUrl"
                validators={{
                  onChange: ({ value }) => {
                    if (!value) return "RTSP URL is required"
                    if (
                      !value.startsWith("rtsp://") &&
                      !value.startsWith("rtsps://")
                    ) {
                      return "URL must start with rtsp:// or rtsps://"
                    }
                    return undefined
                  },
                }}
                children={(field) => {
                  const isInvalid =
                    field.state.meta.isTouched && !field.state.meta.isValid
                  return (
                    <div className="space-y-2">
                      <Label htmlFor={field.name}>RTSP Stream URL</Label>
                      <div className="relative">
                        <Video className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                          id={field.name}
                          name={field.name}
                          type="text"
                          placeholder="rtsp://192.168.1.100:554/stream"
                          className="pl-9"
                          value={field.state.value}
                          disabled={updateRtspUrlMutation.isPending}
                          onBlur={field.handleBlur}
                          onChange={(e) => {
                            if (serverError) setServerError(null)
                            if (successMessage) setSuccessMessage(null)
                            field.handleChange(e.target.value)
                          }}
                          aria-invalid={isInvalid}
                          autoComplete="off"
                        />
                      </div>
                      {isInvalid && (
                        <p className="text-xs text-destructive">
                          {field.state.meta.errors.join(", ")}
                        </p>
                      )}
                    </div>
                  )
                }}
              />
            </CardContent>

            <CardFooter>
              <Button
                type="submit"
                className="w-full"
                disabled={updateRtspUrlMutation.isPending}
              >
                {updateRtspUrlMutation.isPending ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Testing RTSP Connection...
                  </>
                ) : (
                  "Save & Connect"
                )}
              </Button>
            </CardFooter>
          </form>
        </Card>
      </DialogContent>
    </Dialog>
  )
}