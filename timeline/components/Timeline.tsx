import {
  Alert,
  Button,
  Center,
  Container,
  Group,
  Paper,
  Skeleton,
  Stack,
  Text,
} from "@mantine/core"
import { IconExclamationCircle, IconReload } from "@tabler/icons-react"
import { QueryErrorResetBoundary, useQueryClient } from "@tanstack/react-query"
import { Suspense } from "react"
import { ErrorBoundary, type FallbackProps } from "react-error-boundary"
import {
  getGetTimelineQueryKey,
  useGetTimelineSuspense,
} from "../../api/timeline/timeline.ts"
import { useMessageSubscription } from "../../hooks/useMessageSubscription.ts"
import { MessageItem } from "./Message.tsx"

const TimelineContent = () => {
  const { data: { data } } = useGetTimelineSuspense()
  const queryClient = useQueryClient()

  const handleMessageUpdated = () => {
    queryClient.invalidateQueries({ queryKey: getGetTimelineQueryKey() })
  }

  // Subscribe to all loaded messages and handle updates
  const messageIds = data.map((item) => item.id)
  useMessageSubscription(messageIds, handleMessageUpdated)

  return (
    <Stack>
      {data.map((item) => <MessageItem key={item.id} message={item} />)}
    </Stack>
  )
}

const LoadingFallback = () => {
  return (
    <Stack
      h="calc(100dvh - var(--mantine-spacing-md))"
      // Cancel out the padding of AppShell
      mb="-md"
      style={{ overflow: "hidden" }}
    >
      {Array.from({ length: 10 }).map((_, index) => (
        <Paper key={index}>
          <Group align="start" wrap="nowrap">
            <Skeleton circle height={38} />

            <Stack flex={1} gap="xs">
              <Skeleton height="1rem" width="10rem" />

              <Skeleton height="1rem" />

              <Skeleton height="1rem" />

              <Skeleton height="1rem" width="80%" />
            </Stack>
          </Group>
        </Paper>
      ))}
    </Stack>
  )
}

const ErrorFallback = ({ resetErrorBoundary }: FallbackProps) => {
  return (
    <Container>
      <Center>
        <Alert
          color="red"
          icon={<IconExclamationCircle />}
          title="エラー"
        >
          <Stack>
            <Text>
              タイムラインの読み込みに失敗しました。
            </Text>

            <Button
              leftSection={<IconReload size={20} />}
              onClick={resetErrorBoundary}
            >
              再読み込み
            </Button>
          </Stack>
        </Alert>
      </Center>
    </Container>
  )
}

export const Timeline = () => {
  return (
    <QueryErrorResetBoundary>
      {({ reset }) => (
        <ErrorBoundary fallbackRender={ErrorFallback} onReset={reset}>
          <Suspense fallback={<LoadingFallback />}>
            <TimelineContent />
          </Suspense>
        </ErrorBoundary>
      )}
    </QueryErrorResetBoundary>
  )
}
