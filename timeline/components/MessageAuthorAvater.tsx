import { Skeleton } from "@mantine/core"
import { Suspense } from "react"
import type { User } from "../../api/twittra.schemas.ts"
import { useGetUserByIdSuspense } from "../../api/user/user.ts"
import { UserAvatar } from "../../components/UserAvatar.tsx"

interface MessageAuthorAvaterLoaderProps {
  userId: string
}

const MessageAuthorAvaterLoader = (
  { userId }: MessageAuthorAvaterLoaderProps,
) => {
  const { data: { data } } = useGetUserByIdSuspense(userId)

  return <UserAvatar username={data.handle} />
}

interface MessageAuthorAvaterProps {
  user?: User
  userId: string
}

export const MessageAuthorAvater = (
  { user, userId }: MessageAuthorAvaterProps,
) => {
  if (user) {
    return <UserAvatar username={user.handle} />
  }

  return (
    <Suspense fallback={<Skeleton circle height={40} />}>
      <MessageAuthorAvaterLoader userId={userId} />
    </Suspense>
  )
}
