import { Encoder, Decoder as IoDecoder } from "socket.io-parser"
import { customReviver } from "../../api/reviver.ts"

class Decoder extends IoDecoder {
  constructor() {
    super(customReviver)
  }
}

export const parser = {
  Encoder,
  Decoder,
}
