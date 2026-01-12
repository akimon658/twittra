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
import { useMemo, useState } from "react"
import { getGetStampImageUrl } from "../../api/stamp/stamp.ts"
import type { Reaction } from "../../api/twittra.schemas.ts"

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

type MessageFooterPillProps = Omit<PillProps, "bg" | "size">

/**
 * A pill with custom styles for message footer reactions.
 */
const MessageFooterPill = ({ children }: MessageFooterPillProps) => {
  return (
    <Pill
      bg="light-dark(var(--mantine-color-gray-2), var(--mantine-color-dark-6))"
      size="lg"
    >
      {children}
    </Pill>
  )
}

interface MessageFooterProps {
  reactions: Reaction[]
}

export const MessageFooter = ({ reactions }: MessageFooterProps) => {
  const groupedReactions = useMemo(() => {
    const map = new Map<string, number>()
    for (const r of reactions) {
      map.set(r.stampId, (map.get(r.stampId) || 0) + r.stampCount)
    }
    return Array.from(map.entries()).map(([stampId, count]) => ({
      stampId,
      count,
    }))
  }, [reactions])

  return (
    <Group gap="xs">
      {groupedReactions.map(({ stampId, count }) => (
        <MessageFooterPill key={stampId}>
          <Group align="center" gap="xs" h="100%" wrap="nowrap">
            <Stamp stampId={stampId} />

            <Text fw={500} size="sm">{count}</Text>
          </Group>
        </MessageFooterPill>
      ))}

      <MessageFooterPill>
        <Flex align="center" h="100%">
          <IconPlus size={16} />
        </Flex>
      </MessageFooterPill>
    </Group>
  )
}
