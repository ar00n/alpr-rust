import { defineConfig } from "orval";

export default defineConfig({
  rustapi: {
    input: {
      target: "http://127.0.0.1:3000/api-docs/openapi.json",
    },
    output: {
      target: "lib/rust_api/schema.ts",
      client: 'react-query',
      httpClient: 'axios',
      override: {
        mutator: {
          path: "./lib/rust_api/custom-axios.ts",
          name: "customAxios",
        },
      },
    },
  },
});
