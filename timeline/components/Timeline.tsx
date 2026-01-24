import {
  Alert,
  Box,
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
import { QueryErrorResetBoundary } from "@tanstack/react-query"
import { Suspense } from "react"
import { ErrorBoundary, type FallbackProps } from "react-error-boundary"
import { VList } from "virtua"
import type { Message } from "../../api/twittra.schemas.ts"
import { useReadManagement } from "../../app/hooks/useReadManagement.ts"
import { useMessageSubscription } from "../../socket/hooks/useMessageSubscription.ts"
import { useTimelineCache } from "../hooks/useTimelineCache.ts"
import { useTimelineInfinite } from "../hooks/useTimelineInfinite.ts"
import { MessageItem } from "./Message.tsx"

const TimelineContent = () => {
  const {
    messages,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useTimelineInfinite()
  const { updateMessage } = useTimelineCache()
  const handleMessageUpdated = (updatedMessage: Message) => {
    updateMessage(updatedMessage.id, (oldMessage) => ({
      ...updatedMessage,
      // Preserve user info from the old message as the socket update doesn't include it
      user: oldMessage.user,
    }))
  }

  // Subscribe to all loaded messages and handle updates
  const messageIds = messages.map((item) => item.id)
  useMessageSubscription(messageIds, handleMessageUpdated)
  const { markAsRead } = useReadManagement()

  return (
    <VList
      style={{ height: "100dvh", paddingTop: "var(--mantine-spacing-md)" }}
      onRangeChange={(_, end) => {
        // Load more when reaching boundaries
        if (end === messages.length - 1 && hasNextPage && !isFetchingNextPage) {
          fetchNextPage()
        }
      }}
    >
      {messages.map((item) => (
        <Box key={item.id} mb="md">
          <MessageItem message={item} onRead={markAsRead} />
        </Box>
      ))}
    </VList>
  )
}

const LoadingFallback = () => {
  return (
    <Stack
      pt="md"
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
        <ErrorBoundary FallbackComponent={ErrorFallback} onReset={reset}>
          <Suspense fallback={<LoadingFallback />}>
            <TimelineContent />
          </Suspense>
        </ErrorBoundary>
      )}
    </QueryErrorResetBoundary>
  )
}
