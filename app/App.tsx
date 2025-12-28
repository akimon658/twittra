import { createTheme, MantineProvider } from "@mantine/core"
import "@mantine/core/styles.css"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { AuthProvider } from "../auth/components/AuthProvider.tsx"
import { useUser } from "../auth/hooks/useUser.ts"
import "./global.css"

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
    },
  },
})
const theme = createTheme({
  fontFamily: "Google Sans Flex, Noto Sans JP, sans-serif",
})

export const App = () => {
  return (
    <MantineProvider defaultColorScheme="auto" theme={theme}>
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
