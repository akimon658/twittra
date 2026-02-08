import { useSuspenseInfiniteQuery, useSuspenseQuery } from "@tanstack/react-query"
import { getMessage } from "../../api/message/message.ts"
import { getChannelMessages } from "../../api/channel/channel.ts"
import type { MessageListItem, Order } from "../../api/twittra.schemas.ts"

const MAX_PAGES = 10

interface UseChannelMessagesOptions {
    channelId: string
    messageId?: string
}

type PageParam = {
    direction: "older" | "newer"
    cursor: Date
} | undefined

/**
 * Hook for bidirectional infinite scrolling of channel messages.
 * If messageId is provided, fetches the message first to get its createdAt,
 * then loads messages in both directions around it.
 */
export const useChannelMessages = ({
    channelId,
    messageId,
}: UseChannelMessagesOptions) => {
    // If messageId is provided, fetch it first to get the anchor timestamp
    const anchorQuery = useSuspenseQuery({
        queryKey: ["message", messageId],
        queryFn: async ({ signal }) => {
            if (!messageId) return null
            const response = await getMessage(messageId, { signal })
            return response.data
        },
        staleTime: 60_000,
    })

    const anchorMessage = anchorQuery.data

    const query = useSuspenseInfiniteQuery({
        queryKey: ["channelMessages", channelId, anchorMessage?.createdAt?.toISOString()],
        queryFn: async ({ pageParam, signal }) => {
            // Initial fetch: get messages around the anchor (or latest if no anchor)
            if (!pageParam) {
                if (anchorMessage) {
                    // Fetch older and newer messages around the anchor
                    const [olderResponse, newerResponse] = await Promise.all([
                        getChannelMessages(
                            channelId,
                            { until: anchorMessage.createdAt, order: "desc" as Order },
                            { signal }
                        ),
                        getChannelMessages(
                            channelId,
                            { since: anchorMessage.createdAt, order: "asc" as Order },
                            { signal }
                        ),
                    ])

                    // Combine: older (reversed to asc) + anchor + newer
                    const olderMessages = [...olderResponse.data].reverse()
                    const newerMessages = newerResponse.data.filter(
                        (m) => m.id !== anchorMessage.id
                    )

                    return {
                        messages: [...olderMessages, anchorMessage, ...newerMessages],
                        oldestTimestamp: olderMessages[0]?.createdAt ?? anchorMessage.createdAt,
                        newestTimestamp:
                            newerMessages[newerMessages.length - 1]?.createdAt ??
                            anchorMessage.createdAt,
                    }
                } else {
                    // No anchor: fetch latest messages
                    const response = await getChannelMessages(
                        channelId,
                        { order: "desc" as Order },
                        { signal }
                    )
                    const messages = [...response.data].reverse()
                    return {
                        messages,
                        oldestTimestamp: messages[0]?.createdAt,
                        newestTimestamp: messages[messages.length - 1]?.createdAt,
                    }
                }
            }

            // Paginated fetch
            const { direction, cursor } = pageParam
            if (direction === "older") {
                const response = await getChannelMessages(
                    channelId,
                    { until: cursor, order: "desc" as Order },
                    { signal }
                )
                const messages = [...response.data].reverse()
                return {
                    messages,
                    oldestTimestamp: messages[0]?.createdAt,
                    newestTimestamp: messages[messages.length - 1]?.createdAt,
                }
            } else {
                const response = await getChannelMessages(
                    channelId,
                    { since: cursor, order: "asc" as Order },
                    { signal }
                )
                return {
                    messages: response.data,
                    oldestTimestamp: response.data[0]?.createdAt,
                    newestTimestamp: response.data[response.data.length - 1]?.createdAt,
                }
            }
        },
        getNextPageParam: (lastPage) => {
            if (!lastPage.newestTimestamp || lastPage.messages.length === 0)
                return undefined
            return { direction: "newer" as const, cursor: lastPage.newestTimestamp }
        },
        getPreviousPageParam: (firstPage) => {
            if (!firstPage.oldestTimestamp || firstPage.messages.length === 0)
                return undefined
            return { direction: "older" as const, cursor: firstPage.oldestTimestamp }
        },
        initialPageParam: undefined as PageParam,
        maxPages: MAX_PAGES,
        refetchOnMount: false,
        refetchOnReconnect: false,
        refetchOnWindowFocus: false,
    })

    // Flatten all pages into a single message array
    const messages: MessageListItem[] =
        query.data?.pages.flatMap((page) => page.messages) ?? []

    return {
        messages,
        anchorMessageId: messageId,
        fetchOlderMessages: query.fetchPreviousPage,
        fetchNewerMessages: query.fetchNextPage,
        hasPreviousPage: query.hasPreviousPage,
        hasNextPage: query.hasNextPage,
        isFetchingPreviousPage: query.isFetchingPreviousPage,
        isFetchingNextPage: query.isFetchingNextPage,
    }
}
