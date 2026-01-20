import { io, type Socket } from "socket.io-client"
import {
    createContext,
    type ReactNode,
    useContext,
    useEffect,
    useState,
} from "react"

interface SocketContextType {
    socket: Socket | null
    isConnected: boolean
}

const SocketContext = createContext<SocketContextType | undefined>(undefined)

export const useSocket = () => {
    const context = useContext(SocketContext)
    if (!context) {
        throw new Error("useSocket must be used within SocketProvider")
    }
    return context
}

interface SocketProviderProps {
    children: ReactNode
}

export const SocketProvider = ({ children }: SocketProviderProps) => {
    const [socket, setSocket] = useState<Socket | null>(null)
    const [isConnected, setIsConnected] = useState(false)

    useEffect(() => {
        const newSocket = io({
            path: "/socket.io/",
            transports: ["websocket", "polling"],
        })

        newSocket.on("connect", () => {
            console.log("Socket.io connected")
            setIsConnected(true)
        })

        newSocket.on("disconnect", () => {
            console.log("Socket.io disconnected")
            setIsConnected(false)
        })

        setSocket(newSocket)

        return () => {
            newSocket.close()
        }
    }, [])

    return (
        <SocketContext.Provider value={{ socket, isConnected }}>
            {children}
        </SocketContext.Provider>
    )
}
