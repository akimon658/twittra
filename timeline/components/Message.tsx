import { Group, Paper, Spoiler, Stack, Typography } from "@mantine/core"
import { type Store, traQMarkdownIt } from "@traptitech/traq-markdown-it"
import type { MessageListItem } from "../../api/twittra.schemas.ts"
import { MessageAuthorAvatar } from "./MessageAuthorAvatar.tsx"
import { MessageFooter } from "./MessageFooter.tsx"
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
        <MessageAuthorAvatar user={message.user} userId={message.userId} />

        <Stack gap="xs" miw={0}>
          <MessageHeader user={message.user} userId={message.userId} />

          <Spoiler hideLabel="閉じる" showLabel="続きを見る">
            <Typography>
              <article
                // deno-lint-ignore react-no-danger -- Rendered content is safe unless href generators are unsafe
                dangerouslySetInnerHTML={{
                  __html: md.render(message.content).renderedText,
                }}
              />
            </Typography>
          </Spoiler>

          <MessageFooter reactions={message.reactions} />
        </Stack>
      </Group>
    </Paper>
  )
}
