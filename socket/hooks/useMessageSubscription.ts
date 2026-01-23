import { useContext, useEffect, useRef } from "react"
import type { Message } from "../../api/twittra.schemas.ts"
import { SocketContext } from "../context/socket.ts"

/**
 * Hook to manage message subscriptions via Socket.io.
 * Automatically subscribes to new message IDs and unsubscribes from removed ones.
 * Optionally handles messageUpdated events via callback.
 */
export const useMessageSubscription = (
  messageIds: string[],
  onMessageUpdated?: (message: Message) => void,
) => {
  const socket = useContext(SocketContext)
  const subscribedIdsRef = useRef<Set<string>>(new Set())

  // Effect to manage subscriptions based on messageIds changes
  useEffect(() => {
    if (!socket) return

    const currentIds = new Set(messageIds)
    const previousIds = subscribedIdsRef.current
    // Find IDs to subscribe (new IDs)
    const toSubscribe = Array.from(currentIds.difference(previousIds))
    // Find IDs to unsubscribe (removed IDs)
    const toUnsubscribe = Array.from(previousIds.difference(currentIds))

    // Subscribe to new messages
    if (toSubscribe.length > 0) {
      socket.emit("subscribe", { messageIds: toSubscribe })
    }

    // Unsubscribe from removed messages
    if (toUnsubscribe.length > 0) {
      socket.emit("unsubscribe", { messageIds: toUnsubscribe })
    }

    // Update the ref
    subscribedIdsRef.current = currentIds
  }, [messageIds, socket])

  // Effect to handle messageUpdated events
  useEffect(() => {
    if (!socket || !onMessageUpdated) return

    socket.on("messageUpdated", onMessageUpdated)

    return () => {
      socket.off("messageUpdated", onMessageUpdated)
    }
  }, [socket, onMessageUpdated])
}
