import { Group, Skeleton, Text } from "@mantine/core"
import { Suspense } from "react"
import type { User } from "../../api/twittra.schemas.ts"
import { useGetUserByIdSuspense } from "../../api/user/user.ts"

interface MessageHeaderLoaderProps {
  userId: string
}

const MessageHeaderLoader = ({ userId }: MessageHeaderLoaderProps) => {
  const { data: { data } } = useGetUserByIdSuspense(userId)

  return (
    <Group gap="xs">
      <Text fw={500} span>{data.displayName}</Text>

      <Text c="dimmed" span>@{data.handle}</Text>
    </Group>
  )
}

interface MessageHeaderProps {
  user?: User
  userId: string
}

export const MessageHeader = ({ user, userId }: MessageHeaderProps) => {
  if (user) {
    return (
      <Group gap="xs">
        <Text fw={500} span>{user.displayName}</Text>

        <Text c="dimmed" span>@{user.handle}</Text>
      </Group>
    )
  }

  return (
    <Suspense fallback={<Skeleton height={16} />}>
      <MessageHeaderLoader userId={userId} />
    </Suspense>
  )
}
