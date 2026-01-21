import { Decoder as IoDecoder, Encoder } from "socket.io-parser"
import { customReviver } from "../api/reviver.ts"

export { Encoder }

export class Decoder extends IoDecoder {
  constructor() {
    super(customReviver)
  }
}
