"use strict";
// CJS shim for file-type (pure ESM in v18+) so Jest can require() it.
module.exports = {
  fileTypeFromBuffer: async () => undefined,
  fileTypeFromStream: async () => undefined,
  fileTypeStream: async (stream) => stream,
};
