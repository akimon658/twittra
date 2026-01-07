import { Stack } from "@mantine/core"
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

export const Timeline = () => {
  return (
    <Suspense fallback={<div>Loading timeline...</div>}>
      <TimelineContent />
    </Suspense>
  )
}
