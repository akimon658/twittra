import {
  Flex,
  Group,
  Image,
  Pill,
  type PillProps,
  Skeleton,
  Text,
} from "@mantine/core"
import { IconPlus } from "@tabler/icons-react"
import { useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import {
  useAddMessageStamp,
  useRemoveMessageStamp,
} from "../../api/message/message.ts"
import { getGetStampImageUrl, getStamps } from "../../api/stamp/stamp.ts"
import { getGetTimelineQueryKey } from "../../api/timeline/timeline.ts"
import type { Reaction } from "../../api/twittra.schemas.ts"
import { useUser } from "../../auth/hooks/useUser.ts"

interface StampProps {
  stampId: string
}

const Stamp = ({ stampId }: StampProps) => {
  const [isLoading, setIsLoading] = useState(true)

  return (
    <>
      {isLoading && <Skeleton height={16} width={16} />}
      <Image
        src={getGetStampImageUrl(stampId)}
        w={16}
        h={16}
        onLoad={() => setIsLoading(false)}
        onError={() => setIsLoading(false)}
        style={{ display: isLoading ? "none" : undefined }}
      />
    </>
  )
}

interface MessageFooterPillProps extends Omit<PillProps, "size"> {
  isUserReacted?: boolean
}

/**
 * A pill with custom styles for message footer reactions.
 */
const MessageFooterPill = (
  { children, isUserReacted = false, ...props }: MessageFooterPillProps,
) => {
  const defaultBg =
    "light-dark(var(--mantine-color-gray-2), var(--mantine-color-dark-6))"
  const activeBg =
    "light-dark(var(--mantine-color-blue-1), var(--mantine-color-blue-9))"
  const hoverBg =
    "light-dark(var(--mantine-color-gray-3), var(--mantine-color-dark-5))"

  return (
    <Pill
      bg={isUserReacted ? activeBg : defaultBg}
      size="lg"
      style={{
        cursor: "pointer",
        transition: "background-color 0.2s ease",
        ...props.style,
      }}
      styles={{
        root: {
          "&:hover": {
            backgroundColor: isUserReacted ? undefined : hoverBg,
          },
        },
        ...props.styles,
      }}
      {...props}
    >
      {children}
    </Pill>
  )
}

interface MessageFooterProps {
  messageId: string
  reactions: Reaction[]
}

export const MessageFooter = ({ messageId, reactions }: MessageFooterProps) => {
  const user = useUser()
  const queryClient = useQueryClient()
  const { mutate: addStamp } = useAddMessageStamp({
    mutation: {
      onSuccess: () => {
        // Refetch timeline to update the UI
        queryClient.invalidateQueries({ queryKey: getGetTimelineQueryKey() })
      },
      onError: (error) => {
        console.error("Failed to add stamp:", error)
      },
    },
  })

  const { mutate: removeStamp } = useRemoveMessageStamp({
    mutation: {
      onSuccess: () => {
        // Refetch timeline to update the UI
        queryClient.invalidateQueries({ queryKey: getGetTimelineQueryKey() })
      },
      onError: (error) => {
        console.error("Failed to remove stamp:", error)
      },
    },
  })

  const stampCountMap = new Map<string, number>()
  const userStampSet = new Set<string>()

  for (const r of reactions) {
    stampCountMap.set(
      r.stampId,
      (stampCountMap.get(r.stampId) || 0) + r.stampCount,
    )
    // Track if current user has reacted with this stamp
    if (r.userId === user.id) {
      userStampSet.add(r.stampId)
    }
  }

  const groupedReactions = Array.from(stampCountMap.entries()).map((
    [stampId, count],
  ) => ({
    stampId,
    count,
    isUserReacted: userStampSet.has(stampId),
  }))

  const handleStampClick = (stampId: string, isUserReacted: boolean) => {
    if (isUserReacted) {
      // User already reacted, so remove the stamp
      removeStamp({ messageId, stampId })
    } else {
      // User haven't reacted, so add the stamp
      addStamp({ messageId, stampId })
    }
  }

  const handleAddStampClick = async () => {
    const stampName = globalThis.prompt(
      "スタンプ名を入力してください (例: eyes, thumbsup, heart)",
    )

    if (!stampName) {
      // User cancelled or entered nothing
      return
    }

    const trimmedStampName = stampName.trim()
    if (trimmedStampName === "") {
      // Empty string after trimming
      return
    }

    // Use server-side search directly
    const { data: stamps } = await getStamps({ name: trimmedStampName })
    // Find exact match from the search results
    const stamp = stamps.find((s) => s.name === trimmedStampName)

    if (!stamp) {
      alert(`スタンプ "${trimmedStampName}" が見つかりませんでした。`)
      return
    }

    // Add the stamp
    addStamp({ messageId, stampId: stamp.id })
  }

  return (
    <Group gap="xs">
      {groupedReactions.map(({ stampId, count, isUserReacted }) => (
        <MessageFooterPill
          key={stampId}
          isUserReacted={isUserReacted}
          onClick={() => handleStampClick(stampId, isUserReacted)}
        >
          <Group align="center" gap="xs" h="100%" wrap="nowrap">
            <Stamp stampId={stampId} />

            <Text fw={500} size="sm">{count}</Text>
          </Group>
        </MessageFooterPill>
      ))}

      <MessageFooterPill onClick={handleAddStampClick}>
        <Flex align="center" h="100%">
          <IconPlus size={16} />
        </Flex>
      </MessageFooterPill>
    </Group>
  )
}
