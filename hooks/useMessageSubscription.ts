import { useEffect, useRef } from "react"
import type { Message } from "../api/twittra.schemas.ts"
import { useSocket } from "../socket/hooks/useSocket.ts"

/**
 * Hook to manage message subscriptions via Socket.io.
 * Automatically subscribes to new message IDs and unsubscribes from removed ones.
 * Optionally handles messageUpdated events via callback.
 */
export const useMessageSubscription = (
  messageIds: string[],
  onMessageUpdated?: (message: Message) => void,
) => {
  const socket = useSocket()
  const subscribedIdsRef = useRef<Set<string>>(new Set())

  useEffect(() => {
    if (!socket) return

    const currentIds = new Set(messageIds)
    const previousIds = subscribedIdsRef.current

    // Find IDs to subscribe (new IDs)
    const toSubscribe = messageIds.filter((id) => !previousIds.has(id))

    // Find IDs to unsubscribe (removed IDs)
    const toUnsubscribe = Array.from(previousIds).filter(
      (id) => !currentIds.has(id),
    )

    // Batch subscribe to new messages
    if (toSubscribe.length > 0) {
      socket.emit("subscribe", { messageIds: toSubscribe })
    }

    // Batch unsubscribe from removed messages
    if (toUnsubscribe.length > 0) {
      socket.emit("unsubscribe", { messageIds: toUnsubscribe })
    }

    // Update the ref
    subscribedIdsRef.current = currentIds
  }, [messageIds, socket])

  useEffect(() => {
    if (!socket || !onMessageUpdated) return

    socket.on("messageUpdated", onMessageUpdated)

    return () => {
      socket.off("messageUpdated", onMessageUpdated)
    }
  }, [socket, onMessageUpdated])
}
