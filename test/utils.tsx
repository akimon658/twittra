import { MantineProvider } from "@mantine/core"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { render } from "@testing-library/react"
import { http, HttpResponse } from "msw"
import type { ReactElement } from "react"
import type { User } from "../api/twittra.schemas.ts"
import { MockSocketProvider } from "../app/MockSocketProvider.tsx"
import { UserContext } from "../auth/context/user.ts"
import { createMockUser } from "./factories.ts"
import { worker } from "./setup.ts"

// Custom render with all providers
export function renderWithProviders(
  ui: ReactElement,
  { user = createMockUser(), socket }: { user?: User; socket?: any } = {},
) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MockSocketProvider socket={socket}>
        <UserContext value={user}>
          <MantineProvider>
            {ui}
          </MantineProvider>
        </UserContext>
      </MockSocketProvider>
    </QueryClientProvider>,
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
