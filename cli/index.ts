#!/usr/bin/env node
import yargs from "yargs/yargs";
import {
  createVrfAccount,
  requestRandomness,
} from "./actions";

var argv = yargs(process.argv.slice(2)).command(
  `create-vrf [queueKey]`,
  "create a new vrf account for a given queue",
  (yarg) => {
    yarg.positional("queueKey", {
      type: "string",
      describe:
        "public key of the oracle queue that the aggregator will belong to",
      demand: true,
    });
    yarg.option("keypair", {
      type: "string",
      describe: "filesystem path to keypair that will store the vrf account",
    });
    yarg.option("maxResult", {
      type: "string",
      describe: "maximum result returned from vrf buffer",
      default: "256000",
    });
    yarg.option("no-example", {
      type: "boolean",
      describe: "ignore example program state and callback",
      default: false,
    });
  },
  createVrfAccount
).command(`requst-randomness [vrfKey]`, "request randomnesss with a CPI call", () => {}, requestRandomness).argv;
