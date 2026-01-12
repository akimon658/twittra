import { Avatar, Skeleton } from "@mantine/core"
import { useState } from "react"

interface UserAvatarProps {
  userId: string
  username: string
}

export const UserAvatar = ({ userId, username }: UserAvatarProps) => {
  const [isLoading, setIsLoading] = useState(true)
  const src = `/api/v1/users/${userId}/icon`

  return (
    <>
      {isLoading && <Skeleton circle height={38} />}
      <Avatar
        alt={`@${username}のアイコン`}
        imageProps={{
          onLoad: () => setIsLoading(false),
          onError: () => setIsLoading(false),
        }}
        src={src}
        style={{ display: isLoading ? "none" : undefined }}
      />
    </>
  )
}
