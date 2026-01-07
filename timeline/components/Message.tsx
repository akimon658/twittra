import { Group, Paper, Stack, Text, Typography } from "@mantine/core"
import { type Store, traQMarkdownIt } from "@traptitech/traq-markdown-it"
import type { Message } from "../../api/twittra.schemas.ts"
import { useGetUserByIdSuspense } from "../../api/user/user.ts"
import { UserAvatar } from "../../components/UserAvatar.tsx"

const store: Store = {
  generateChannelHref: () => "",
  generateUserGroupHref: () => "",
  generateUserHref: () => "",
  getChannel: (id) => ({ id }),
  getMe: () => undefined,
  getStampByName: () => undefined,
  getUser: (id) => ({ id }),
  getUserByName: () => undefined,
  getUserGroup: () => undefined,
}
const md = new traQMarkdownIt(store, undefined, "")

interface MessageProps {
  message: Message
}

export const MessageItem = ({ message }: MessageProps) => {
  const { data: { data } } = useGetUserByIdSuspense(message.userId)

  return (
    <Paper>
      <Group align="start" wrap="nowrap">
        <UserAvatar username={data.handle} />

        <Stack gap={0}>
          <Group gap="xs">
            <Text fw={500} span>{data.displayName}</Text>

            <Text c="dimmed" span>@{data.handle}</Text>
          </Group>

          <Typography>
            <div
              // deno-lint-ignore react-no-danger
              dangerouslySetInnerHTML={{
                __html: md.render(message.content).renderedText,
              }}
            />
          </Typography>
        </Stack>
      </Group>
    </Paper>
  )
}
