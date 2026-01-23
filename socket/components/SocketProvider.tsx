import { type PropsWithChildren, useEffect, useState } from "react"
import { io } from "socket.io-client"
import { SocketContext } from "../context/socket.ts"
import { parser } from "../lib/parser.ts"
import type { AppSocket } from "../lib/types.ts"

export const SocketProvider = ({ children }: PropsWithChildren) => {
  const [socket, setSocket] = useState<AppSocket>()

  useEffect(() => {
    const newSocket = io({
      path: "/socket.io/",
      transports: ["websocket", "polling"],
      parser,
    })

    setSocket(newSocket)

    return () => {
      newSocket.close()
    }
  }, [])

  return (
    <SocketContext value={socket}>
      {children}
    </SocketContext>
  )
}
