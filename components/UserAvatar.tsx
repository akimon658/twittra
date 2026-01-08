import { Avatar } from "@mantine/core"

interface UserAvatarProps {
  username: string
}

export const UserAvatar = ({ username }: UserAvatarProps) => {
  return (
    <Avatar
      alt={`@${username}のアイコン`}
      src={`https://image-proxy.trap.jp/icon/${
        // image-proxy.trap.jp doesn't encode special characters, so we need to double encode them
        encodeURIComponent(encodeURIComponent(username))}?width=128`}
    />
  )
}
