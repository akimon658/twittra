// MSW handlers aggregated from all API endpoints
import { getTimelineMock } from "./timeline/timeline.msw"
import { getUserMock } from "./user/user.msw"
import { getStampMock } from "./stamp/stamp.msw"
import { getMessageMock } from "./message/message.msw"

export const handlers = [
    ...getTimelineMock(),
    ...getUserMock(),
    ...getStampMock(),
    ...getMessageMock(),
]
