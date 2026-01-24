import { type InfiniteData, useQueryClient } from "@tanstack/react-query"
import {
  getGetTimelineQueryKey,
  type getTimelineResponseSuccess,
} from "../../api/timeline/timeline.ts"
import type { MessageListItem } from "../../api/twittra.schemas.ts"

export const useTimelineCache = () => {
  const queryClient = useQueryClient()

  const updateMessage = (
    messageId: string,
    updater: (message: MessageListItem) => MessageListItem,
  ) => {
    queryClient.setQueryData<InfiniteData<getTimelineResponseSuccess>>(
      getGetTimelineQueryKey(),
      (oldData) => {
        if (!oldData) return oldData

        return {
          ...oldData,
          pages: oldData.pages.map((page) => ({
            ...page,
            data: page.data.map((item) =>
              item.id === messageId ? updater(item) : item
            ),
          })),
        }
      },
    )
  }

  return { updateMessage }
}
