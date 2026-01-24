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
import {
  type InfiniteData,
  QueryErrorResetBoundary,
  useQueryClient,
} from "@tanstack/react-query"
import { Suspense, useCallback } from "react"
import { ErrorBoundary, type FallbackProps } from "react-error-boundary"
import { VList } from "virtua"
import {
  getGetTimelineQueryKey,
  type getTimelineResponseSuccess,
} from "../../api/timeline/timeline.ts"
import type { Message } from "../../api/twittra.schemas.ts"
import { useReadManagement } from "../../app/hooks/useReadManagement.ts"
import { useMessageSubscription } from "../../socket/hooks/useMessageSubscription.ts"
import { useTimelineInfinite } from "../hooks/useTimelineInfinite.ts"
import { MessageItem } from "./Message.tsx"

const TimelineContent = () => {
  const {
    messages,
    fetchNextPage,
    fetchPreviousPage,
    hasNextPage,
    hasPreviousPage,
  } = useTimelineInfinite()
  const queryClient = useQueryClient()

  // Optimize socket updates: update specific message in cache instead of refetching
  const handleMessageUpdated = useCallback(
    (updatedMessage: Message) => {
      queryClient.setQueryData<InfiniteData<getTimelineResponseSuccess>>(
        getGetTimelineQueryKey(),
        (oldData) => {
          if (!oldData) return oldData

          return {
            ...oldData,
            pages: oldData.pages.map((page) => ({
              ...page,
              data: page.data.map((item) =>
                item.id === updatedMessage.id ? updatedMessage : item
              ),
            })),
          }
        },
      )
    },
    [queryClient],
  )

  // Subscribe to all loaded messages and handle updates
  const messageIds = messages.map((item) => item.id)
  useMessageSubscription(messageIds, handleMessageUpdated)
  const { markAsRead } = useReadManagement()

  return (
    <VList
      style={{ height: "calc(100dvh - 2 * var(--mantine-spacing-md))" }}
      onRangeChange={(start, end) => {
        // Load more when reaching boundaries
        if (start === 0 && hasPreviousPage) {
          fetchPreviousPage()
        }
        if (end === messages.length - 1 && hasNextPage) {
          fetchNextPage()
        }
      }}
    >
      <Stack>
        {messages.map((item) => (
          <MessageItem key={item.id} message={item} onRead={markAsRead} />
        ))}
      </Stack>
    </VList>
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
