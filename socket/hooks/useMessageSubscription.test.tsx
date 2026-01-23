import { renderHook } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import type { AppSocket } from "../lib/types.ts"
import { useMessageSubscription } from "./useMessageSubscription.ts"

// Mock the SocketProvider
const mockSocket = {
  emit: vi.fn(),
} as unknown as AppSocket

vi.mock("../socket/hooks/useSocket.ts", () => ({
  useSocket: () => mockSocket,
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
        initialProps: { ids: [] as string[] },
      },
    )

    // Add new message IDs
    rerender({ ids: ["msg-1", "msg-2"] })

    expect(mockSocket.emit).toHaveBeenCalledTimes(1)
    expect(mockSocket.emit).toHaveBeenCalledWith("subscribe", {
      messageIds: ["msg-1", "msg-2"],
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

    expect(mockSocket.emit).toHaveBeenCalledTimes(1)
    expect(mockSocket.emit).toHaveBeenCalledWith("unsubscribe", {
      messageIds: ["msg-1", "msg-3"],
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
      messageIds: ["msg-3"],
    })
    expect(mockSocket.emit).toHaveBeenCalledWith("unsubscribe", {
      messageIds: ["msg-1"],
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

  it("registers and unregisters onMessageUpdated callback", () => {
    const mockCallback = vi.fn()
    const mockOn = vi.fn()
    const mockOff = vi.fn()

    // Temporarily add on/off methods to mockSocket
    Object.assign(mockSocket, {
      on: mockOn,
      off: mockOff,
    })

    const { unmount } = renderHook(() =>
      useMessageSubscription(["msg-1"], mockCallback)
    )

    expect(mockOn).toHaveBeenCalledWith("messageUpdated", mockCallback)

    unmount()

    expect(mockOff).toHaveBeenCalledWith("messageUpdated", mockCallback)
  })
})
