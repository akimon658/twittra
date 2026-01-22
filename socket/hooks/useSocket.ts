import { useContext } from "react"
import { SocketContext } from "../context/socket.ts"

export const useSocket = () => {
  return useContext(SocketContext)
}
