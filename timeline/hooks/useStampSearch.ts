
import { useGetStampsSuspense } from "../../api/stamp/stamp.ts"


export const useStampSearch = () => {
    // Fetch all stamps, enabling caching/suspense
    // The response is wrapped in a data property
    const { data: { data: stamps } } = useGetStampsSuspense()

    const findStampByName = (name: string) => {
        const normalizedName = name.toLowerCase().trim()
        return stamps.find((s) => s.name.toLowerCase() === normalizedName)
    }

    const findStampById = (id: string) => {
        return stamps.find((s) => s.id === id)
    }

    return {
        stamps,
        findStampByName,
        findStampById
    }
}
