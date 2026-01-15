import { faker } from "@faker-js/faker"
import type { User, Reaction, MessageListItem } from "../api/twittra.schemas"

/**
 * Create a mock user with random data
 */
export function createMockUser(overrides?: Partial<User>): User {
    return {
        id: faker.string.uuid(),
        handle: faker.internet.username(),
        displayName: faker.person.fullName(),
        ...overrides,
    }
}

/**
 * Create a mock reaction with random data
 */
export function createMockReaction(overrides?: Partial<Reaction>): Reaction {
    return {
        stampId: faker.string.uuid(),
        userId: faker.string.uuid(),
        stampCount: faker.number.int({ min: 1, max: 99 }),
        ...overrides,
    }
}

/**
 * Create a mock message with random data
 */
export function createMockMessage(
    overrides?: Partial<MessageListItem>,
): MessageListItem {
    const userId = faker.string.uuid()
    return {
        id: faker.string.uuid(),
        content: faker.lorem.sentence(),
        userId,
        user: createMockUser({ id: userId }),
        channelId: faker.string.uuid(),
        createdAt: faker.date.recent(),
        updatedAt: faker.date.recent(),
        reactions: [],
        ...overrides,
    }
}
