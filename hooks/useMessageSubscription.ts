import { useEffect, useRef } from "react"
import { useSocket } from "../socket/hooks/useSocket.ts"

/**
 * Hook to manage message subscriptions via Socket.io.
 * Automatically subscribes to new message IDs and unsubscribes from removed ones.
 */
export const useMessageSubscription = (messageIds: string[]) => {
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

    // Subscribe to new messages
    for (const messageId of toSubscribe) {
      socket.emit("subscribe", { messageId })
    }

    // Unsubscribe from removed messages
    for (const messageId of toUnsubscribe) {
      socket.emit("unsubscribe", { messageId })
    }

    // Update the ref
    subscribedIdsRef.current = currentIds
  }, [messageIds, socket])
}
