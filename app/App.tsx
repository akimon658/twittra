import {
  Alert,
  Center,
  Code,
  Container,
  createTheme,
  MantineProvider,
} from "@mantine/core"
import "@mantine/core/styles.css"
import { IconExclamationCircle } from "@tabler/icons-react"
import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { ErrorBoundary, type FallbackProps } from "react-error-boundary"
import { AuthProvider } from "../auth/components/AuthProvider.tsx"
import { Layout } from "../components/Layout.tsx"
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

const GlobalErrorFallback = ({ error }: FallbackProps) => {
  return (
    <Container>
      <Center h="100dvh">
        <Alert
          color="red"
          icon={<IconExclamationCircle />}
          title="予期しないエラーが発生しました"
        >
          <Code block>
            {error instanceof Error
              ? (error.stack ?? error.message)
              : String(error)}
          </Code>
        </Alert>
      </Center>
    </Container>
  )
}

export const App = () => {
  return (
    <MantineProvider defaultColorScheme="auto" theme={theme}>
      <ErrorBoundary FallbackComponent={GlobalErrorFallback}>
        <QueryClientProvider client={queryClient}>
          <AuthProvider>
            <Layout />
          </AuthProvider>
        </QueryClientProvider>
      </ErrorBoundary>
    </MantineProvider>
  )
}
