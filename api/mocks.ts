// MSW handlers aggregated from all API endpoints
import { getMessageMock } from "./message/message.msw.ts"
import { getStampMock } from "./stamp/stamp.msw.ts"
import { getTimelineMock } from "./timeline/timeline.msw.ts"
import { getUserMock } from "./user/user.msw.ts"

export const handlers = [
  ...getMessageMock(),
  ...getStampMock(),
  ...getTimelineMock(),
  ...getUserMock(),
]
