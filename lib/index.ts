import type { Env } from "@terra-money/terrain";
import { LuncIncineratorClient } from './clients/LuncIncineratorClient';

export class Lib extends LuncIncineratorClient {
  env: Env;

  constructor(env: Env) {
    super(env.client, env.defaultWallet, env.refs['lunc-incinerator'].contractAddresses.default);
    this.env = env;
  }
};

export default Lib;
