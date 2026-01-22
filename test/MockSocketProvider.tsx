import type { ReactNode } from "react"
import { SocketContext } from "../socket/context/socket.ts"
import type { AppSocket } from "../socket/lib/types.ts"

interface MockSocketProviderProps {
  children: ReactNode
  socket?: AppSocket
}

/**
 * Mock SocketProvider for testing
 * Accepts an optional socket instance to inject for testing
 */
export const MockSocketProvider = (
  { children, socket = undefined }: MockSocketProviderProps,
) => {
  return (
    <SocketContext.Provider value={socket}>
      {children}
    </SocketContext.Provider>
  )
}
