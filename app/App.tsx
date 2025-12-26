import { QueryClient, QueryClientProvider } from "@tanstack/react-query"

const queryClient = new QueryClient()

export const App = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <a href="/api/v1/auth/login">Login</a>
    </QueryClientProvider>
  )
}
