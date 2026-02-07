import { useParams, useSearch } from "@tanstack/react-router"
import {
    Alert,
    Box,
    Button,
    Center,
    Container,
    Group,
    Paper,
    Skeleton,
    Stack,
    Text,
} from "@mantine/core"
import { IconArrowLeft, IconExclamationCircle, IconReload } from "@tabler/icons-react"
import { QueryErrorResetBoundary } from "@tanstack/react-query"
import { useNavigate } from "@tanstack/react-router"
import { Suspense, useEffect, useRef } from "react"
import { ErrorBoundary, type FallbackProps } from "react-error-boundary"
import { VList, type VListHandle } from "virtua"
import { useChannelMessages } from "../hooks/useChannelMessages.ts"
import { MessageItem } from "./Message.tsx"
import { useReadManagement } from "../../app/hooks/useReadManagement.ts"

const ChannelViewContent = () => {
    const { channelId } = useParams({ from: "/channels/$channelId" })
    const { messageId } = useSearch({ from: "/channels/$channelId" })
    const navigate = useNavigate()
    const listRef = useRef<VListHandle>(null)
    const hasScrolledToAnchor = useRef(false)

    const {
        messages,
        anchorMessageId,
        fetchOlderMessages,
        fetchNewerMessages,
        hasPreviousPage,
        hasNextPage,
        isFetchingPreviousPage,
        isFetchingNextPage,
    } = useChannelMessages({ channelId, messageId })

    const { markAsRead } = useReadManagement()

    // Scroll to anchor message on initial load
    useEffect(() => {
        if (
            anchorMessageId &&
            messages.length > 0 &&
            !hasScrolledToAnchor.current &&
            listRef.current
        ) {
            const anchorIndex = messages.findIndex((m) => m.id === anchorMessageId)
            if (anchorIndex !== -1) {
                // Delay scroll to ensure virtualized list has rendered
                requestAnimationFrame(() => {
                    setTimeout(() => {
                        listRef.current?.scrollToIndex(anchorIndex, { align: "center" })
                        hasScrolledToAnchor.current = true
                    }, 100)
                })
            }
        }
    }, [anchorMessageId, messages])

    return (
        <Box h="100%">
            <Box
                p="xs"
                style={{ borderBottom: "1px solid var(--mantine-color-dark-4)" }}
            >
                <Group>
                    <Button
                        leftSection={<IconArrowLeft size={16} />}
                        onClick={() => navigate({ to: "/" })}
                        size="compact-sm"
                        variant="subtle"
                    >
                        戻る
                    </Button>
                    <Text fw={500} size="sm">
                        チャンネル
                    </Text>
                </Group>
            </Box>

            <VList
                ref={listRef}
                style={{
                    height: "calc(100dvh - 48px)",
                    paddingTop: "var(--mantine-spacing-md)",
                }}
                onRangeChange={(start, end) => {
                    // Load older when reaching top
                    if (start === 0 && hasPreviousPage && !isFetchingPreviousPage) {
                        fetchOlderMessages()
                    }
                    // Load newer when reaching bottom
                    if (
                        end === messages.length - 1 &&
                        hasNextPage &&
                        !isFetchingNextPage
                    ) {
                        fetchNewerMessages()
                    }
                }}
            >
                {isFetchingPreviousPage && (
                    <Center py="md">
                        <Skeleton height={20} width={100} />
                    </Center>
                )}
                {messages.map((item) => (
                    <Box key={item.id} mb="md">
                        <MessageItem message={item} onRead={markAsRead} />
                    </Box>
                ))}
                {isFetchingNextPage && (
                    <Center py="md">
                        <Skeleton height={20} width={100} />
                    </Center>
                )}
            </VList>
        </Box>
    )
}

const LoadingFallback = () => {
    return (
        <Stack pt="md" style={{ overflow: "hidden" }}>
            {Array.from({ length: 10 }).map((_, index) => (
                <Paper key={index}>
                    <Group align="start" wrap="nowrap">
                        <Skeleton circle height={38} />

                        <Stack flex={1} gap="xs">
                            <Skeleton height="1rem" width="10rem" />
                            <Skeleton height="1rem" />
                            <Skeleton height="1rem" />
                            <Skeleton height="1rem" width="80%" />
                        </Stack>
                    </Group>
                </Paper>
            ))}
        </Stack>
    )
}

const ErrorFallback = ({ resetErrorBoundary }: FallbackProps) => {
    return (
        <Container>
            <Center>
                <Alert
                    color="red"
                    icon={<IconExclamationCircle />}
                    title="エラー"
                >
                    <Stack>
                        <Text>チャンネルメッセージの読み込みに失敗しました。</Text>

                        <Button
                            leftSection={<IconReload size={20} />}
                            onClick={resetErrorBoundary}
                        >
                            再読み込み
                        </Button>
                    </Stack>
                </Alert>
            </Center>
        </Container>
    )
}

export const ChannelView = () => {
    return (
        <QueryErrorResetBoundary>
            {({ reset }) => (
                <ErrorBoundary FallbackComponent={ErrorFallback} onReset={reset}>
                    <Suspense fallback={<LoadingFallback />}>
                        <ChannelViewContent />
                    </Suspense>
                </ErrorBoundary>
            )}
        </QueryErrorResetBoundary>
    )
}
