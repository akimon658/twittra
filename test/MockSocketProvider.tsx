import type { PropsWithChildren } from "react"
import { SocketContext } from "../socket/context/socket.ts"
import type { AppSocket } from "../socket/lib/types.ts"

interface MockSocketProviderProps {
  socket?: AppSocket
}

/**
 * Mock SocketProvider for testing
 * Accepts an optional socket instance to inject for testing
 */
export const MockSocketProvider = (
  { children, socket }: PropsWithChildren<MockSocketProviderProps>,
) => {
  return (
    <SocketContext.Provider value={socket}>
      {children}
    </SocketContext.Provider>
  )
}
