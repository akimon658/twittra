import { Skeleton } from "@mantine/core"
import { Suspense } from "react"
import type { User } from "../../api/twittra.schemas.ts"
import { useGetUserByIdSuspense } from "../../api/user/user.ts"
import { UserAvatar } from "../../components/UserAvatar.tsx"

interface MessageAuthorAvatarLoaderProps {
  userId: string
}

const MessageAuthorAvatarLoader = (
  { userId }: MessageAuthorAvatarLoaderProps,
) => {
  const { data: { data } } = useGetUserByIdSuspense(userId)

  return <UserAvatar username={data.handle} />
}

interface MessageAuthorAvatarProps {
  user?: User
  userId: string
}

export const MessageAuthorAvatar = (
  { user, userId }: MessageAuthorAvatarProps,
) => {
  if (user) {
    return <UserAvatar username={user.handle} />
  }

  return (
    <Suspense fallback={<Skeleton circle height={38} />}>
      <MessageAuthorAvatarLoader userId={userId} />
    </Suspense>
  )
}
