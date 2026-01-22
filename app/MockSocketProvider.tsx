import type { ReactNode } from "react"
import type { TypedSocket } from "./typedSocket.ts"
import { SocketContext } from "./SocketProvider.tsx"

interface MockSocketProviderProps {
  children: ReactNode
  socket?: TypedSocket | null
}

/**
 * Mock SocketProvider for testing
 * Accepts an optional socket instance to inject for testing
 */
export const MockSocketProvider = (
  { children, socket = null }: MockSocketProviderProps,
) => {
  return (
    <SocketContext.Provider value={{ socket, isConnected: socket !== null }}>
      {children}
    </SocketContext.Provider>
  )
}
