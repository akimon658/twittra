import { MantineProvider } from "@mantine/core"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import type { Router } from "@tanstack/react-router"
import { RouterProvider } from "@tanstack/react-router"
import { render } from "@testing-library/react"
import { http, HttpResponse } from "msw"
import type { ReactElement, ReactNode } from "react"
import type { User } from "../api/twittra.schemas.ts"
import { UserContext } from "../auth/context/user.ts"
import type { AppSocket } from "../socket/lib/types.ts"
import { createMockUser } from "./factories.ts"
import { MockSocketProvider } from "./MockSocketProvider.tsx"
import { worker } from "./setup.ts"

// Shared test query client configuration
export function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  })
}

// All providers wrapper component
export function AllProviders({
  children,
  queryClient,
  user,
  socket,
}: {
  children: ReactNode
  queryClient: QueryClient
  user: User
  socket?: AppSocket
}) {
  return (
    <QueryClientProvider client={queryClient}>
      <MockSocketProvider socket={socket}>
        <UserContext value={user}>
          <MantineProvider>{children}</MantineProvider>
        </UserContext>
      </MockSocketProvider>
    </QueryClientProvider>
  )
}

// Custom render with all providers
export function renderWithProviders(
  ui: ReactElement,
  {
    user = createMockUser(),
    socket,
    router,
  }: { user?: User; socket?: AppSocket; router?: Router } = {},
) {
  const queryClient = createTestQueryClient()

  const content = router ? <RouterProvider router={router} /> : ui

  return render(
    <AllProviders queryClient={queryClient} user={user} socket={socket}>
      {content}
    </AllProviders>,
  )
}

// Wrapper for use with renderHook
export function createTestWrapper({
  user = createMockUser(),
  socket,
}: { user?: User; socket?: AppSocket } = {}) {
  const queryClient = createTestQueryClient()

  return ({ children }: { children: ReactNode }) => (
    <AllProviders queryClient={queryClient} user={user} socket={socket}>
      {children}
    </AllProviders>
  )
}

// Helper to simulate API errors
export function mockApiError(endpoint: string, status = 500) {
  worker.use(
    http.get(endpoint, () => {
      return new HttpResponse(null, { status })
    }),
  )
}

// Export commonly used testing utilities
export { screen, waitFor, within } from "@testing-library/react"
export { userEvent } from "@testing-library/user-event"
