import { useState } from "react"
import { useForm } from "@tanstack/react-form"
import { useQueryClient } from "@tanstack/react-query"
import { Lock, Mail, Loader2, AlertCircle } from "lucide-react"

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
import { loginUser } from "@/lib/rust_api/schema"

import { AxiosError } from "axios"

export function LoginModal() {
  const { isAuthenticated, setCredentials } = useAuthStore()
  const queryClient = useQueryClient()
  const [isPending, setIsPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const form = useForm({
    defaultValues: {
      username: "",
      password: "",
    },
    onSubmit: async ({ value }) => {
      setIsPending(true)
      setError(null)
      try {
        const res = await loginUser(value)
        setCredentials(res.token, value.username)
        queryClient.invalidateQueries()
      } catch (err) {
        console.error("Login failed:", err)
        
        if (err instanceof AxiosError) {
          if (err.response?.status === 401) {
            setError("Invalid username or password. Please try again.")
          } else {
            setError(
              `Login failed with status ${err.response?.status}. Please try again later.`
            )
          }
        }
      } finally {
        setIsPending(false)
      }
    },
  })

  return (
    // Opens automatically when NOT authenticated
    <Dialog 
      open={!isAuthenticated} 
      disablePointerDismissal
      onOpenChange={(_, eventDetails) => {
        if (
          eventDetails.reason === "escape-key" ||
          eventDetails.reason === "outside-press"
        ) {
          eventDetails.cancel(); // Prevents Base UI from closing the dialog
        }
      }}
    >
      <DialogContent className="p-0 sm:max-w-[425px]">
        <Card className="border-0 shadow-none">
          <CardHeader>
            <CardTitle>Sign In</CardTitle>
            <CardDescription>
              Enter your account credentials to log in.
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
              {/* Login Error Alert */}
              {error && (
                <div 
                  role="alert" 
                  className="flex items-start gap-3 rounded-md bg-destructive/15 border border-destructive/20 p-3 text-sm text-destructive"
                >
                  <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
                  <span className="flex-1 font-medium">{error}</span>
                </div>
              )}

              <form.Field
                name="username"
                children={(field) => {
                  const isInvalid =
                    field.state.meta.isTouched && !field.state.meta.isValid
                  return (
                    <div className="space-y-2">
                      <Label htmlFor={field.name}>Username</Label>
                      <div className="relative">
                        <Mail className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                          id={field.name}
                          name={field.name}
                          type="text"
                          placeholder="username"
                          className="pl-9"
                          value={field.state.value}
                          onBlur={field.handleBlur}
                          onChange={(e) => {
                            if (error) setError(null)
                            field.handleChange(e.target.value)
                          }}
                          aria-invalid={isInvalid}
                          autoComplete="username"
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

              <form.Field
                name="password"
                children={(field) => {
                  const isInvalid =
                    field.state.meta.isTouched && !field.state.meta.isValid
                  return (
                    <div className="space-y-2">
                      <Label htmlFor={field.name}>Password</Label>
                      <div className="relative">
                        <Lock className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                        <Input
                          id={field.name}
                          name={field.name}
                          type="password"
                          placeholder="••••••••"
                          className="pl-9"
                          value={field.state.value}
                          onBlur={field.handleBlur}
                          onChange={(e) => {
                            if (error) setError(null)
                            field.handleChange(e.target.value)
                          }}
                          aria-invalid={isInvalid}
                          autoComplete="current-password"
                        />
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Enter your account password.
                      </p>
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
              <Button type="submit" className="w-full" disabled={isPending}>
                {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Sign In
              </Button>
            </CardFooter>
          </form>
        </Card>
      </DialogContent>
    </Dialog>
  )
}