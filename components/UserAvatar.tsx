import { Avatar, Skeleton } from "@mantine/core"
import { Suspense, use } from "react"

const getImagePromise = (src: string) => (
  new Promise<void>((resolve) => {
    const img = new Image()
    img.src = src
    img.onload = () => resolve()
    // Even if the image fails to load, we resolve the promise so that Mantine Avatar can show the fallback UI
    img.onerror = () => resolve()
  })
)

interface UserAvatarProps {
  username: string
}

const UserAvatarLoader = ({ username }: UserAvatarProps) => {
  // image-proxy.trap.jp doesn't encode special characters, so we need to double encode them
  const src = `https://image-proxy.trap.jp/icon/${
    encodeURIComponent(encodeURIComponent(username))
  }?width=128`

  use(getImagePromise(src))

  return (
    <Avatar
      alt={`@${username}のアイコン`}
      src={src}
    />
  )
}

export const UserAvatar = ({ username }: UserAvatarProps) => {
  return (
    <Suspense fallback={<Skeleton circle height={38} />}>
      <UserAvatarLoader username={username} />
    </Suspense>
  )
}
