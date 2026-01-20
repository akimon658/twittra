import { createContext, type ReactNode, useContext } from "react"
import type { Socket } from "socket.io-client"

interface SocketContextType {
    socket: Socket | null
    isConnected: boolean
}

const SocketContext = createContext<SocketContextType | undefined>(undefined)

export const useSocket = () => {
    const context = useContext(SocketContext)
    if (!context) {
        // In test environment, return null socket
        return { socket: null, isConnected: false }
    }
    return context
}

interface MockSocketProviderProps {
    children: ReactNode
}

/**
 * Mock SocketProvider for testing
 * Returns a null socket which is gracefully handled by components
 */
export const MockSocketProvider = ({ children }: MockSocketProviderProps) => {
    return (
        <SocketContext.Provider value={{ socket: null, isConnected: false }}>
            {children}
        </SocketContext.Provider>
    )
}
