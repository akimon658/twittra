import { act, renderHook } from "@testing-library/react"
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  type Mock,
  vi,
} from "vitest"
import { useReadManagement } from "./useReadManagement.ts"
import { useMarkMessagesAsRead } from "../../api/message/message.ts"

// Mocking the module with correct path
vi.mock("../../api/message/message.ts", () => ({
  useMarkMessagesAsRead: vi.fn(),
}))

describe("useReadManagement", () => {
  const mutateMock = vi.fn()

  beforeEach(() => {
    ;(useMarkMessagesAsRead as unknown as Mock).mockReturnValue({
      mutate: mutateMock,
    })
    mutateMock.mockClear()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it("batches read messages", () => {
    const { result } = renderHook(() => useReadManagement())

    act(() => {
      result.current.markAsRead("1")
      result.current.markAsRead("2")
    })

    expect(mutateMock).not.toHaveBeenCalled()

    act(() => {
      vi.advanceTimersByTime(2000)
    })

    expect(mutateMock).toHaveBeenCalledTimes(1)
    expect(mutateMock).toHaveBeenCalledWith(expect.objectContaining({
      data: expect.objectContaining({
        message_ids: expect.arrayContaining(["1", "2"]),
      }),
    }))
  })

  it("ignores duplicates", () => {
    const { result } = renderHook(() => useReadManagement())

    act(() => {
      result.current.markAsRead("1")
      result.current.markAsRead("1")
    })

    act(() => {
      vi.advanceTimersByTime(2000)
    })

    expect(mutateMock).toHaveBeenCalledWith(expect.objectContaining({
      data: expect.objectContaining({
        message_ids: ["1"],
      }),
    }))
  })

  it("flushes on unmount", () => {
    const { result, unmount } = renderHook(() => useReadManagement())

    act(() => {
      result.current.markAsRead("1")
    })

    expect(mutateMock).not.toHaveBeenCalled()

    unmount()

    expect(mutateMock).toHaveBeenCalled()
  })
})
