import { Decoder as DefaultDecoder, Encoder } from "socket.io-parser"
import { customReviver } from "../../api/reviver.ts"

class Decoder extends DefaultDecoder {
  constructor() {
    super(customReviver)
  }
}

export const parser = {
  Encoder,
  Decoder,
}
