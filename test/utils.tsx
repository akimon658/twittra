import { render } from "@testing-library/react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { MantineProvider } from "@mantine/core"
import type { ReactElement } from "react"
import { worker } from "./setup"
import { http, HttpResponse } from "msw"

// Custom render with all providers
export function renderWithProviders(ui: ReactElement) {
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
            <MantineProvider>
                {ui}
            </MantineProvider>
        </QueryClientProvider>,
    )
}

// Helper to override MSW responses per test
export function mockApiResponse(endpoint: string, response: unknown) {
    worker.use(
        http.get(endpoint, () => {
            return HttpResponse.json(response)
        }),
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
