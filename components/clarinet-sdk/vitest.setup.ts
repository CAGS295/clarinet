import { afterAll, beforeAll, beforeEach } from "vitest";
import { Cl, ClarityValue } from "@stacks/transactions";

import { ClarityVM } from "./dist/esm";

declare global {
  var vm: ClarityVM;
  var testEnvironment: string;
  var coverageReports: string[];
  var options: {
    clarinet: {
      clarityCoverage: boolean;
    };
  };
}

import { expect } from "vitest";

expect.extend({
  toBeUint(received: ClarityValue, expected: number | bigint) {
    const { isNot } = this;
    return {
      pass: this.equals(received, Cl.uint(expected)),
      message: () => `${received} is${isNot ? " not" : ""} uint(${expected})`,
    };
  },
  toBeInt(received: ClarityValue, expected: number | bigint) {
    const { isNot } = this;
    return {
      pass: this.equals(received, Cl.int(expected)),
      message: () => `${received} is${isNot ? " not" : ""} int(${expected})`,
    };
  },
});

function getFullTestName(task, names) {
  const fullNames = [task.name, ...names];
  if (task.suite?.name) {
    return getFullTestName(task.suite, fullNames);
  }
  return fullNames;
}

beforeAll(async () => {
  await vm.initSession(process.cwd(), "./Clarinet.toml");
});

beforeEach(async (ctx) => {
  if (global.options.clarinet.clarityCoverage) {
    const suiteTestNames = getFullTestName(ctx.task, []);
    const fullName = [ctx.task.file?.name || "", ...suiteTestNames].join("__");
    vm.setCurrentTestName(fullName);
  }
});

afterAll(() => {
  if (global.options.clarinet.clarityCoverage) {
    coverageReports.push(vm.getReport().coverage);
  }
});
