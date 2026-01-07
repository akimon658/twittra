import { Avatar } from "@mantine/core"

interface UserAvatarProps {
  username: string
}

export const UserAvatar = ({ username }: UserAvatarProps) => {
  return (
    <Avatar
      alt={`@${username}のアイコン`}
      src={`https://image-proxy.trap.jp/icon/${username}?width=128`}
    />
  )
}
