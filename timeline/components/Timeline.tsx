import { Group, Paper, Skeleton, Stack } from "@mantine/core"
import { Suspense } from "react"
import { useGetTimelineSuspense } from "../../api/timeline/timeline.ts"
import { MessageItem } from "./Message.tsx"

const TimelineContent = () => {
  const { data: { data } } = useGetTimelineSuspense()

  return (
    <Stack>
      {data.map((item) => <MessageItem key={item.id} message={item} />)}
    </Stack>
  )
}

const TimelineFallback = () => {
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

export const Timeline = () => {
  return (
    <Suspense fallback={<TimelineFallback />}>
      <TimelineContent />
    </Suspense>
  )
}
