import { renderHook } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import type { TypedSocket } from "../app/typedSocket.ts"
import { useMessageSubscription } from "./useMessageSubscription.ts"

// Mock the SocketProvider
const mockSocket = {
  emit: vi.fn(),
} as unknown as TypedSocket

vi.mock("../app/SocketProvider.tsx", () => ({
  useSocket: () => ({ socket: mockSocket, isConnected: true }),
}))

describe("useMessageSubscription", () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it("subscribes to new message IDs", () => {
    const { rerender } = renderHook(
      ({ ids }) => useMessageSubscription(ids),
      {
        initialProps: { ids: [] },
      },
    )

    // Add new message IDs
    rerender({ ids: ["msg-1", "msg-2"] })

    expect(mockSocket.emit).toHaveBeenCalledTimes(2)
    expect(mockSocket.emit).toHaveBeenCalledWith("subscribe", {
      messageId: "msg-1",
    })
    expect(mockSocket.emit).toHaveBeenCalledWith("subscribe", {
      messageId: "msg-2",
    })
  })

  it("unsubscribes from removed message IDs", () => {
    const { rerender } = renderHook(
      ({ ids }) => useMessageSubscription(ids),
      {
        initialProps: { ids: ["msg-1", "msg-2", "msg-3"] },
      },
    )

    vi.clearAllMocks()

    // Remove some message IDs
    rerender({ ids: ["msg-2"] })

    expect(mockSocket.emit).toHaveBeenCalledTimes(2)
    expect(mockSocket.emit).toHaveBeenCalledWith("unsubscribe", {
      messageId: "msg-1",
    })
    expect(mockSocket.emit).toHaveBeenCalledWith("unsubscribe", {
      messageId: "msg-3",
    })
  })

  it("handles both subscribe and unsubscribe in the same update", () => {
    const { rerender } = renderHook(
      ({ ids }) => useMessageSubscription(ids),
      {
        initialProps: { ids: ["msg-1", "msg-2"] },
      },
    )

    vi.clearAllMocks()

    // Replace message IDs
    rerender({ ids: ["msg-2", "msg-3"] })

    expect(mockSocket.emit).toHaveBeenCalledTimes(2)
    expect(mockSocket.emit).toHaveBeenCalledWith("subscribe", {
      messageId: "msg-3",
    })
    expect(mockSocket.emit).toHaveBeenCalledWith("unsubscribe", {
      messageId: "msg-1",
    })
  })

  it("does nothing when message IDs don't change", () => {
    const { rerender } = renderHook(
      ({ ids }) => useMessageSubscription(ids),
      {
        initialProps: { ids: ["msg-1", "msg-2"] },
      },
    )

    vi.clearAllMocks()

    // Rerender with same IDs
    rerender({ ids: ["msg-1", "msg-2"] })

    expect(mockSocket.emit).not.toHaveBeenCalled()
  })
})
