import { MantineProvider } from "@mantine/core"
import "@mantine/core/styles.css"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { Suspense } from "react"
import { useGetMeSuspense } from "../api/user/user.ts"

const queryClient = new QueryClient()

export const App = () => {
  return (
    <MantineProvider>
      <QueryClientProvider client={queryClient}>
        <a href="/api/v1/auth/login">Login</a>

        <Suspense fallback={<div>Loading...</div>}>
          <Greet />
        </Suspense>
      </QueryClientProvider>
    </MantineProvider>
  )
}

const Greet = () => {
  const { data } = useGetMeSuspense()

  return <div>Hello, {data.handle}!</div>
}
