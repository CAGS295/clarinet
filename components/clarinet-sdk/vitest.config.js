/// <reference types="vitest" />

// doc: https://github.com/vitest-dev/vitest/blob/main/test/env-custom/vitest.config.ts

import { defineConfig } from "vite";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

const defaultArgs = {
  clarityCoverage: false,
};

function getArgv() {
  const argv = hideBin(process.argv);
  const topLevel = yargs(argv).argv;
  // @ts-ignore
  const clarinetArgv = yargs(topLevel._).option("clarity-coverage", {
    alias: "cov",
    type: "boolean",
  }).argv;
  return { ...defaultArgs, ...clarinetArgv };
}

const argv = getArgv();

console.log("-".repeat(20));
console.log(process.cwd());

export default defineConfig({
  test: {
    environment: "clarinet",
    singleThread: true,
    setupFiles: ["node_modules/obscurity-sdk/vitest.setup.js"],
    environmentOptions: {
      clarinet: argv,
    },
  },
});
