import { MantineProvider } from "@mantine/core"
import "@mantine/core/styles.css"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { AuthProvider } from "../auth/components/AuthProvider.tsx"
import { useUser } from "../auth/hooks/useUser.ts"

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
})

export const App = () => {
  return (
    <MantineProvider>
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <Greet />
        </AuthProvider>
      </QueryClientProvider>
    </MantineProvider>
  )
}

const Greet = () => {
  const user = useUser()

  return <div>Hello, {user.handle}!</div>
}
