import { Group, Paper, Stack, Typography } from "@mantine/core"
import { type Store, traQMarkdownIt } from "@traptitech/traq-markdown-it"
import type { MessageListItem } from "../../api/twittra.schemas.ts"
import { MessageAuthorAvater } from "./MessageAuthorAvater.tsx"
import { MessageHeader } from "./MessageHeader.tsx"

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
  message: MessageListItem
}

export const MessageItem = ({ message }: MessageProps) => {
  return (
    <Paper>
      <Group align="start" wrap="nowrap">
        <MessageAuthorAvater user={message.user} userId={message.userId} />

        <Stack gap="xs">
          <MessageHeader user={message.user} userId={message.userId} />

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
