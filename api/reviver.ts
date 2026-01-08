type JSONReviver = Parameters<typeof JSON.parse>[1]

const dateTypeFields = ["createdAt", "updatedAt"] as const

export const customReviver: JSONReviver = (key, value) => {
  if (dateTypeFields.includes(key as typeof dateTypeFields[number])) {
    return new Date(value)
  }

  return value
}
