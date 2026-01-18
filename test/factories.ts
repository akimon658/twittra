import { faker } from "@faker-js/faker"
import type { MessageListItem, Reaction, User } from "../api/twittra.schemas.ts"

/** Creates a mock user for testing */
export function createMockUser(overrides?: Partial<User>): User {
  return {
    id: faker.string.uuid(),
    handle: faker.internet.username(),
    displayName: faker.person.fullName(),
    ...overrides,
  }
}

/** Creates a mock reaction for testing */
export function createMockReaction(overrides?: Partial<Reaction>): Reaction {
  return {
    stampId: faker.string.uuid(),
    userId: faker.string.uuid(),
    stampCount: faker.number.int({ min: 1, max: 99 }),
    ...overrides,
  }
}

/** Creates a mock message for testing */
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
