import { Avatar, Skeleton } from "@mantine/core"
import { useState } from "react"

interface UserAvatarProps {
  username: string
}

export const UserAvatar = ({ username }: UserAvatarProps) => {
  const [isLoading, setIsLoading] = useState(true)
  // image-proxy.trap.jp doesn't encode special characters, so we need to double encode them
  const src = `https://image-proxy.trap.jp/icon/${
    encodeURIComponent(encodeURIComponent(username))
  }?width=128`

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
